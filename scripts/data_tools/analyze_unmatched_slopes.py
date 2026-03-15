#!/usr/bin/env python3
"""
Analyze unmatched slopes to identify the root cause of matching failures.

This script will:
1. Load all slopes from the API
2. Load OSM slopes for a specific ski area
3. Analyze unmatched slopes to identify patterns
4. Check for coordinate system issues, missing data, or other problems
"""

import json
import requests
import math
import logging
from typing import Dict, List, Tuple, Optional

# Configuration
OWN_API_BASE_URL = "http://localhost:8080"
API_KEY = "R3StTY4OfadeFJZurXdZ1pZMVbWB3zWuL6FnuPGIbvA"
HEADERS = {
    "Content-Type": "application/json",
    "Authorization": f"Bearer {API_KEY}",
}

SLOPES_ENDPOINT = f"{OWN_API_BASE_URL}/slopes"
RESORTS_ENDPOINT = f"{OWN_API_BASE_URL}/resorts"
OVERPASS_URL = "https://overpass-api.de/api/interpreter"

# Set up logging
logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(levelname)s - %(message)s')
log = logging.getLogger(__name__)


def _haversine_distance(lat1: float, lon1: float, lat2: float, lon2: float) -> float:
    """Calculate the great circle distance between two points on Earth in meters."""
    R = 6371000.0
    lat1_rad = math.radians(lat1)
    lon1_rad = math.radians(lon1)
    lat2_rad = math.radians(lat2)
    lon2_rad = math.radians(lon2)
    
    dlat = lat2_rad - lat1_rad
    dlon = lon2_rad - lon1_rad
    
    a = (math.sin(dlat / 2) ** 2 + 
         math.cos(lat1_rad) * math.cos(lat2_rad) * math.sin(dlon / 2) ** 2)
    c = 2 * math.atan2(math.sqrt(a), math.sqrt(1 - a))
    
    return R * c


def load_ski_areas() -> List[Dict]:
    """Load all ski areas from the API."""
    log.info("Loading ski areas...")
    try:
        response = requests.get(RESORTS_ENDPOINT, headers=HEADERS, timeout=30)
        response.raise_for_status()
        return response.json()
    except Exception as e:
        log.error(f"Failed to load ski areas: {e}")
        return []


def load_slopes() -> List[Dict]:
    """Load all slopes from the API."""
    log.info("Loading slopes...")
    try:
        response = requests.get(SLOPES_ENDPOINT, headers=HEADERS, timeout=30)
        response.raise_for_status()
        return response.json()
    except Exception as e:
        log.error(f"Failed to load slopes: {e}")
        return []


def _parse_resort_center(resort: Dict) -> Optional[Tuple[float, float]]:
    """Extract (lon, lat) center from a resort dict."""
    try:
        coords = resort["geography"]["coordinates"]
        lat = coords["latitude"]
        lon = coords["longitude"]
        if lat is not None and lon is not None:
            return float(lon), float(lat)
    except (KeyError, TypeError):
        pass
    return None


def build_bounding_box(center_lon: float, center_lat: float, radius_km: float) -> Dict:
    """Build a bounding box with given radius around the center point."""
    d_lat = radius_km / 111.0
    d_lon = radius_km / (111.0 * math.cos(math.radians(center_lat)))
    return {
        "min_lat": center_lat - d_lat,
        "min_lon": center_lon - d_lon,
        "max_lat": center_lat + d_lat,
        "max_lon": center_lon + d_lon,
    }


def fetch_osm_slopes(bbox: Dict) -> List[Dict]:
    """Query Overpass API for all downhill slopes within the bounding box."""
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
    
    try:
        response = requests.post(OVERPASS_URL, data={"data": query}, timeout=90)
        response.raise_for_status()
        return _parse_overpass_response(response.json())
    except Exception as e:
        log.error(f"Overpass query failed: {e}")
        return []


def _parse_overpass_response(data: Dict) -> List[Dict]:
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

        coords = [node_map[n] for n in el.get("nodes", []) if n in node_map]
        difficulty = tags.get("piste:difficulty", "unknown")
        name = tags.get("name", "").strip()

        if not name and coords:
            mid = coords[len(coords) // 2]
            name = f"{difficulty} slope [{mid[1]:.4f},{mid[0]:.4f}]"

        slopes.append({
            "osm_id": str(el["id"]),
            "name": name,
            "difficulty": difficulty,
            "coordinates": coords,
        })

    return slopes


def _is_auto_generated_name(name: str) -> bool:
    """Check if a slope name follows the auto-generated pattern."""
    if not name:
        return False
    
    name_lower = name.lower().strip()
    import re
    
    patterns = [
        r'^(green|blue|red|black)\s+slope\s*\[\s*-?\d+(\.\d+)?\s*,\s*-?\d+(\.\d+)?\s*\]$',
        r'^(easy|intermediate|advanced|novice)\s+slope\s*\[\s*-?\d+(\.\d+)?\s*,\s*-?\d+(\.\d+)?\s*\]$',
    ]
    
    return any(re.match(pattern, name_lower) for pattern in patterns)


def _get_start_coords(api_slope: Dict) -> Optional[Tuple[float, float]]:
    """Extract start coordinates from API slope."""
    geometry = api_slope.get("geometry", {})
    start = geometry.get("start")
    if start and start.get("latitude") is not None and start.get("longitude") is not None:
        return (float(start["longitude"]), float(start["latitude"]))
    
    lat_start = api_slope.get("lat_start")
    lon_start = api_slope.get("lon_start")
    if lat_start is not None and lon_start is not None:
        return (float(lon_start), float(lat_start))
    
    return None


def _get_end_coords(api_slope: Dict) -> Optional[Tuple[float, float]]:
    """Extract end coordinates from API slope."""
    geometry = api_slope.get("geometry", {})
    end = geometry.get("end")
    if end and end.get("latitude") is not None and end.get("longitude") is not None:
        return (float(end["longitude"]), float(end["latitude"]))
    
    lat_end = api_slope.get("lat_end")
    lon_end = api_slope.get("lon_end")
    if lat_end is not None and lon_end is not None:
        return (float(lon_end), float(lat_end))
    
    return None


def _get_difficulty(api_slope: Dict) -> Optional[str]:
    """Extract difficulty from API slope."""
    difficulty = api_slope.get("difficulty")
    if difficulty:
        return difficulty.lower().strip()
    
    display = api_slope.get("display", {})
    difficulty = display.get("difficulty")
    if difficulty:
        return difficulty.lower().strip()
    
    return None


def analyze_unmatched_slopes():
    """Analyze unmatched slopes to identify root causes."""
    log.info("Starting analysis of unmatched slopes...")
    
    # Load data
    ski_areas = load_ski_areas()
    api_slopes = load_slopes()
    
    if not ski_areas or not api_slopes:
        log.error("Failed to load required data")
        return
    
    # Find the 3 Zinnen Dolomites ski area
    target_resort = None
    for resort in ski_areas:
        if resort.get("name") == "3 Zinnen Dolomites":
            target_resort = resort
            break
    
    if not target_resort:
        log.error("Could not find '3 Zinnen Dolomites' ski area")
        return
    
    log.info(f"Analyzing ski area: {target_resort['name']}")
    
    # Get bounding box
    center = _parse_resort_center(target_resort)
    if not center:
        log.error("Could not get center coordinates for ski area")
        return
    
    bbox = build_bounding_box(center[0], center[1], 10.0)
    log.info(f"Bounding box: {bbox}")
    
    # Fetch OSM slopes
    osm_slopes = fetch_osm_slopes(bbox)
    log.info(f"Found {len(osm_slopes)} OSM slopes")
    
    # Build API slope index
    api_index = {slope["name"].lower().strip(): slope for slope in api_slopes if slope.get("name")}
    log.info(f"Built API index with {len(api_index)} slopes")
    
    # Separate auto-generated and named slopes
    auto_generated_api = [s for s in api_slopes if _is_auto_generated_name(s.get("name", ""))]
    named_api = [s for s in api_slopes if not _is_auto_generated_name(s.get("name", ""))]
    
    log.info(f"API slopes: {len(api_slopes)} total, {len(auto_generated_api)} auto-generated, {len(named_api)} named")
    
    # Analyze OSM slopes
    auto_generated_osm = [s for s in osm_slopes if _is_auto_generated_name(s["name"])]
    named_osm = [s for s in osm_slopes if not _is_auto_generated_name(s["name"])]
    
    log.info(f"OSM slopes: {len(osm_slopes)} total, {len(auto_generated_osm)} auto-generated, {len(named_osm)} named")
    
    # Analyze auto-generated slopes in detail
    log.info("\n" + "="*80)
    log.info("ANALYSIS OF AUTO-GENERATED SLOPES")
    log.info("="*80)
    
    # Check coordinate patterns in API auto-generated slopes
    api_coords = []
    for slope in auto_generated_api[:10]:  # Sample first 10
        start = _get_start_coords(slope)
        end = _get_end_coords(slope)
        difficulty = _get_difficulty(slope)
        log.info(f"API slope: {slope['name']} | Difficulty: {difficulty} | Start: {start} | End: {end}")
        if start and end:
            api_coords.append((start, end, difficulty))
    
    # Check coordinate patterns in OSM auto-generated slopes
    osm_coords = []
    for slope in auto_generated_osm[:10]:  # Sample first 10
        coords = slope["coordinates"]
        if coords:
            start = (coords[0][0], coords[0][1])  # lon, lat
            end = (coords[-1][0], coords[-1][1])  # lon, lat
            log.info(f"OSM slope: {slope['name']} | Difficulty: {slope['difficulty']} | Start: {start} | End: {end}")
            osm_coords.append((start, end, slope["difficulty"]))
    
    # Try to find matches with different tolerances
    log.info("\n" + "="*80)
    log.info("COORDINATE MATCHING ANALYSIS")
    log.info("="*80)
    
    matches_found = 0
    for osm_start, osm_end, osm_diff in osm_coords:
        for api_start, api_end, api_diff in api_coords:
            # Check difficulty match
            if osm_diff != api_diff:
                continue
            
            # Calculate distances
            start_dist = _haversine_distance(osm_start[1], osm_start[0], api_start[1], api_start[0])
            end_dist = _haversine_distance(osm_end[1], osm_end[0], api_end[1], api_end[0])
            
            log.info(f"Comparing OSM {osm_start} -> {osm_end} with API {api_start} -> {api_end}")
            log.info(f"  Start distance: {start_dist/1000:.2f} km, End distance: {end_dist/1000:.2f} km")
            
            # Try different tolerances
            for tolerance_km in [0.1, 0.5, 1.0, 2.0, 5.0]:
                if start_dist <= tolerance_km * 1000 and end_dist <= tolerance_km * 1000:
                    log.info(f"  MATCH FOUND with {tolerance_km}km tolerance!")
                    matches_found += 1
                    break
    
    log.info(f"\nTotal matches found with various tolerances: {matches_found}")
    
    # Analyze coordinate ranges
    log.info("\n" + "="*80)
    log.info("COORDINATE RANGE ANALYSIS")
    log.info("="*80)
    
    if api_coords:
        api_lons = [c[0][0] for c in api_coords] + [c[1][0] for c in api_coords]
        api_lats = [c[0][1] for c in api_coords] + [c[1][1] for c in api_coords]
        log.info(f"API coordinates range: Lon [{min(api_lons):.4f}, {max(api_lons):.4f}], Lat [{min(api_lats):.4f}, {max(api_lats):.4f}]")
    
    if osm_coords:
        osm_lons = [c[0][0] for c in osm_coords] + [c[1][0] for c in osm_coords]
        osm_lats = [c[0][1] for c in osm_coords] + [c[1][1] for c in osm_coords]
        log.info(f"OSM coordinates range: Lon [{min(osm_lons):.4f}, {max(osm_lons):.4f}], Lat [{min(osm_lats):.4f}, {max(osm_lats):.4f}]")
    
    # Save detailed analysis
    analysis_data = {
        "ski_area": target_resort["name"],
        "bbox": bbox,
        "api_stats": {
            "total": len(api_slopes),
            "auto_generated": len(auto_generated_api),
            "named": len(named_api)
        },
        "osm_stats": {
            "total": len(osm_slopes),
            "auto_generated": len(auto_generated_osm),
            "named": len(named_osm)
        },
        "sample_api_coords": api_coords[:5],
        "sample_osm_coords": osm_coords[:5],
        "matches_found": matches_found
    }
    
    with open("unmatched_slopes_analysis.json", "w", encoding="utf-8") as f:
        json.dump(analysis_data, f, indent=2, ensure_ascii=False)
    
    log.info("Analysis complete. Results saved to 'unmatched_slopes_analysis.json'")


if __name__ == "__main__":
    analyze_unmatched_slopes()