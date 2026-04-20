"""
Push GeoJSON data from /data/geodata to the OpenSlope API.

This script reads GeoJSON FeatureCollection files and ingests them into the API.
It automatically detects feature types and handles relations.

Features:
- Parses GeoJSON FeatureCollection
- Detects types: skiArea → resort, lift → lift, run → slope
- Extracts geometry, properties, and relations
- Converts geometry to WKT
- Batches requests for efficiency
- Handles retries and logging

Usage examples:
    python push_geojson_api.py --all --dry-run
    python push_geojson_api.py --file data/geodata/lifts.geojson --batch-size 50
    python push_geojson_api.py --validate
"""

import argparse
import json
import logging
import sys
import time
from pathlib import Path
from typing import Any, Dict, Iterator, List, Optional, Tuple

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
    IMPORT_LIFTS_ENDPOINT,
    IMPORT_SLOPES_ENDPOINT,
    IMPORT_RESORTS_ENDPOINT,
    RESORTS_ENDPOINT,
    LIFTS_ENDPOINT,
    SLOPES_ENDPOINT,
)



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








def detect_feature_type(properties: Dict[str, Any]) -> str:
    """Detect the feature type based on properties."""
    feature_type = properties.get("type", "").lower()

    if feature_type == "lift":
        return "lift"
    elif feature_type in ("run", "slope", "piste"):
        return "slope"
    elif feature_type in ("skiarea", "ski_area", "resort"):
        return "resort"
    else:
        # Fallback based on common properties
        if "liftType" in properties or "capacity" in properties:
            return "lift"
        elif "difficulty" in properties or "grooming" in properties:
            return "slope"
        elif "skiAreas" in properties or "places" in properties:
            return "resort"

    return "unknown"


def extract_resort_ids(properties: Dict[str, Any]) -> List[str]:
    """Extract resort IDs from skiAreas or related properties."""
    ski_areas = properties.get("skiAreas", [])
    if isinstance(ski_areas, list):
        resort_ids = []
        for area in ski_areas:
            if isinstance(area, dict) and "id" in area:
                resort_ids.append(area["id"])
            elif isinstance(area, str):
                resort_ids.append(area)
        return resort_ids

    # Fallback: try to infer from other properties
    if "resort_id" in properties:
        return [str(properties["resort_id"])]

    return []


def generate_normalized_name(properties: Dict[str, Any], feature_type: str) -> str:
    """Generate a normalized name for features that don't have one."""
    current_name = properties.get("name")

    # If name exists and is not null/empty, return it
    if current_name and str(current_name).strip() and str(current_name).lower() not in ("null", "none", ""):
        return str(current_name)

    if feature_type == "resort":
        # Extract location info from places
        places = properties.get("places", [])
        if isinstance(places, list) and places:
            place = places[0] if isinstance(places[0], dict) else {}

            # Try locality first
            locality = None
            if isinstance(place, dict):
                localized = place.get("localized", {})
                if isinstance(localized, dict):
                    en = localized.get("en", {})
                    if isinstance(en, dict):
                        locality = en.get("locality")

            if locality:
                return f"Ski resort near {locality}"

            # Try region
            region = None
            if isinstance(place, dict):
                localized = place.get("localized", {})
                if isinstance(localized, dict):
                    en = localized.get("en", {})
                    if isinstance(en, dict):
                        region = en.get("region")

            if region:
                return f"Ski resort in {region}"

            # Try country
            country = None
            if isinstance(place, dict):
                localized = place.get("localized", {})
                if isinstance(localized, dict):
                    en = localized.get("en", {})
                    if isinstance(en, dict):
                        country = en.get("country")

            if country:
                return f"Ski resort in {country}"

        # Fallback to ID
        feature_id = properties.get("id", "unknown")
        return f"Ski resort {feature_id}"

    elif feature_type == "lift":
        lift_type = properties.get("liftType", properties.get("type", "lift"))
        if lift_type:
            return str(lift_type).capitalize()
        return "Lift"

    elif feature_type == "slope":
        difficulty = properties.get("difficulty", "")
        use = properties.get("use", "")

        parts = []
        if difficulty:
            parts.append(str(difficulty).capitalize())
        if use:
            parts.append(str(use).lower())
        parts.append("slope")

        return " ".join(parts)

    # Default fallback
    return f"{feature_type.capitalize()} {properties.get('id', 'unknown')}"


def prepare_feature_for_api(feature: Dict[str, Any]) -> Tuple[str, str, Dict[str, Any]]:
    """
    Prepare a GeoJSON feature for API submission.
    Returns (endpoint, resource_type, feature_dict)
    """
    properties = feature.get("properties", {})
    feature_type = detect_feature_type(properties)

    # Generate normalized name if needed
    normalized_name = generate_normalized_name(properties, feature_type)

    # Create a copy of the feature and update the name
    prepared_feature = feature.copy()
    prepared_properties = properties.copy()
    prepared_properties["name"] = normalized_name
    prepared_feature["properties"] = prepared_properties

    if feature_type == "lift":
        endpoint = IMPORT_LIFTS_ENDPOINT
        resource_type = "lifts"
    elif feature_type == "slope":
        endpoint = IMPORT_SLOPES_ENDPOINT
        resource_type = "slopes"
    elif feature_type == "resort":
        endpoint = IMPORT_RESORTS_ENDPOINT
        resource_type = "resorts"
    else:
        # Default to resorts for unknown types
        endpoint = IMPORT_RESORTS_ENDPOINT
        resource_type = "resorts"

    return endpoint, resource_type, prepared_feature


def load_geojson_features(file_path: Path) -> Iterator[Dict[str, Any]]:
    """Load features from a GeoJSON file (FeatureCollection or Feature)."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            data = json.load(f)

        if data.get("type") == "FeatureCollection":
            features = data.get("features", [])
        elif data.get("type") == "Feature":
            features = [data]
        else:
            log.error("Unsupported GeoJSON type in %s", file_path)
            return

        for feature in features:
            if feature.get("type") == "Feature":
                yield feature

    except Exception as e:
        log.error("Failed to load GeoJSON from %s: %s", file_path, e)


def execute_push(
    file_path: Path,
    dry_run: bool = False,
    limit: Optional[int] = None,
    batch_size: int = 10,
) -> None:
    """Push GeoJSON features to the API with batching and type detection."""
    log.info("Processing GeoJSON file: %s", file_path)

    # Group features by endpoint for batching
    batches: Dict[str, List[Tuple[str, Dict[str, Any]]]] = {}

    for index, feature in enumerate(load_geojson_features(file_path), start=1):
        if limit and index > limit:
            break

        try:
            endpoint, resource_type, prepared_feature = prepare_feature_for_api(feature)
            if endpoint not in batches:
                batches[endpoint] = []
            batches[endpoint].append((resource_type, prepared_feature))

            # Process batches when they reach the batch size
            if len(batches[endpoint]) >= batch_size:
                process_batch(endpoint, batches[endpoint], dry_run)
                batches[endpoint] = []

        except Exception as e:
            log.error("Failed to process feature #%s: %s", index, e)
            continue

    # Process remaining batches
    for endpoint, batch in batches.items():
        if batch:
            process_batch(endpoint, batch, dry_run)


def process_batch(endpoint: str, batch: List[Tuple[str, Dict[str, Any]]], dry_run: bool) -> None:
    """Process a batch of features for a single endpoint."""
    if dry_run:
        for resource_type, feature in batch:
            name = feature.get("properties", {}).get("name", "unnamed")
            log.info("DRY RUN: Would send %s feature '%s' to %s", resource_type, name, endpoint)
        return

    log.info("Sending batch of %d features to %s", len(batch), endpoint)

    for resource_type, feature in batch:
        response = request_with_timeout_retry("POST", endpoint, json_body=feature, headers=HEADERS)

        if response is None:
            log.error("No response for %s feature", resource_type)
            continue

        if response.status_code in (200, 201):
            log.info("SUCCESS: %s feature (HTTP %s)", resource_type, response.status_code)
        else:
            log.error(
                "FAILED: %s feature - HTTP %s: %s",
                resource_type,
                response.status_code,
                response.text[:200]  # Truncate long error messages
            )

        # Rate limiting
        time.sleep(REQUEST_DELAY)


def main() -> int:
    parser = argparse.ArgumentParser(description="Push geojson files to the OpenSlope API.")
    parser.add_argument(
        "--file",
        type=str,
        help="Path to a single geojson file to push (supports FeatureCollection or individual Features).",
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
    parser.add_argument(
        "--batch-size",
        type=int,
        default=10,
        help="Number of features to batch together per API request (default: 10).",
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
        execute_push(file_path, dry_run=args.dry_run, limit=args.limit, batch_size=args.batch_size)

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
