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
# Step 1a: Load all ski areas (resorts) from own API
# ---------------------------------------------------------------------------

def load_ski_areas_from_api() -> list[dict]:
    """Load all resorts from own API via GET /resorts."""
    log.info("Loading ski areas from own API (%s) ...", RESORTS_ENDPOINT)
    try:
        response = requests.get(RESORTS_ENDPOINT, headers=HEADERS, timeout=15)
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
        response = requests.get(SLOPES_ENDPOINT, headers=HEADERS, timeout=15)
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
        response = requests.get(f"{SLOPES_ENDPOINT}/{slope_id}", headers=HEADERS, timeout=10)
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
) -> list[tuple[dict, dict]]:
    """
    Return only OSM slopes that also exist in own API (matched by name).
    Returns a list of (osm_slope, api_slope) tuples.
    """
    matches = [
        (osm, api_index[_normalize_name(osm["name"])])
        for osm in osm_slopes
        if _normalize_name(osm["name"]) in api_index
    ]
    log.debug("%d / %d OSM slopes matched with own API.", len(matches), len(osm_slopes))
    return matches


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

def collect_geojson() -> dict:
    """
    Main entry point following the flowchart:
      1a. Load ski areas (resorts) from own API
      1b. Load slopes from own API
      2.  Per ski area: bounding box -> OSM query
      3.  Match OSM slopes against own API
      4.  Extract GeoJSON + direction
      5a. Save GeoJSON via PUT (direction included in properties)
      5b. Save direction separately via extra PUT
    """
    log.info("=" * 60)
    log.info("Starting GeoJSON collection")
    log.info("=" * 60)

    # --- Step 1a: Resorts ---
    ski_areas = load_ski_areas_from_api()
    if not ski_areas:
        log.error("No ski areas found in own API - aborting.")
        return {"type": "FeatureCollection", "features": []}

    # --- Step 1b: Slopes ---
    api_slopes = load_slopes_from_api()
    if not api_slopes:
        log.error("No slopes found in own API - aborting.")
        return {"type": "FeatureCollection", "features": []}

    api_index = build_name_index(api_slopes)
    log.info("Name index built: %d entries.", len(api_index))

    all_features: list[dict] = []

    # --- Step 2: Per ski area ---
    for ski_area in ski_areas:
        name   = ski_area.get("name", f"Resort ID {ski_area.get('id', '?')}")
        center = _parse_resort_center(ski_area)

        log.info("-" * 60)
        log.info("Ski area: %s", name)

        if center is None:
            log.warning("No center coordinates found for '%s' - skipping.", name)
            continue

        bbox = build_bounding_box(center[0], center[1], BOUNDING_BOX_RADIUS_KM)
        log.debug("Bounding box: %s", bbox)

        log.info("Fetching OSM slopes ...")
        osm_slopes = fetch_osm_slopes(bbox)
        log.info("  -> %d OSM slopes found.", len(osm_slopes))

        # --- Step 3: Match ---
        matches = filter_known_slopes(osm_slopes, api_index)
        log.info("  -> %d slopes matched with own API.", len(matches))

        for osm_slope, api_slope_raw in matches:
            api_slope = _extract_slope_fields(api_slope_raw)
            slope_id  = api_slope["id"]
            log.info("Processing slope: '%s' (ID=%s)", api_slope["name"], slope_id)

            # Prefer OSM waypoints (more detailed), fall back to API geometry
            if has_waypoints(osm_slope):
                waypoints = osm_slope["coordinates"]
                log.debug("  Waypoints from OSM: %d points", len(waypoints))
            else:
                waypoints = get_api_slope_waypoints(api_slope)
                log.debug("  Waypoints from API geometry: %d points", len(waypoints))

            if not waypoints:
                log.warning("  No waypoints for slope '%s' - skipping.", api_slope["name"])
                continue

            # --- Step 4: Direction + GeoJSON ---
            direction = extract_direction(waypoints)
            if direction is not None:
                log.info("  Direction: %.2f degrees", direction)
            else:
                log.warning("  Direction not available (< 2 waypoints).")

            geojson_feature = build_geojson_feature(api_slope, waypoints, direction)

            # --- Step 5: PUT full slope (GeoJSON + direction) ---
            if save_slope(api_slope, geojson_feature, direction):
                log.info("  Slope saved (GeoJSON + direction) via PUT.")
            else:
                log.error("  Slope PUT failed - kept locally only.")

            all_features.append(geojson_feature)

        log.info("Moving to next ski area ...")

    # Assemble FeatureCollection
    feature_collection = {"type": "FeatureCollection", "features": all_features}

    log.info("=" * 60)
    log.info("Done! %d slopes processed.", len(all_features))
    log.info("=" * 60)

    with open("geojson_slopes.json", "w", encoding="utf-8") as f:
        json.dump(feature_collection, f, ensure_ascii=False, indent=2)
    log.info("Local backup saved: geojson_slopes.json")

    return feature_collection


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------

if __name__ == "__main__":
    result = collect_geojson()
    log.info("Total result: %d features in FeatureCollection.", len(result["features"]))