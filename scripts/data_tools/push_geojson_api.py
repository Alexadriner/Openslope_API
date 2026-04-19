"""
Push GeoJSON data from /data/geodata to the OpenSlope API.

This script reads the large geojson files under data/geodata and sends each
feature to the appropriate API endpoint.

It supports:
- lifts.geojson -> /lifts
- runs.geojson -> /slopes
- ski_areas.geojson -> /resorts

Timeout handling:
- HTTP 408 + timeout exceptions are retried with exponential backoff
- On success or non-retryable error, the request is returned

Usage examples:
    python push_geojson_api.py --all --dry-run
    python push_geojson_api.py --file data/geodata/lifts.geojson --limit 10
    python push_geojson_api.py --validate
"""

import argparse
import json
import logging
import sys
import time
from pathlib import Path
from typing import Any, Dict, Iterator, Optional

import requests
from requests.adapters import HTTPAdapter
from requests.exceptions import RequestException, Timeout
from urllib3.util.retry import Retry

from config import (
    API_BASE_URL,
    API_KEY,
    HEADERS,
    HTTP_RETRIES,
    REQUEST_DELAY,
    RESORTS_ENDPOINT,
    LIFTS_ENDPOINT,
    SLOPES_ENDPOINT,
    IMPORT_LIFTS_ENDPOINT,
    IMPORT_SLOPES_ENDPOINT,
    IMPORT_RESORTS_ENDPOINT,
)

DATA_FILE_ENDPOINTS = {
    "lifts.geojson": IMPORT_LIFTS_ENDPOINT,
    "runs.geojson": IMPORT_SLOPES_ENDPOINT,
    "ski_areas.geojson": IMPORT_RESORTS_ENDPOINT,
}

DEFAULT_RETRY_WAIT = 1
MAX_RETRY_WAIT = 60
REQUEST_TIMEOUT = 30
CHUNK_SIZE = 65536

ROOT_DIR = Path(__file__).resolve().parents[2]
DATA_DIR = ROOT_DIR / "data" / "geodata"


def setup_logger() -> logging.Logger:
    logger = logging.getLogger("push_geojson_api")
    logger.setLevel(logging.INFO)
    handler = logging.StreamHandler()
    handler.setFormatter(
        logging.Formatter("%(asctime)s [%(levelname)s] %(message)s", "%Y-%m-%d %H:%M:%S")
    )
    logger.addHandler(handler)
    return logger


log = setup_logger()


def create_session() -> requests.Session:
    """Create a requests session with a lightweight retry adapter."""
    session = requests.Session()
    retry_strategy = Retry(
        total=HTTP_RETRIES,
        backoff_factor=1,
        status_forcelist=[429, 500, 502, 503, 504],
        allowed_methods=["HEAD", "GET", "OPTIONS", "POST", "PUT", "DELETE"],
    )
    adapter = HTTPAdapter(max_retries=retry_strategy)
    session.mount("http://", adapter)
    session.mount("https://", adapter)
    return session


SESSION = create_session()


def request_with_timeout_retry(
    method: str,
    url: str,
    json_body: Optional[Dict[str, Any]] = None,
    headers: Optional[Dict[str, str]] = None,
) -> Optional[requests.Response]:
    """Send a request and retry on 408 / timeout until it succeeds or a different error occurs."""
    wait = DEFAULT_RETRY_WAIT
    headers = headers or HEADERS

    while True:
        try:
            response = SESSION.request(
                method,
                url,
                json=json_body,
                headers=headers,
                timeout=REQUEST_TIMEOUT,
            )
        except Timeout as err:
            log.warning("Timeout for %s %s: %s. Retrying in %ss...", method, url, err, wait)
            time.sleep(wait)
            wait = min(wait * 2, MAX_RETRY_WAIT)
            continue
        except RequestException as err:
            log.error("Request failed for %s %s: %s", method, url, err)
            return None

        if response.status_code == 408:
            log.warning("HTTP 408 from %s %s. Retrying in %ss...", method, url, wait)
            time.sleep(wait)
            wait = min(wait * 2, MAX_RETRY_WAIT)
            continue

        if response.status_code in (429, 500, 502, 503, 504):
            log.warning(
                "HTTP %s from %s %s. Retrying in %ss...",
                response.status_code,
                method,
                url,
                wait,
            )
            time.sleep(wait)
            wait = min(wait * 2, MAX_RETRY_WAIT)
            continue

        return response


def validate_api() -> bool:
    """Validate that the OpenSlope API is reachable and accepts the configured key."""
    endpoints = {
        "resorts": RESORTS_ENDPOINT,
        "lifts": LIFTS_ENDPOINT,
        "slopes": SLOPES_ENDPOINT,
    }
    log.info("Validating OpenSlope API endpoints...")
    for name, endpoint in endpoints.items():
        response = request_with_timeout_retry("GET", endpoint, headers=HEADERS)
        if response is None:
            log.error("Failed to connect to %s endpoint", name)
            return False
        if response.status_code != 200:
            log.error(
                "Unexpected response from %s endpoint: %s %s",
                name,
                response.status_code,
                response.text.strip(),
            )
            return False
        log.info("Endpoint %s is reachable (HTTP %s)", name, response.status_code)
    return True


def get_default_resort_id(properties: Dict[str, Any]) -> str:
    """Infer a resort identifier from geojson properties if possible."""
    ski_areas = properties.get("skiAreas") or []
    if isinstance(ski_areas, list) and ski_areas:
        first = ski_areas[0]
        if isinstance(first, dict) and first.get("id"):
            return first["id"]
        if isinstance(first, str):
            return first

    places = properties.get("places") or []
    if isinstance(places, list) and places:
        first = places[0]
        if isinstance(first, dict):
            iso = first.get("iso3166_1Alpha2")
            if isinstance(iso, str):
                return iso

    return "unassigned"


def parse_coordinates(geometry: Dict[str, Any]) -> Optional[list[Dict[str, float]]]:
    if not geometry:
        return None
    coords = geometry.get("coordinates")
    if not isinstance(coords, list) or len(coords) == 0:
        return None

    def extract_point(point: Any) -> Optional[Dict[str, float]]:
        if not isinstance(point, list) or len(point) < 2:
            return None
        lon, lat = point[0], point[1]
        if isinstance(lat, (int, float)) and isinstance(lon, (int, float)):
            return {"latitude": lat, "longitude": lon}
        return None

    points = []
    for item in coords:
        if isinstance(item, list) and len(item) > 0 and isinstance(item[0], list):
            # nested arrays inside geometry
            nested = [extract_point(point) for point in item if extract_point(point)]
            points.extend([pt for pt in nested if pt])
        else:
            point = extract_point(item)
            if point:
                points.append(point)

    return points if points else None


def build_resort_payload(feature: Dict[str, Any]) -> Dict[str, Any]:
    properties = feature.get("properties", {}) or {}
    geometry = feature.get("geometry", {}) or {}
    coords = geometry.get("coordinates")
    latitude = None
    longitude = None
    if isinstance(coords, list) and len(coords) >= 2:
        longitude, latitude = coords[0], coords[1]

    places = properties.get("places") or []
    country = None
    region = None
    if isinstance(places, list) and places:
        first = places[0]
        if isinstance(first, dict):
            country = first.get("iso3166_1Alpha2")
            region = first.get("localized", {}).get("en", {}).get("region")

    return {
        "id": properties.get("id"),
        "name": properties.get("name") or properties.get("type") or "unnamed",
        "country": country or "unknown",
        "region": region,
        "continent": None,
        "latitude": latitude,
        "longitude": longitude,
        "ski_area_type": properties.get("type") or "skiArea",
        "official_website": None,
        "lift_status_url": None,
        "slope_status_url": None,
        "snow_report_url": None,
        "weather_url": None,
        "status_provider": properties.get("status"),
    }


def build_lift_payload(feature: Dict[str, Any]) -> Dict[str, Any]:
    properties = feature.get("properties", {}) or {}
    geometry = feature.get("geometry", {}) or {}
    points = parse_coordinates(geometry) or []
    start = points[0] if points else {"latitude": None, "longitude": None}
    end = points[-1] if len(points) > 1 else start

    def to_bool(value: Any) -> bool:
        return bool(value) and str(value).lower() not in ("false", "0", "none", "null")

    return {
        "resort_id": get_default_resort_id(properties),
        "name": properties.get("name"),
        "lift_type": properties.get("liftType") or properties.get("type"),
        "capacity_per_hour": properties.get("capacity"),
        "seats": properties.get("seats"),
        "duration_minutes": properties.get("duration"),
        "detachable": to_bool(properties.get("detachable")),
        "heating": to_bool(properties.get("heating")),
        "bubble": to_bool(properties.get("bubble")),
        "lat_start": start.get("latitude"),
        "lon_start": start.get("longitude"),
        "lat_end": end.get("latitude"),
        "lon_end": end.get("longitude"),
        "slope_path_json": json.dumps(points) if points else None,
        "source_system": "geojson",
        "source_entity_id": properties.get("id"),
        "status": properties.get("status"),
        "description": properties.get("description"),
    }


def build_slope_payload(feature: Dict[str, Any]) -> Dict[str, Any]:
    properties = feature.get("properties", {}) or {}
    geometry = feature.get("geometry", {}) or {}
    points = parse_coordinates(geometry) or []
    start = points[0] if points else {"latitude": None, "longitude": None}
    end = points[-1] if len(points) > 1 else start

    def to_bool(value: Any) -> bool:
        return bool(value) and str(value).lower() not in ("false", "0", "none", "null")

    return {
        "resort_id": get_default_resort_id(properties),
        "name": properties.get("name"),
        "difficulty": properties.get("difficulty"),
        "length_m": properties.get("length"),
        "average_gradient": properties.get("averageGradient"),
        "max_gradient": properties.get("maxGradient"),
        "snowmaking": to_bool(properties.get("snowmaking")),
        "lit": to_bool(properties.get("lit")),
        "patrolled": to_bool(properties.get("patrolled")),
        "difficulty_convention": properties.get("difficultyConvention"),
        "grooming_status": properties.get("grooming"),
        "status": properties.get("status"),
        "lat_start": start.get("latitude"),
        "lon_start": start.get("longitude"),
        "lat_end": end.get("latitude"),
        "lon_end": end.get("longitude"),
        "slope_path_json": json.dumps(points) if points else None,
        "source_system": "geojson",
        "source_entity_id": properties.get("id"),
        "description": properties.get("description"),
    }


def build_payload(file_type: str, feature: Dict[str, Any]) -> Dict[str, Any]:
    if file_type == "resorts":
        return build_resort_payload(feature)
    if file_type == "lifts":
        return build_lift_payload(feature)
    if file_type == "slopes":
        return build_slope_payload(feature)
    raise ValueError(f"Unsupported file type: {file_type}")


def feature_stream(file_path: Path) -> Iterator[Dict[str, Any]]:
    decoder = json.JSONDecoder()
    buffer = ""
    started = False
    file_type = file_path.name

    with file_path.open("r", encoding="utf-8") as source:
        while True:
            chunk = source.read(CHUNK_SIZE)
            if not chunk:
                break
            buffer += chunk
            if not started:
                features_index = buffer.find("\"features\"")
                if features_index < 0:
                    continue
                open_bracket = buffer.find("[", features_index)
                if open_bracket < 0:
                    continue
                buffer = buffer[open_bracket + 1 :]
                started = True

            while True:
                stripped = buffer.lstrip()
                if not stripped:
                    break
                if stripped[0] in ",\n\r \t":
                    buffer = stripped[1:]
                    continue
                if stripped[0] == "]":
                    return
                try:
                    obj, index = decoder.raw_decode(stripped)
                    yield obj
                    buffer = stripped[index:]
                except json.JSONDecodeError:
                    break

    # Final buffer flush after file read
    while True:
        stripped = buffer.lstrip()
        if not stripped or stripped[0] == "]":
            return
        try:
            obj, index = decoder.raw_decode(stripped)
            yield obj
            buffer = stripped[index:]
        except json.JSONDecodeError:
            return


def execute_push(
    file_path: Path,
    dry_run: bool = False,
    limit: Optional[int] = None,
) -> None:
    file_name = file_path.name
    endpoint = DATA_FILE_ENDPOINTS.get(file_name)
    if endpoint is None:
        log.error("Unsupported geojson file: %s", file_name)
        return

    resource_type = {
        "lifts.geojson": "lifts",
        "runs.geojson": "slopes",
        "ski_areas.geojson": "resorts",
    }[file_name]

    log.info("Pushing data from %s to %s", file_path, endpoint)
    pushed = 0
    skipped = 0

    for index, feature in enumerate(feature_stream(file_path), start=1):
        if limit and index > limit:
            break
        payload = build_payload(resource_type, feature)
        if dry_run:
            log.info("Dry run payload for %s #%s: %s", file_name, index, payload)
            skipped += 1
            continue

        response = request_with_timeout_retry("POST", endpoint, json_body=feature, headers=HEADERS)
        if response is None:
            log.error("No response for %s record %s", file_name, index)
            skipped += 1
            continue

        if response.status_code in (200, 201):
            pushed += 1
            log.info("PUSHED %s #%s (HTTP %s)", file_name, index, response.status_code)
        else:
            log.error(
                "Failed to push %s #%s: HTTP %s %s",
                file_name,
                index,
                response.status_code,
                response.text.strip(),
            )
            skipped += 1

        time.sleep(REQUEST_DELAY)

    log.info(
        "Finished %s: pushed=%s skipped=%s limit=%s",
        file_name,
        pushed,
        skipped,
        limit,
    )


def main() -> int:
    parser = argparse.ArgumentParser(description="Push geojson files to the OpenSlope API.")
    parser.add_argument(
        "--file",
        type=str,
        help="Path to a single geojson file to push (lifts.geojson, runs.geojson, ski_areas.geojson).",
    )
    parser.add_argument(
        "--all",
        action="store_true",
        help="Push all supported data files from data/geodata.",
    )
    parser.add_argument(
        "--validate",
        action="store_true",
        help="Validate API connectivity before pushing data.",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Build payloads and validate API connectivity without sending POST/PUT requests.",
    )
    parser.add_argument(
        "--limit",
        type=int,
        help="Limit the number of features pushed from each file.",
    )
    args = parser.parse_args()

    if args.validate:
        if not validate_api():
            log.error("API validation failed. Aborting.")
            return 1
        log.info("API validation completed successfully.")
        if not args.all and not args.file:
            return 0

    if not args.all and not args.file:
        parser.print_help(sys.stdout)
        return 1

    files_to_push = []
    if args.all:
        files_to_push = [
            DATA_DIR / "lifts.geojson",
            DATA_DIR / "runs.geojson",
            DATA_DIR / "ski_areas.geojson",
        ]
    else:
        file_path = Path(args.file)
        if not file_path.is_absolute():
            file_path = ROOT_DIR / file_path
        files_to_push = [file_path]

    for file_path in files_to_push:
        if not file_path.exists():
            log.error("GeoJSON file not found: %s", file_path)
            continue
        execute_push(file_path, dry_run=args.dry_run, limit=args.limit)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
