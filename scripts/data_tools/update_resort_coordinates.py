import argparse
import json
import logging
import os
import time
from pathlib import Path

import requests


API_BASE_URL = os.getenv("API_BASE_URL", "http://localhost:8080")
API_KEY = os.getenv("API_KEY", "R3StTY4OfadeFJZurXdZ1pZMVbWB3zWuL6FnuPGIbvA")
HEADERS = {
    "Content-Type": "application/json",
    "Authorization": f"Bearer {API_KEY}",
}

NOMINATIM_URL = "https://nominatim.openstreetmap.org/search"

ROOT_DIR = Path(__file__).resolve().parents[2]
CACHE_DIR = ROOT_DIR / "checkpoints" / "cleanup"
CACHE_FILE = CACHE_DIR / "resort_geocode_cache.json"

logger = logging.getLogger("update_resort_coordinates")


def api_get(path):
    url = f"{API_BASE_URL}{path}"
    r = requests.get(url, params={"api_key": API_KEY}, headers=HEADERS, timeout=60)
    r.raise_for_status()
    return r.json()


def api_put(path, payload):
    url = f"{API_BASE_URL}{path}"
    r = requests.put(url, json=payload, headers=HEADERS, timeout=60)
    if r.status_code not in (200, 201):
        raise RuntimeError(f"PUT {path} failed: {r.status_code} {r.text}")


def to_float(v):
    try:
        if v is None:
            return None
        return float(v)
    except (TypeError, ValueError):
        return None


def load_cache():
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    if not CACHE_FILE.exists():
        return {}
    try:
        with open(CACHE_FILE, "r", encoding="utf-8") as f:
            data = json.load(f)
        return data if isinstance(data, dict) else {}
    except Exception:
        return {}


def save_cache(cache):
    CACHE_DIR.mkdir(parents=True, exist_ok=True)
    with open(CACHE_FILE, "w", encoding="utf-8") as f:
        json.dump(cache, f, ensure_ascii=True, indent=2)


def centroid_from_entities(resort):
    points = []
    for entity in (resort.get("lifts") or []) + (resort.get("slopes") or []):
        geometry = entity.get("geometry") or {}
        start = geometry.get("start") or {}
        end = geometry.get("end") or {}
        for p in (start, end):
            lat = to_float(p.get("latitude"))
            lon = to_float(p.get("longitude"))
            if lat is not None and lon is not None:
                points.append((lat, lon))

    if not points:
        return None
    lat = sum(p[0] for p in points) / len(points)
    lon = sum(p[1] for p in points) / len(points)
    return (lat, lon)


def nominatim_query(query, timeout_s):
    headers = {
        "User-Agent": "SkiAPI-Cleanup/1.0 (local-maintenance)",
        "Accept": "application/json",
    }
    params = {
        "q": query,
        "format": "jsonv2",
        "limit": 1,
    }
    r = requests.get(NOMINATIM_URL, params=params, headers=headers, timeout=timeout_s)
    r.raise_for_status()
    rows = r.json()
    if not rows:
        return None
    first = rows[0]
    lat = to_float(first.get("lat"))
    lon = to_float(first.get("lon"))
    if lat is None or lon is None:
        return None
    return (lat, lon)


def lookup_osm_coords(resort, cache, timeout_s, delay_s):
    name = (resort.get("name") or "").strip()
    country = ((resort.get("geography") or {}).get("country") or resort.get("country") or "").strip()
    region = ((resort.get("geography") or {}).get("region") or resort.get("region") or "").strip()
    if not name:
        return None

    queries = []
    if region and country:
        queries.append(f"{name}, {region}, {country}")
    if country:
        queries.append(f"{name}, {country}")
    queries.append(f"{name} ski resort")
    queries.append(name)

    for q in queries:
        key = q.lower()
        if key in cache:
            cached = cache[key]
            if cached is None:
                continue
            return (cached["lat"], cached["lon"])

        try:
            coords = nominatim_query(q, timeout_s)
            if coords:
                cache[key] = {"lat": coords[0], "lon": coords[1]}
                time.sleep(delay_s)
                return coords
            cache[key] = None
            time.sleep(delay_s)
        except Exception:
            cache[key] = None
            time.sleep(delay_s)
            continue

    return None


def build_resort_put_payload(resort, new_lat, new_lon):
    geography = resort.get("geography") or {}
    coordinates = geography.get("coordinates") or {}
    altitude = resort.get("altitude") or {}
    ski_area = resort.get("ski_area") or {}
    sources = resort.get("sources") or {}
    live = resort.get("live_status") or {}

    return {
        "name": resort.get("name"),
        "country": geography.get("country") or resort.get("country"),
        "region": geography.get("region") or resort.get("region"),
        "continent": geography.get("continent") or resort.get("continent"),
        "latitude": new_lat if new_lat is not None else coordinates.get("latitude", resort.get("latitude")),
        "longitude": new_lon if new_lon is not None else coordinates.get("longitude", resort.get("longitude")),
        "village_altitude_m": altitude.get("village_m", resort.get("village_altitude_m")),
        "min_altitude_m": altitude.get("min_m", resort.get("min_altitude_m")),
        "max_altitude_m": altitude.get("max_m", resort.get("max_altitude_m")),
        "ski_area_name": ski_area.get("name", resort.get("ski_area_name")),
        "ski_area_type": ski_area.get("area_type", resort.get("ski_area_type")) or "alpine",
        "official_website": sources.get("official_website", resort.get("official_website")),
        "lift_status_url": sources.get("lift_status_url", resort.get("lift_status_url")),
        "slope_status_url": sources.get("slope_status_url", resort.get("slope_status_url")),
        "snow_report_url": sources.get("snow_report_url", resort.get("snow_report_url")),
        "weather_url": sources.get("weather_url", resort.get("weather_url")),
        "status_provider": sources.get("status_provider", resort.get("status_provider")),
        "status_last_scraped_at": live.get("last_scraped_at", resort.get("status_last_scraped_at")),
        "lifts_open_count": live.get("lifts_open_count", resort.get("lifts_open_count")),
        "slopes_open_count": live.get("slopes_open_count", resort.get("slopes_open_count")),
        "snow_depth_valley_cm": live.get("snow_depth_valley_cm", resort.get("snow_depth_valley_cm")),
        "snow_depth_mountain_cm": live.get("snow_depth_mountain_cm", resort.get("snow_depth_mountain_cm")),
        "new_snow_24h_cm": live.get("new_snow_24h_cm", resort.get("new_snow_24h_cm")),
        "temperature_valley_c": live.get("temperature_valley_c", resort.get("temperature_valley_c")),
        "temperature_mountain_c": live.get("temperature_mountain_c", resort.get("temperature_mountain_c")),
    }


def process_resort(resort, cache, timeout_s, delay_s):
    rid = resort.get("id")
    old_lat = to_float(((resort.get("geography") or {}).get("coordinates") or {}).get("latitude", resort.get("latitude")))
    old_lon = to_float(((resort.get("geography") or {}).get("coordinates") or {}).get("longitude", resort.get("longitude")))

    osm_coords = lookup_osm_coords(resort, cache, timeout_s, delay_s)
    if osm_coords:
        return rid, old_lat, old_lon, osm_coords[0], osm_coords[1], "osm"

    centroid = centroid_from_entities(resort)
    if centroid:
        return rid, old_lat, old_lon, centroid[0], centroid[1], "centroid"

    return rid, old_lat, old_lon, old_lat, old_lon, "keep"


def main():
    parser = argparse.ArgumentParser(
        description="Update resort coordinates: OSM first, fallback centroid, else keep."
    )
    parser.add_argument("--resort-ids", default="", help="Optional comma-separated resort ids.")
    parser.add_argument("--nominatim-timeout", type=float, default=12.0)
    parser.add_argument("--nominatim-delay", type=float, default=1.1)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    logging.basicConfig(level=logging.INFO, format="%(asctime)s | %(levelname)s | %(message)s")
    cache = load_cache()
    resorts = api_get("/resorts")
    if args.resort_ids:
        wanted = {x.strip() for x in args.resort_ids.split(",") if x.strip()}
        resorts = [r for r in resorts if r.get("id") in wanted]

    changed = 0
    source_stats = {"osm": 0, "centroid": 0, "keep": 0}

    for idx, resort in enumerate(resorts, start=1):
        rid, old_lat, old_lon, new_lat, new_lon, source = process_resort(
            resort,
            cache,
            timeout_s=args.nominatim_timeout,
            delay_s=args.nominatim_delay,
        )
        source_stats[source] += 1

        if old_lat == new_lat and old_lon == new_lon:
            logger.info("[%s/%s] %s unchanged (%s)", idx, len(resorts), rid, source)
            continue

        logger.info(
            "[%s/%s] %s coord update via %s: (%.6f, %.6f) -> (%.6f, %.6f)",
            idx,
            len(resorts),
            rid,
            source,
            old_lat if old_lat is not None else 0.0,
            old_lon if old_lon is not None else 0.0,
            new_lat if new_lat is not None else 0.0,
            new_lon if new_lon is not None else 0.0,
        )

        if not args.dry_run:
            payload = build_resort_put_payload(resort, new_lat, new_lon)
            api_put(f"/resorts/{rid}", payload)
        changed += 1

    save_cache(cache)
    logger.info(
        "Done. changed=%s total=%s source_counts=%s mode=%s",
        changed,
        len(resorts),
        source_stats,
        "dry-run" if args.dry_run else "apply",
    )


if __name__ == "__main__":
    main()

