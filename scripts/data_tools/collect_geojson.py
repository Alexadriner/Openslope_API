"""
Step 1: Collect GeoJSON

Process:
  1. Load all ski areas (resorts) from own API
  2. Load all slopes from own API
  3. Per ski area: build bounding box (r=10km) and query OSM
  4. Keep only OSM slopes that also exist in own API (match by name)
  5. Extract GeoJSON (LineString) + direction
  6. Save GeoJSON via PUT (direction included in properties)
  7. Save direction separately via extra PUT (no POST anywhere)

Slope data model (own API):
  - id:          unique ID
  - name:        slope name or "[color] slope [coordinates]"
  - difficulty:  difficulty as color ("blue", "red", "black", ...)
  - coordinates: list of [lon, lat] pairs

# ===========================================================================
# VALUES TO CONFIGURE BEFORE RUNNING
# ===========================================================================
#
#  1. OWN_API_BASE_URL       (line ~35)
#       Base URL of your own API, e.g. "https://my-api.com/api"
#       or "http://localhost:8000/api"
#
#  2. BOUNDING_BOX_RADIUS_KM (line ~38)
#       Radius of the bounding box around each resort center in km.
#       Default is 10. Increase for large ski areas.
#
#  3. LOG_LEVEL               (line ~41)
#       Controls how much output you see. Choose one of:
#         logging.DEBUG    -> everything (very verbose)
#         logging.INFO     -> normal operational messages  [default]
#         logging.WARNING  -> only warnings and errors
#         logging.ERROR    -> only errors
#
#  4. LOG_TO_FILE             (line ~42)
#       True  -> logs are written to LOG_FILE_PATH as well as the console
#       False -> console only
#
#  5. LOG_FILE_PATH           (line ~43)
#       Path of the log file, e.g. "logs/collect_geojson.log"
#       The directory will be created automatically if it does not exist.
#
#  6. _parse_resort_center()  (line ~120)
#       If your /resorts response has a different shape than the four
#       formats already handled (see docstring), add your own branch here.
#
#  7. save_geojson()          (line ~230)
#       The PUT endpoint for GeoJSON is currently:
#         PUT {OWN_API_BASE_URL}/slopes/{id}/geojson
#       Adjust the path if your API uses a different route.
#
#  8. save_direction()        (line ~248)
#       The PUT endpoint for direction is currently:
#         PUT {OWN_API_BASE_URL}/slopes/{id}
#       Adjust the path if your API uses a different route.
#
# ===========================================================================
"""

import json
import logging
import math
import os
import requests
from pathlib import Path
from typing import Optional


# ---------------------------------------------------------------------------
# Configuration  <-- EDIT THESE VALUES
# ---------------------------------------------------------------------------

OWN_API_BASE_URL       = "http://localhost:8080"   # <-- your API base URL
API_KEY                = "R3StTY4OfadeFJZurXdZ1pZMVbWB3zWuL6FnuPGIbvA"           # <-- your API key

HEADERS = {
    "Content-Type": "application/json",
    "Authorization": f"Bearer {API_KEY}",
}

BOUNDING_BOX_RADIUS_KM = 10                            # <-- bounding box radius in km

LOG_LEVEL              = logging.INFO                  # <-- logging.DEBUG / INFO / WARNING / ERROR
LOG_TO_FILE            = False                         # <-- True to also write logs to a file
LOG_FILE_PATH          = "logs/collect_geojson.log"   # <-- log file path (if LOG_TO_FILE=True)

# Debug option for unbenannte Pisten
DEBUG_UNNAMED_SLOPES    = True                         # <-- Set to True to enable debug logging for unnamed slopes

# Storage configuration
SAVE_SINGLE_RESORT_FILE = True                         # <-- Save each resort to a single file that gets overwritten
SINGLE_RESORT_FILE      = "current_resort_geojson.json" # <-- File that always contains the most recent resort

# Configuration for nearest neighbor matching
NEAREST_NEIGHBOR_MAX_DISTANCE_KM = 10.0                 # Maximum distance in km for nearest neighbor matching
COORDINATE_TOLERANCE_KM          = 2.0                  # Tolerance in km for coordinate matching

# Derived endpoints (no need to change these unless your API routes differ)
SLOPES_ENDPOINT  = f"{OWN_API_BASE_URL}/slopes"
RESORTS_ENDPOINT = f"{OWN_API_BASE_URL}/resorts"
OVERPASS_URL     = "https://overpass-api.de/api/interpreter"


# ---------------------------------------------------------------------------
# Logger setup
# ---------------------------------------------------------------------------

def _setup_logger() -> logging.Logger:
    """
    Configure and return the application logger.
    - Always logs to stdout with timestamps and level.
    - Optionally also writes to LOG_FILE_PATH when LOG_TO_FILE is True.
    """
    logger = logging.getLogger("collect_geojson")
    logger.setLevel(LOG_LEVEL)

    formatter = logging.Formatter(
        fmt="%(asctime)s  [%(levelname)-8s]  %(message)s",
        datefmt="%Y-%m-%d %H:%M:%S",
    )

    # Console handler
    console_handler = logging.StreamHandler()
    console_handler.setLevel(LOG_LEVEL)
    console_handler.setFormatter(formatter)
    logger.addHandler(console_handler)

    # Optional file handler
    if LOG_TO_FILE:
        os.makedirs(os.path.dirname(LOG_FILE_PATH), exist_ok=True)
        file_handler = logging.FileHandler(LOG_FILE_PATH, encoding="utf-8")
        file_handler.setLevel(LOG_LEVEL)
        file_handler.setFormatter(formatter)
        logger.addHandler(file_handler)
        logger.info("Logging to file: %s", LOG_FILE_PATH)

    return logger


log = _setup_logger()


# ---------------------------------------------------------------------------
# Configuration Override for Worker Mode
# ---------------------------------------------------------------------------

def override_config_for_worker(worker_id: int, total_workers: int):
    """
    Override configuration when running in worker mode to avoid conflicts.
    - Disable file logging to prevent multiple workers writing to same file
    - Adjust timeouts for better performance under load
    """
    global LOG_TO_FILE, LOG_LEVEL
    
    # Disable file logging in worker mode to prevent conflicts
    LOG_TO_FILE = False
    
    # Reduce log level for workers to reduce noise (optional)
    # LOG_LEVEL = logging.WARNING
    
    log.info(f"Worker {worker_id}/{total_workers} - Configuration adjusted for parallel execution")


# ---------------------------------------------------------------------------
# HTTP Session Management
# ---------------------------------------------------------------------------

import requests.adapters
from urllib3.util.retry import Retry
import time

def create_session_with_retries():
    """Create a requests session with retry strategy for better reliability."""
    session = requests.Session()
    retry_strategy = Retry(
        total=5,  # Increased retries
        backoff_factor=2,  # Increased backoff
        status_forcelist=[429, 500, 502, 503, 504, 408],  # Added 408 timeout
        allowed_methods=["HEAD", "GET", "OPTIONS", "POST", "PUT"]
    )
    adapter = requests.adapters.HTTPAdapter(max_retries=retry_strategy)
    session.mount("http://", adapter)
    session.mount("https://", adapter)
    return session


# Use session for all requests
SESSION = create_session_with_retries()


# ---------------------------------------------------------------------------
# Enhanced API Functions with Better Error Handling
# ---------------------------------------------------------------------------

def load_ski_areas_from_api() -> list[dict]:
    """Load all resorts from own API via GET /resorts with enhanced error handling."""
    log.info("Loading ski areas from own API (%s) ...", RESORTS_ENDPOINT)
    try:
        # Add delay to prevent overwhelming the server
        time.sleep(0.5)
        response = SESSION.get(RESORTS_ENDPOINT, headers=HEADERS, timeout=30)  # Increased timeout
        response.raise_for_status()
        resorts = response.json()
        log.info("  -> %d ski areas loaded.", len(resorts))
        return resorts
    except requests.RequestException as e:
        log.error("Could not load resorts: %s", e)
        return []


def load_slopes_from_api() -> list[dict]:
    """Load all slopes from own API via GET /slopes with enhanced error handling."""
    log.info("Loading all slopes from own API (%s) ...", SLOPES_ENDPOINT)
    try:
        # Add delay to prevent overwhelming the server
        time.sleep(0.5)
        response = SESSION.get(SLOPES_ENDPOINT, headers=HEADERS, timeout=30)  # Increased timeout
        response.raise_for_status()
        slopes = response.json()
        log.info("  -> %d slopes loaded.", len(slopes))
        return slopes
    except requests.RequestException as e:
        log.error("API unreachable: %s", e)
        return []


def fetch_slope_difficulty(slope_id) -> Optional[str]:
    """
    Fetch a single slope from own API to retrieve its difficulty with enhanced error handling.
    Returns the difficulty string if valid, otherwise None.
    """
    try:
        # Add delay to prevent overwhelming the server
        time.sleep(0.1)
        response = SESSION.get(f"{SLOPES_ENDPOINT}/{slope_id}", headers=HEADERS, timeout=15)  # Increased timeout
        response.raise_for_status()
        data       = response.json()
        difficulty = data.get("display", {}).get("difficulty")
        if difficulty in VALID_DIFFICULTIES:
            return difficulty
        log.warning("Slope %s has no valid difficulty in display.difficulty: %s", slope_id, difficulty)
        return None
    except requests.RequestException as e:
        log.error("GET slope %s failed: %s", slope_id, e)
        return None


def save_slope(api_slope: dict, geojson_feature: dict, direction: Optional[float]) -> bool:
    """
    Send a full UpdateSlope payload via PUT /slopes/{id} with enhanced error handling.
    - slope_path_json: serialized GeoJSON LineString geometry string
    - direction:       azimuth in degrees (null if not available)
    - All other fields are taken from the existing API slope to avoid overwriting with nulls.
    """
    slope_id = api_slope["id"]
    url      = f"{SLOPES_ENDPOINT}/{slope_id}"

    # Serialize path as array of {latitude, longitude} objects
    # matching the format expected by parse_path_geojson() in the Rust API
    coords = geojson_feature["geometry"]["coordinates"]
    path_points = [{"latitude": c[1], "longitude": c[0]} for c in coords]
    path_geojson_str = json.dumps(path_points) if path_points else None

    # Extract start/end from GeoJSON coordinates
    coords    = geojson_feature["geometry"]["coordinates"]
    lat_start = coords[0][1]  if coords else None
    lon_start = coords[0][0]  if coords else None
    lat_end   = coords[-1][1] if coords else None
    lon_end   = coords[-1][0] if coords else None

    # resort_id and difficulty are required (non-Option) in UpdateSlope struct
    resort_id  = api_slope.get("resort_id")
    difficulty = api_slope.get("difficulty")

    if not resort_id:
        log.warning("Slope %s has no resort_id - skipping.", slope_id)
        return False

    # difficulty must be a valid enum value - fetch from API if missing
    if difficulty not in VALID_DIFFICULTIES:
        log.debug("Slope %s has no valid difficulty (%s) - fetching from API ...", slope_id, difficulty)
        difficulty = fetch_slope_difficulty(slope_id)
        if difficulty is None:
            log.warning("Slope %s has no valid difficulty - skipping.", slope_id)
            return False

    payload = {
        "resort_id":          resort_id,
        "name":               api_slope.get("name"),
        "difficulty":         difficulty,
        "length_m":           api_slope.get("length_m"),
        "vertical_drop_m":    api_slope.get("vertical_drop_m"),
        "average_gradient":   api_slope.get("average_gradient"),
        "max_gradient":       api_slope.get("max_gradient"),
        "snowmaking":         api_slope.get("snowmaking", False),
        "night_skiing":       api_slope.get("night_skiing", False),
        "family_friendly":    api_slope.get("family_friendly", False),
        "race_slope":         api_slope.get("race_slope", False),
        "lat_start":          lat_start or api_slope.get("lat_start"),
        "lon_start":          lon_start or api_slope.get("lon_start"),
        "lat_end":            lat_end   or api_slope.get("lat_end"),
        "lon_end":            lon_end   or api_slope.get("lon_end"),
        "slope_path_json":    path_geojson_str,
        "direction":          direction,
        "source_system":      api_slope.get("source_system", "osm"),
        "source_entity_id":   api_slope.get("source_entity_id"),
        "name_normalized":    api_slope.get("name_normalized"),
        "operational_status": api_slope.get("operational_status", "unknown"),
        "grooming_status":    api_slope.get("grooming_status", "unknown"),
        "operational_note":   api_slope.get("operational_note"),
        "status_updated_at":  api_slope.get("status_updated_at"),
        "status_source_url":  api_slope.get("status_source_url"),
    }

    try:
        # Add delay to prevent overwhelming the server
        time.sleep(0.2)
        response = SESSION.put(url, json=payload, headers=HEADERS, timeout=15)  # Increased timeout
        if not response.ok:
            log.error("PUT slope %s -> %s | body sent: %s", slope_id, response.status_code, json.dumps(payload, default=str))
        response.raise_for_status()
        log.debug("Slope %s saved (GeoJSON + direction).", slope_id)
        return True
    except requests.RequestException as e:
        log.error("PUT slope failed for ID %s: %s", slope_id, e)
        return False


# ---------------------------------------------------------------------------
# Step 1a: Load all ski areas (resorts) from own API
# ---------------------------------------------------------------------------

def load_ski_areas_from_api() -> list[dict]:
    """Load all resorts from own API via GET /resorts."""
    log.info("Loading ski areas from own API (%s) ...", RESORTS_ENDPOINT)
    try:
        # Use session with retries for better reliability
        response = SESSION.get(RESORTS_ENDPOINT, headers=HEADERS, timeout=60)
        response.raise_for_status()
        resorts = response.json()
        log.info("  -> %d ski areas loaded.", len(resorts))
        return resorts
    except requests.RequestException as e:
        log.error("Could not load resorts: %s", e)
        return []


def _parse_resort_center(resort: dict) -> Optional[tuple[float, float]]:
    """
    Extract (lon, lat) center from a resort dict.
    Expects the API response shape:
      {
        "geography": {
          "coordinates": {
            "latitude": 45.9845,
            "longitude": 7.7481
          }
        }
      }
    """
    try:
        coords = resort["geography"]["coordinates"]
        lat = coords["latitude"]
        lon = coords["longitude"]
        if lat is not None and lon is not None:
            return float(lon), float(lat)
    except (KeyError, TypeError):
        pass
    return None


# ---------------------------------------------------------------------------
# Step 1b: Load all slopes from own API
# ---------------------------------------------------------------------------

def load_slopes_from_api() -> list[dict]:
    """Load all slopes from own API via GET /slopes."""
    log.info("Loading all slopes from own API (%s) ...", SLOPES_ENDPOINT)
    try:
        # Use session with retries for better reliability
        response = SESSION.get(SLOPES_ENDPOINT, headers=HEADERS, timeout=60)
        response.raise_for_status()
        slopes = response.json()
        log.info("  -> %d slopes loaded.", len(slopes))
        return slopes
    except requests.RequestException as e:
        log.error("API unreachable: %s", e)
        return []


VALID_DIFFICULTIES = {"green", "blue", "red", "black"}


def fetch_slope_difficulty(slope_id) -> Optional[str]:
    """
    Fetch a single slope from own API to retrieve its difficulty.
    Returns the difficulty string if valid, otherwise None.
    """
    try:
        # Use session with retries for better reliability
        response = SESSION.get(f"{SLOPES_ENDPOINT}/{slope_id}", headers=HEADERS, timeout=30)
        response.raise_for_status()
        data       = response.json()
        difficulty = data.get("display", {}).get("difficulty")
        if difficulty in VALID_DIFFICULTIES:
            return difficulty
        log.warning("Slope %s has no valid difficulty in display.difficulty: %s", slope_id, difficulty)
        return None
    except requests.RequestException as e:
        log.error("GET slope %s failed: %s", slope_id, e)
        return None


def build_name_index(api_slopes: list[dict]) -> dict[str, dict]:
    """Build a {normalized_name -> slope} index for fast lookup. Skips slopes with no name."""
    skipped = [s for s in api_slopes if not s.get("name")]
    if skipped:
        log.warning("Skipping %d slopes with missing name.", len(skipped))
    return {_normalize_name(s["name"]): s for s in api_slopes if s.get("name")}


def _extract_slope_fields(api_slope: dict) -> dict:
    """
    Flatten the nested API slope response into a simple dict for easy access.
    Handles the nested structure:
      display.difficulty, display.normalized_name
      geometry.start/end/path/direction
      specs.length_m, specs.vertical_drop_m, ...
      source.system, source.entity_id
      status.operational_status, status.grooming_status, ...
    """
    display  = api_slope.get("display")  or {}
    geometry = api_slope.get("geometry") or {}
    specs    = api_slope.get("specs")    or {}
    source   = api_slope.get("source")   or {}
    status   = api_slope.get("status")   or {}
    start    = geometry.get("start")     or {}
    end      = geometry.get("end")       or {}

    return {
        "id":                 api_slope.get("id"),
        "resort_id":          api_slope.get("resort_id"),
        "name":               api_slope.get("name"),
        "difficulty":         display.get("difficulty"),
        "name_normalized":    display.get("normalized_name"),
        "lat_start":          start.get("latitude"),
        "lon_start":          start.get("longitude"),
        "lat_end":            end.get("latitude"),
        "lon_end":            end.get("longitude"),
        "length_m":           specs.get("length_m"),
        "vertical_drop_m":    specs.get("vertical_drop_m"),
        "average_gradient":   specs.get("average_gradient"),
        "max_gradient":       specs.get("max_gradient"),
        "snowmaking":         specs.get("snowmaking", False),
        "night_skiing":       specs.get("night_skiing", False),
        "family_friendly":    specs.get("family_friendly", False),
        "race_slope":         specs.get("race_slope", False),
        "source_system":      source.get("system", "osm"),
        "source_entity_id":   source.get("entity_id"),
        "operational_status": status.get("operational_status", "unknown"),
        "grooming_status":    status.get("grooming_status", "unknown"),
        "operational_note":   status.get("note"),
        "status_updated_at":  status.get("updated_at"),
        "status_source_url":  status.get("source_url"),
    }


def _normalize_name(name: str) -> str:
    return name.strip().lower()


# ---------------------------------------------------------------------------
# Step 2: Bounding box + OSM query
# ---------------------------------------------------------------------------

def _km_to_deg_lat(km: float) -> float:
    return km / 111.0


def _km_to_deg_lon(km: float, lat: float) -> float:
    return km / (111.0 * math.cos(math.radians(lat)))


def build_bounding_box(center_lon: float, center_lat: float, radius_km: float) -> dict:
    """Build a bounding box with given radius around the center point."""
    d_lat = _km_to_deg_lat(radius_km)
    d_lon = _km_to_deg_lon(radius_km, center_lat)
    return {
        "min_lat": center_lat - d_lat,
        "min_lon": center_lon - d_lon,
        "max_lat": center_lat + d_lat,
        "max_lon": center_lon + d_lon,
    }


OVERPASS_RETRIES    = 3
OVERPASS_RETRY_WAIT = 10  # seconds between retries


def fetch_osm_slopes(bbox: dict) -> list[dict]:
    """
    Query Overpass API for all downhill slopes within the bounding box.
    Retries up to OVERPASS_RETRIES times on 504 Gateway Timeout.
    """
    import time

    query = f"""
    [out:json][timeout:60];
    (
      way["piste:type"="downhill"]
         ({bbox['min_lat']},{bbox['min_lon']},{bbox['max_lat']},{bbox['max_lon']});
      relation["piste:type"="downhill"]
              ({bbox['min_lat']},{bbox['min_lon']},{bbox['max_lat']},{bbox['max_lon']});
    );
    out body;
    >;
    out skel qt;
    """
    for attempt in range(1, OVERPASS_RETRIES + 1):
        try:
            response = requests.post(OVERPASS_URL, data={"data": query}, timeout=90)
            response.raise_for_status()
            slopes = _parse_overpass_response(response.json())
            log.debug("Overpass returned %d raw slopes.", len(slopes))
            return slopes
        except requests.RequestException as e:
            log.warning("Overpass attempt %d/%d failed: %s", attempt, OVERPASS_RETRIES, e)
            if attempt < OVERPASS_RETRIES:
                log.info("Retrying in %ds ...", OVERPASS_RETRY_WAIT)
                time.sleep(OVERPASS_RETRY_WAIT)
    log.error("Overpass query failed after %d attempts - skipping ski area.", OVERPASS_RETRIES)
    return []


def _parse_overpass_response(data: dict) -> list[dict]:
    """Parse raw Overpass response into a structured slope list."""
    node_map = {
        el["id"]: [el["lon"], el["lat"]]
        for el in data.get("elements", [])
        if el["type"] == "node"
    }

    slopes = []
    for el in data.get("elements", []):
        if el["type"] not in ("way", "relation"):
            continue
        tags = el.get("tags", {})
        if tags.get("piste:type") != "downhill":
            continue

        coords     = [node_map[n] for n in el.get("nodes", []) if n in node_map]
        difficulty = tags.get("piste:difficulty", "unknown")
        name       = tags.get("name", "").strip()

        if not name and coords:
            mid  = coords[len(coords) // 2]
            name = f"{difficulty} slope [{mid[1]:.4f},{mid[0]:.4f}]"

        slopes.append({
            "osm_id":      str(el["id"]),
            "name":        name,
            "difficulty":  difficulty,
            "coordinates": coords,
        })

    return slopes


# ---------------------------------------------------------------------------
# Step 3: Match OSM slopes against own API
# ---------------------------------------------------------------------------

def filter_known_slopes(
    osm_slopes: list[dict],
    api_index:  dict[str, dict],
    api_slopes: list[dict],
) -> list[tuple[dict, dict]]:
    """
    Return only OSM slopes that also exist in own API.
    For auto-generated names like "[difficulty] slope [coordinates]", match by coordinates and difficulty.
    For regular names, match by name.
    For unmatched unbenannte Pisten, find nearest matching slopes with same difficulty.
    Returns a list of (osm_slope, api_slope) tuples.
    """
    matches = []
    
    # Build coordinate-based index for auto-generated slopes
    coord_index = {}
    for api_slope in api_slopes:
        name = api_slope.get("name", "")
        # Check if this is an auto-generated name
        if _is_auto_generated_name(name):
            start_coords = _get_start_coords(api_slope)
            end_coords = _get_end_coords(api_slope)
            difficulty = _get_difficulty(api_slope)
            
            if start_coords and end_coords and difficulty:
                # Create a key based on start/end coordinates and difficulty
                key = f"{difficulty}_{start_coords[0]:.4f}_{start_coords[1]:.4f}_{end_coords[0]:.4f}_{end_coords[1]:.4f}"
                coord_index[key] = api_slope
    
    # Build difficulty-based index for nearest neighbor matching
    difficulty_index = {}
    for api_slope in api_slopes:
        difficulty = _get_difficulty(api_slope)
        if difficulty:
            if difficulty not in difficulty_index:
                difficulty_index[difficulty] = []
            start_coords = _get_start_coords(api_slope)
            end_coords = _get_end_coords(api_slope)
            if start_coords and end_coords:
                difficulty_index[difficulty].append({
                    'slope': api_slope,
                    'start': start_coords,
                    'end': end_coords
                })
    
    # Debug logging for coordinate matching
    if DEBUG_UNNAMED_SLOPES:
        log.info("  DEBUG: Coordinate index built with %d entries", len(coord_index))
        log.info("  DEBUG: Difficulty index built with %d difficulties", len(difficulty_index))
    
    for osm_slope in osm_slopes:
        osm_name = osm_slope["name"]
        osm_difficulty = osm_slope["difficulty"]
        osm_coords = osm_slope["coordinates"]
        
        # Check if OSM slope has auto-generated name pattern
        if _is_auto_generated_name(osm_name):
            osm_start = osm_coords[0] if osm_coords else None
            osm_end = osm_coords[-1] if osm_coords else None
            
            if osm_start and osm_end:
                # Try exact coordinate match first
                key = f"{osm_difficulty}_{osm_start[0]:.4f}_{osm_start[1]:.4f}_{osm_end[0]:.4f}_{osm_end[1]:.4f}"
                if key in coord_index:
                    matches.append((osm_slope, coord_index[key]))
                    if DEBUG_UNNAMED_SLOPES:
                        log.info("  DEBUG: Exact coordinate match found for OSM slope '%s'", osm_name)
                    continue
                elif DEBUG_UNNAMED_SLOPES:
                    log.info("  DEBUG: No exact coordinate match for OSM slope '%s' (key: %s)", osm_name, key)
                
                # Try tolerance-based coordinate matching
                tolerance_match = _find_tolerance_match(osm_start, osm_end, osm_difficulty, coord_index, COORDINATE_TOLERANCE_KM)
                if tolerance_match:
                    matches.append((osm_slope, tolerance_match))
                    if DEBUG_UNNAMED_SLOPES:
                        log.info("  DEBUG: Tolerance-based coordinate match found for OSM slope '%s'", osm_name)
                    continue
                elif DEBUG_UNNAMED_SLOPES:
                    log.info("  DEBUG: No tolerance-based coordinate match for OSM slope '%s'", osm_name)
                
                # NEW: Try nearest neighbor matching for unbenannte Pisten
                nearest_match = _find_nearest_neighbor_match(osm_start, osm_end, osm_difficulty, difficulty_index, NEAREST_NEIGHBOR_MAX_DISTANCE_KM)
                if nearest_match:
                    matches.append((osm_slope, nearest_match))
                    if DEBUG_UNNAMED_SLOPES:
                        log.info("  DEBUG: Nearest neighbor match found for OSM slope '%s' (distance: %.2f km)", 
                                osm_name, _calculate_distance(osm_start, osm_end, nearest_match))
                    continue
                elif DEBUG_UNNAMED_SLOPES:
                    log.info("  DEBUG: No nearest neighbor match for OSM slope '%s'", osm_name)
                
                # Fallback: Try matching with any difficulty if nearest neighbor fails
                fallback_match = _find_fallback_match(osm_start, osm_end, osm_difficulty, difficulty_index, 20.0)
                if fallback_match:
                    matches.append((osm_slope, fallback_match))
                    if DEBUG_UNNAMED_SLOPES:
                        log.info("  DEBUG: Fallback match found for OSM slope '%s' (distance: %.2f km)", 
                                osm_name, _calculate_distance(osm_start, osm_end, fallback_match))
                    continue
                elif DEBUG_UNNAMED_SLOPES:
                    log.info("  DEBUG: No fallback match for OSM slope '%s'", osm_name)
        
        # Fallback to name-based matching for regular names
        normalized_name = _normalize_name(osm_name)
        if normalized_name in api_index:
            matches.append((osm_slope, api_index[normalized_name]))
            if DEBUG_UNNAMED_SLOPES:
                log.info("  DEBUG: Name match found for OSM slope '%s'", osm_name)
        elif DEBUG_UNNAMED_SLOPES:
            log.info("  DEBUG: No match found for OSM slope '%s'", osm_name)
    
    log.debug("%d / %d OSM slopes matched with own API.", len(matches), len(osm_slopes))
    return matches


def _find_tolerance_match(osm_start, osm_end, osm_difficulty, coord_index, tolerance_km=0.5):
    """
    Find a coordinate match within tolerance distance.
    Returns the matching API slope if found, otherwise None.
    """
    for key, api_slope in coord_index.items():
        # Parse the key to get API coordinates and difficulty
        parts = key.split("_")
        if len(parts) != 5:
            continue
            
        api_difficulty = parts[0]
        try:
            api_start_lon = float(parts[1])
            api_start_lat = float(parts[2])
            api_end_lon = float(parts[3])
            api_end_lat = float(parts[4])
        except ValueError:
            continue
        
        # Check difficulty match first
        if api_difficulty != osm_difficulty:
            continue
        
        # Check if start coordinates are within tolerance
        start_distance = _haversine_distance(osm_start[1], osm_start[0], api_start_lat, api_start_lon)
        if start_distance > tolerance_km * 1000:  # Convert km to meters
            continue
            
        # Check if end coordinates are within tolerance
        end_distance = _haversine_distance(osm_end[1], osm_end[0], api_end_lat, api_end_lon)
        if end_distance > tolerance_km * 1000:  # Convert km to meters
            continue
            
        # Both start and end are within tolerance - this is a match!
        return api_slope
    
    return None


def _find_nearest_neighbor_match(osm_start, osm_end, osm_difficulty, difficulty_index, max_distance_km=5.0):
    """
    Find the nearest matching slope with the same difficulty for unbenannte Pisten.
    Returns the closest matching API slope if found within max_distance_km, otherwise None.
    """
    if osm_difficulty not in difficulty_index:
        return None
    
    candidates = difficulty_index[osm_difficulty]
    if not candidates:
        return None
    
    best_match = None
    best_distance = float('inf')
    
    for candidate in candidates:
        api_slope = candidate['slope']
        api_start = candidate['start']
        api_end = candidate['end']
        
        # Calculate distances between start points
        start_distance = _haversine_distance(osm_start[1], osm_start[0], api_start[1], api_start[0])
        
        # Calculate distances between end points
        end_distance = _haversine_distance(osm_end[1], osm_end[0], api_end[1], api_end[0])
        
        # Use maximum of start and end distances as the matching criterion
        # This ensures both endpoints are reasonably close
        max_endpoint_distance = max(start_distance, end_distance)
        
        # Update best match if this candidate is closer
        if max_endpoint_distance < best_distance:
            best_distance = max_endpoint_distance
            best_match = api_slope
    
    # Only return match if within the maximum allowed distance
    if best_match and best_distance <= max_distance_km * 1000:  # Convert km to meters
        return best_match
    
    return None


def _find_fallback_match(osm_start, osm_end, osm_difficulty, difficulty_index, max_distance_km=20.0):
    """
    Fallback matching that tries to find any slope within a very large radius,
    regardless of exact difficulty match. This helps with cases where:
    1. API has different difficulty classification
    2. OSM has "unknown" difficulty
    3. Coordinates are significantly different
    """
    # Try all difficulties if exact match fails
    all_candidates = []
    for diff, candidates in difficulty_index.items():
        all_candidates.extend(candidates)
    
    if not all_candidates:
        return None
    
    best_match = None
    best_distance = float('inf')
    
    for candidate in all_candidates:
        api_slope = candidate['slope']
        api_start = candidate['start']
        api_end = candidate['end']
        
        # Calculate distances between start points
        start_distance = _haversine_distance(osm_start[1], osm_start[0], api_start[1], api_start[0])
        
        # Calculate distances between end points
        end_distance = _haversine_distance(osm_end[1], osm_end[0], api_end[1], api_end[0])
        
        # Use maximum of start and end distances as the matching criterion
        max_endpoint_distance = max(start_distance, end_distance)
        
        # Update best match if this candidate is closer
        if max_endpoint_distance < best_distance:
            best_distance = max_endpoint_distance
            best_match = api_slope
    
    # Use much larger distance threshold for fallback
    if best_match and best_distance <= max_distance_km * 1000:  # 20km threshold
        return best_match
    
    return None


def _calculate_distance(osm_start, osm_end, api_slope):
    """
    Calculate the average distance between OSM and API slope endpoints for logging purposes.
    """
    api_start = _get_start_coords(api_slope)
    api_end = _get_end_coords(api_slope)
    
    if not api_start or not api_end:
        return float('inf')
    
    start_distance = _haversine_distance(osm_start[1], osm_start[0], api_start[1], api_start[0])
    end_distance = _haversine_distance(osm_end[1], osm_end[0], api_end[1], api_end[0])
    
    return (start_distance + end_distance) / 2000.0  # Return in km


def _is_auto_generated_name(name: str) -> bool:
    """Check if a slope name follows the auto-generated pattern."""
    if not name:
        return False
    
    name_lower = name.lower().strip()
    
    # Check for various patterns of auto-generated names
    import re
    
    # Pattern 1: "blue slope [45.1234,7.5678]" or "red slope [45.1234, 7.5678]"
    pattern1 = r'^(green|blue|red|black)\s+slope\s*\[\s*-?\d+(\.\d+)?\s*,\s*-?\d+(\.\d+)?\s*\]$'
    
    # Pattern 2: "easy slope [45.1234,7.5678]" or "intermediate slope [45.1234, 7.5678]"
    pattern2 = r'^(easy|intermediate|advanced|novice)\s+slope\s*\[\s*-?\d+(\.\d+)?\s*,\s*-?\d+(\.\d+)?\s*\]$'
    
    # Pattern 3: "blue slope [45.1234,7.5678]" or "red slope [45.1234, 7.5678]" (with different difficulty names)
    pattern3 = r'^(green|blue|red|black)\s+slope\s*\[\s*-?\d+(\.\d+)?\s*,\s*-?\d+(\.\d+)?\s*\]$'
    
    # Pattern 4: "intermediate slope [46.7122,12.3713]" (from debug output)
    pattern4 = r'^(intermediate|easy|advanced|novice)\s+slope\s*\[\s*-?\d+(\.\d+)?\s*,\s*-?\d+(\.\d+)?\s*\]$'
    
    return bool(re.match(pattern1, name_lower) or re.match(pattern2, name_lower) or re.match(pattern3, name_lower) or re.match(pattern4, name_lower))


def _get_start_coords(api_slope: dict) -> Optional[tuple[float, float]]:
    """Extract start coordinates from API slope."""
    geometry = api_slope.get("geometry", {})
    start = geometry.get("start")
    if start and start.get("latitude") is not None and start.get("longitude") is not None:
        return (float(start["longitude"]), float(start["latitude"]))
    
    # Fallback to lat_start/lon_start
    lat_start = api_slope.get("lat_start")
    lon_start = api_slope.get("lon_start")
    if lat_start is not None and lon_start is not None:
        return (float(lon_start), float(lat_start))
    
    return None


def _get_end_coords(api_slope: dict) -> Optional[tuple[float, float]]:
    """Extract end coordinates from API slope."""
    geometry = api_slope.get("geometry", {})
    end = geometry.get("end")
    if end and end.get("latitude") is not None and end.get("longitude") is not None:
        return (float(end["longitude"]), float(end["latitude"]))
    
    # Fallback to lat_end/lon_end
    lat_end = api_slope.get("lat_end")
    lon_end = api_slope.get("lon_end")
    if lat_end is not None and lon_end is not None:
        return (float(lon_end), float(lat_end))
    
    return None


def _get_difficulty(api_slope: dict) -> Optional[str]:
    """Extract difficulty from API slope."""
    difficulty = api_slope.get("difficulty")
    if difficulty:
        return difficulty.lower().strip()
    
    # Try display.difficulty
    display = api_slope.get("display", {})
    difficulty = display.get("difficulty")
    if difficulty:
        return difficulty.lower().strip()
    
    return None


# ---------------------------------------------------------------------------
# Step 4: Build GeoJSON + extract direction
# ---------------------------------------------------------------------------

def has_waypoints(slope: dict) -> bool:
    return bool(slope.get("coordinates"))


def get_api_slope_waypoints(api_slope: dict) -> list[list[float]]:
    """
    Extract waypoints from own API slope geometry.
    Converts {"latitude": x, "longitude": y} path points to [lon, lat] pairs.
    Falls back to [start, end] if path is null.
    """
    geometry = api_slope.get("geometry", {})
    path = geometry.get("path")

    if path:
        return [[p["longitude"], p["latitude"]] for p in path]

    # Fallback: use start and end points
    start = geometry.get("start")
    end   = geometry.get("end")
    if start and end:
        return [
            [start["longitude"], start["latitude"]],
            [end["longitude"],   end["latitude"]],
        ]
    return []


def build_geojson_feature(
    api_slope:  dict,
    waypoints:  list[list[float]],
    direction:  Optional[float],
) -> dict:
    """
    Build a GeoJSON Feature (LineString) using data from the own API slope.
    Direction is included in properties if available.
    """
    properties = {
        "id":         api_slope.get("id"),
        "name":       api_slope.get("name"),
        "difficulty": api_slope.get("difficulty", api_slope.get("piste:difficulty", "unknown")),
    }
    if direction is not None:
        properties["direction"] = direction

    return {
        "type": "Feature",
        "geometry": {
            "type":        "LineString",
            "coordinates": waypoints,
        },
        "properties": properties,
    }


def extract_direction(waypoints: list[list[float]]) -> Optional[float]:
    """
    Calculate the slope's main direction as azimuth (0-360 degrees)
    from the first to the last waypoint. Returns None if < 2 points.
    """
    if len(waypoints) < 2:
        return None

    start, end = waypoints[0], waypoints[-1]
    d_lon = math.radians(end[0] - start[0])
    lat1  = math.radians(start[1])
    lat2  = math.radians(end[1])

    x = math.sin(d_lon) * math.cos(lat2)
    y = math.cos(lat1) * math.sin(lat2) - math.sin(lat1) * math.cos(lat2) * math.cos(d_lon)

    return round((math.degrees(math.atan2(x, y)) + 360) % 360, 2)


def interpolate_waypoints(waypoints: list[list[float]], max_distance_m: float = 50.0) -> list[list[float]]:
    """
    Interpolate additional waypoints to ensure maximum distance between points.
    This creates more detailed GeoJSON with smaller segments.
    
    Args:
        waypoints: Original list of [lon, lat] coordinates
        max_distance_m: Maximum distance in meters between consecutive points
    
    Returns:
        List of interpolated waypoints with higher density
    """
    if len(waypoints) < 2:
        return waypoints
    
    interpolated = [waypoints[0]]
    
    for i in range(1, len(waypoints)):
        start = waypoints[i-1]
        end = waypoints[i]
        
        # Calculate distance between current points
        distance = _haversine_distance(start[1], start[0], end[1], end[0])
        
        # If distance is too large, interpolate additional points
        if distance > max_distance_m:
            num_points = math.ceil(distance / max_distance_m)
            
            for j in range(1, num_points):
                ratio = j / num_points
                
                # Linear interpolation
                lat = start[1] + (end[1] - start[1]) * ratio
                lon = start[0] + (end[0] - start[0]) * ratio
                
                interpolated.append([lon, lat])
        
        interpolated.append(end)
    
    return interpolated


def _haversine_distance(lat1: float, lon1: float, lat2: float, lon2: float) -> float:
    """
    Calculate the great circle distance between two points on Earth.
    
    Args:
        lat1, lon1: Latitude and longitude of first point in degrees
        lat2, lon2: Latitude and longitude of second point in degrees
    
    Returns:
        Distance in meters
    """
    # Earth's radius in meters
    R = 6371000.0
    
    # Convert to radians
    lat1_rad = math.radians(lat1)
    lon1_rad = math.radians(lon1)
    lat2_rad = math.radians(lat2)
    lon2_rad = math.radians(lon2)
    
    # Haversine formula
    dlat = lat2_rad - lat1_rad
    dlon = lon2_rad - lon1_rad
    
    a = (math.sin(dlat / 2) ** 2 + 
         math.cos(lat1_rad) * math.cos(lat2_rad) * math.sin(dlon / 2) ** 2)
    c = 2 * math.atan2(math.sqrt(a), math.sqrt(1 - a))
    
    return R * c


# ---------------------------------------------------------------------------
# Progress Tracking Functions
# ---------------------------------------------------------------------------

def save_worker_progress(worker_id: int, processed_resorts: list) -> None:
    """
    Save the progress of a worker to a JSON file.
    Uses atomic file operations to prevent corruption.
    """
    from pathlib import Path
    import tempfile
    
    progress_file = Path("checkpoints") / "collect_geojson" / f"worker_{worker_id}_progress.json"
    progress_file.parent.mkdir(parents=True, exist_ok=True)
    
    progress_data = {"processed_resorts": processed_resorts}
    
    try:
        # Use atomic write to prevent corruption
        temp_file = progress_file.with_suffix('.tmp')
        with open(temp_file, 'w', encoding='utf-8') as f:
            json.dump(progress_data, f, ensure_ascii=False, indent=2)
        temp_file.rename(progress_file)
        log.debug(f"Progress saved for worker {worker_id}: {len(processed_resorts)} resorts")
    except Exception as e:
        log.error(f"Failed to save progress for worker {worker_id}: {e}")


def save_single_resort_file(resort_name: str, resort_id: str, features: list[dict]) -> None:
    """
    Save the current resort to a single file that gets overwritten.
    This file always contains the most recently processed resort.
    """
    if not SAVE_SINGLE_RESORT_FILE:
        return
    
    try:
        resort_data = {
            "type": "FeatureCollection",
            "resort_name": resort_name,
            "resort_id": resort_id,
            "features": features,
            "updated_at": datetime.now(timezone.utc).isoformat()
        }
        
        # Use atomic write to prevent corruption
        temp_file = Path(SINGLE_RESORT_FILE).with_suffix('.tmp')
        with open(temp_file, 'w', encoding='utf-8') as f:
            json.dump(resort_data, f, ensure_ascii=False, indent=2)
        temp_file.rename(SINGLE_RESORT_FILE)
        
        log.info(f"Single resort file saved: {SINGLE_RESORT_FILE} (Resort: {resort_name})")
        
    except Exception as e:
        log.error(f"Failed to save single resort file: {e}")


def clear_progress(worker_id: int = None) -> None:
    """
    Clear progress files for the specified worker or all workers.
    If worker_id is None, clears all progress files.
    """
    from pathlib import Path
    import shutil
    
    checkpoint_dir = Path("checkpoints") / "collect_geojson"
    
    if worker_id is not None:
        # Clear specific worker progress
        progress_file = checkpoint_dir / f"worker_{worker_id}_progress.json"
        if progress_file.exists():
            progress_file.unlink()
            log.info(f"Cleared progress for worker {worker_id}")
        else:
            log.info(f"No progress file found for worker {worker_id}")
    else:
        # Clear all progress files
        if checkpoint_dir.exists():
            shutil.rmtree(checkpoint_dir)
            log.info("Cleared all progress files")
        else:
            log.info("No progress directory found")
    
    # Also clear the single resort file
    single_resort_file = Path(SINGLE_RESORT_FILE)
    if single_resort_file.exists():
        single_resort_file.unlink()
        log.info("Cleared single resort file")


def skip_resort_after_processing(resort_name: str, resort_id: str, features: list[dict]) -> None:
    """
    Debug function: Save the resort data and immediately skip to the next resort.
    Used in debug mode to test the saving logic without processing all slopes.
    """
    if not DEBUG_MODE:
        return
    
    log.info(f"DEBUG MODE: Processing resort '{resort_name}' and then skipping to next...")
    
    # Save the current resort data
    save_single_resort_file(resort_name, resort_id, features)
    
    # Immediately skip to next resort
    log.info(f"DEBUG MODE: Skipping remaining processing for '{resort_name}', moving to next resort")


# ---------------------------------------------------------------------------
# Step 5: Save GeoJSON + direction via PUT /slopes/{id}
# ---------------------------------------------------------------------------

def save_slope(api_slope: dict, geojson_feature: dict, direction: Optional[float]) -> bool:
    """
    Send a full UpdateSlope payload via PUT /slopes/{id}.
    - slope_path_json: serialized GeoJSON LineString geometry string
    - direction:       azimuth in degrees (null if not available)
    - All other fields are taken from the existing API slope to avoid overwriting with nulls.
    """
    slope_id = api_slope["id"]
    url      = f"{SLOPES_ENDPOINT}/{slope_id}"

    # Serialize path as array of {latitude, longitude} objects
    # matching the format expected by parse_path_geojson() in the Rust API
    coords = geojson_feature["geometry"]["coordinates"]
    path_points = [{"latitude": c[1], "longitude": c[0]} for c in coords]
    path_geojson_str = json.dumps(path_points) if path_points else None

    # Extract start/end from GeoJSON coordinates
    coords    = geojson_feature["geometry"]["coordinates"]
    lat_start = coords[0][1]  if coords else None
    lon_start = coords[0][0]  if coords else None
    lat_end   = coords[-1][1] if coords else None
    lon_end   = coords[-1][0] if coords else None

    # resort_id and difficulty are required (non-Option) in UpdateSlope struct
    resort_id  = api_slope.get("resort_id")
    difficulty = api_slope.get("difficulty")

    if not resort_id:
        log.warning("Slope %s has no resort_id - skipping.", slope_id)
        return False

    # difficulty must be a valid enum value - fetch from API if missing
    if difficulty not in VALID_DIFFICULTIES:
        log.debug("Slope %s has no valid difficulty (%s) - fetching from API ...", slope_id, difficulty)
        difficulty = fetch_slope_difficulty(slope_id)
        if difficulty is None:
            log.warning("Slope %s has no valid difficulty - skipping.", slope_id)
            return False

    payload = {
        "resort_id":          resort_id,
        "name":               api_slope.get("name"),
        "difficulty":         difficulty,
        "length_m":           api_slope.get("length_m"),
        "vertical_drop_m":    api_slope.get("vertical_drop_m"),
        "average_gradient":   api_slope.get("average_gradient"),
        "max_gradient":       api_slope.get("max_gradient"),
        "snowmaking":         api_slope.get("snowmaking", False),
        "night_skiing":       api_slope.get("night_skiing", False),
        "family_friendly":    api_slope.get("family_friendly", False),
        "race_slope":         api_slope.get("race_slope", False),
        "lat_start":          lat_start or api_slope.get("lat_start"),
        "lon_start":          lon_start or api_slope.get("lon_start"),
        "lat_end":            lat_end   or api_slope.get("lat_end"),
        "lon_end":            lon_end   or api_slope.get("lon_end"),
        "slope_path_json":    path_geojson_str,
        "direction":          direction,
        "source_system":      api_slope.get("source_system", "osm"),
        "source_entity_id":   api_slope.get("source_entity_id"),
        "name_normalized":    api_slope.get("name_normalized"),
        "operational_status": api_slope.get("operational_status", "unknown"),
        "grooming_status":    api_slope.get("grooming_status", "unknown"),
        "operational_note":   api_slope.get("operational_note"),
        "status_updated_at":  api_slope.get("status_updated_at"),
        "status_source_url":  api_slope.get("status_source_url"),
    }

    try:
        response = requests.put(url, json=payload, headers=HEADERS, timeout=10)
        if not response.ok:
            log.error("PUT slope %s -> %s | body sent: %s", slope_id, response.status_code, json.dumps(payload, default=str))
        response.raise_for_status()
        log.debug("Slope %s saved (GeoJSON + direction).", slope_id)
        return True
    except requests.RequestException as e:
        log.error("PUT slope failed for ID %s: %s", slope_id, e)
        return False


# ---------------------------------------------------------------------------
# Main pipeline
# ---------------------------------------------------------------------------

def collect_geojson(worker_id: int = 0, total_workers: int = 1) -> dict:
    """
    Main entry point following the flowchart:
      1a. Load ski areas (resorts) from own API
      1b. Load slopes from own API
      2.  Per ski area: bounding box -> OSM query
      3.  Match OSM slopes against own API
      4.  Interpolate waypoints for more detailed GeoJSON (max 50m spacing)
      5.  Extract GeoJSON + direction
      6a. Save GeoJSON via PUT (direction included in properties)
      6b. Save direction separately via extra PUT
    """
    # Override configuration for worker mode
    override_config_for_worker(worker_id, total_workers)
    
    log.info("=" * 60)
    log.info(f"Starting GeoJSON collection (Worker {worker_id}/{total_workers})")
    log.info("=" * 60)

    # --- Step 1a: Resorts ---
    ski_areas = load_ski_areas_from_api()
    if not ski_areas:
        log.error("No ski areas found in own API - aborting.")
        return {"type": "FeatureCollection", "features": []}

    # Filter ski areas by worker assignment
    filtered_ski_areas = [
        area for i, area in enumerate(ski_areas)
        if i % total_workers == worker_id
    ]
    log.info(f"Worker {worker_id} assigned {len(filtered_ski_areas)} ski areas out of {len(ski_areas)} total.")
    
    # Early exit if no ski areas assigned to this worker
    if not filtered_ski_areas:
        log.info(f"Worker {worker_id} has no ski areas to process. Exiting early.")
        return {"type": "FeatureCollection", "features": []}

    # --- Step 1b: Slopes ---
    api_slopes = load_slopes_from_api()
    if not api_slopes:
        log.error("No slopes found in own API - aborting.")
        return {"type": "FeatureCollection", "features": []}

    api_index = build_name_index(api_slopes)
    log.info("Name index built: %d entries.", len(api_index))

    all_features: list[dict] = []

    # --- Progress tracking ---
    from pathlib import Path
    progress_file = Path("checkpoints") / "collect_geojson" / f"worker_{worker_id}_progress.json"
    progress_file.parent.mkdir(parents=True, exist_ok=True)
    
    # Load existing progress
    processed_resorts = set()
    if progress_file.exists():
        try:
            with open(progress_file, "r", encoding="utf-8") as f:
                data = json.load(f)
                processed_resorts = set(data.get("processed_resorts", []))
                log.info(f"Resumed from checkpoint: {len(processed_resorts)} resorts already processed")
        except (json.JSONDecodeError, KeyError):
            log.warning("Could not load progress file, starting fresh")

    # --- Step 2: Per ski area ---
    for ski_area in filtered_ski_areas:
        # Skip already processed resorts
        resort_id = ski_area.get("id")
        if resort_id in processed_resorts:
            log.info(f"Skipping already processed resort: {ski_area.get('name', resort_id)}")
            continue
        name   = ski_area.get("name", f"Resort ID {ski_area.get('id', '?')}")
        center = _parse_resort_center(ski_area)

        log.info("-" * 60)
        log.info("Ski area: %s", name)

        if center is None:
            log.warning("No center coordinates found for '%s' - skipping.", name)
            continue

        bbox = build_bounding_box(center[0], center[1], BOUNDING_BOX_RADIUS_KM)
        log.debug("Bounding box: %s", bbox)

        # Save debug mode: skip to next resort immediately after finding it, without processing slopes
        if DEBUG_MODE:
            log.info(f"SAVE DEBUG MODE: Resort '{name}' found, saving and moving to next resort without processing slopes")
            # Save empty features for this resort (just the resort info)
            save_single_resort_file(name, resort_id, [])
            # Save progress
            processed_resorts.add(resort_id)
            save_worker_progress(worker_id, list(processed_resorts))
            log.info("Moving to next ski area ...")
            continue

        log.info("Fetching OSM slopes ...")
        osm_slopes = fetch_osm_slopes(bbox)
        log.info("  -> %d OSM slopes found.", len(osm_slopes))

        # --- Step 3: Match ---
        matches = filter_known_slopes(osm_slopes, api_index, api_slopes)
        log.info("  -> %d slopes matched with own API.", len(matches))

        # Debug logging for matching process
        if DEBUG_UNNAMED_SLOPES:
            log.info("  DEBUG: OSM slopes found: %d", len(osm_slopes))
            log.info("  DEBUG: API slopes in index: %d", len(api_index))
            # Count auto-generated names in API slopes
            auto_generated_count = sum(1 for name in api_index.keys() if _is_auto_generated_name(name))
            log.info("  DEBUG: Auto-generated names in API: %d", auto_generated_count)

        # Count unnamed slopes for debug summary
        unnamed_count = 0
        named_count = 0

        for osm_slope, api_slope_raw in matches:
            api_slope = _extract_slope_fields(api_slope_raw)
            slope_id  = api_slope["id"]
            slope_name = api_slope["name"]
            
            # Debug logging for unnamed slopes
            if DEBUG_UNNAMED_SLOPES and _is_auto_generated_name(slope_name):
                log.info("DEBUG: Processing UNNAMED slope: '%s' (ID=%s)", slope_name, slope_id)
                unnamed_count += 1
            else:
                log.info("Processing slope: '%s' (ID=%s)", slope_name, slope_id)
                named_count += 1

            # Prefer OSM waypoints (more detailed), fall back to API geometry
            if has_waypoints(osm_slope):
                waypoints = osm_slope["coordinates"]
                log.debug("  Waypoints from OSM: %d points", len(waypoints))
            else:
                waypoints = get_api_slope_waypoints(api_slope)
                log.debug("  Waypoints from API geometry: %d points", len(waypoints))

            if not waypoints:
                log.warning("  No waypoints for slope '%s' - skipping.", slope_name)
                continue

            # --- Step 4: Interpolate waypoints for more detailed GeoJSON ---
            # Ensure maximum distance of 50 meters between points
            interpolated_waypoints = interpolate_waypoints(waypoints, max_distance_m=50.0)
            if len(interpolated_waypoints) > len(waypoints):
                log.info("  Interpolated %d waypoints to %d points for better detail", 
                        len(waypoints), len(interpolated_waypoints))

            # --- Step 5: Direction + GeoJSON ---
            direction = extract_direction(interpolated_waypoints)
            if direction is not None:
                log.info("  Direction: %.2f degrees", direction)
            else:
                log.warning("  Direction not available (< 2 waypoints).")

            geojson_feature = build_geojson_feature(api_slope, interpolated_waypoints, direction)

            # --- Step 5: PUT full slope (GeoJSON + direction) ---
            if save_slope(api_slope, geojson_feature, direction):
                log.info("  Slope saved (GeoJSON + direction) via PUT.")
            else:
                log.error("  Slope PUT failed - kept locally only.")

            all_features.append(geojson_feature)

        log.info("Moving to next ski area ...")

        # Save the current resort to the single file (overwrites previous)
        if all_features:
            save_single_resort_file(name, resort_id, all_features)
        
        # Save progress after each resort
        processed_resorts.add(resort_id)
        save_worker_progress(worker_id, list(processed_resorts))

    # Assemble FeatureCollection
    feature_collection = {"type": "FeatureCollection", "features": all_features}

    log.info("=" * 60)
    log.info("Done! %d slopes processed.", len(all_features))
    log.info("=" * 60)

    # Save worker-specific output
    output_file = f"geojson_slopes_worker_{worker_id}.json"
    with open(output_file, "w", encoding="utf-8") as f:
        json.dump(feature_collection, f, ensure_ascii=False, indent=2)
    log.info("Local backup saved: %s", output_file)

    return feature_collection


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    import sys
    import argparse
    from datetime import datetime, timezone
    
    # Parse command line arguments
    parser = argparse.ArgumentParser(description="Collect GeoJSON for ski slopes")
    parser.add_argument("worker_id", nargs="?", type=int, default=0, help="Worker ID (default: 0)")
    parser.add_argument("total_workers", nargs="?", type=int, default=1, help="Total number of workers (default: 1)")
    parser.add_argument("--save_debug", action="store_true", help="Enable save debug mode - find resorts, save locally, then skip to next resort without processing slopes")
    parser.add_argument("--clear", nargs="*", type=int, help="Clear progress files. If no IDs provided, clears all progress files. If IDs provided, clears only those worker progress files.")
    
    args = parser.parse_args()
    
    # Handle clear command first
    if args.clear is not None:
        if len(args.clear) == 0:
            # Clear all progress files
            clear_progress()
            log.info("All progress files cleared. You can now start from scratch.")
        else:
            # Clear specific worker progress files
            for worker_id in args.clear:
                clear_progress(worker_id)
            log.info("Progress files cleared for workers: %s", args.clear)
        sys.exit(0)
    
    # Set debug mode
    global DEBUG_MODE
    DEBUG_MODE = args.save_debug
    
    if DEBUG_MODE:
        log.info("SAVE DEBUG MODE ENABLED: Resorts will be processed and saved, then skipped to next resort")
    
    log.info(f"Script started with worker_id={args.worker_id}, total_workers={args.total_workers}, debug={DEBUG_MODE}")
    
    result = collect_geojson(args.worker_id, args.total_workers)
    log.info("Total result: %d features in FeatureCollection.", len(result["features"]))
