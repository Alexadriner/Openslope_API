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
from decimal import Decimal
from pathlib import Path
from typing import Any, Dict, Iterator, List, Optional, Tuple

import ijson
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
MAX_FEATURE_PUSH_ATTEMPTS = 6
TRANSIENT_STATUS_CODES = {408, 423, 425, 429, 500, 502, 503, 504}
TRANSIENT_ERROR_HINTS = (
    "too many connections",
    "deadlock",
    "lock wait timeout",
    "temporarily unavailable",
    "timeout",
    "timed out",
    "connection reset",
    "connection refused",
    "try again",
    "rate limit",
    "overloaded",
    "busy",
)

ROOT_DIR = Path(__file__).resolve().parents[2]
DATA_DIR = ROOT_DIR / "data" / "geodata"
CHECKPOINTS_DIR = ROOT_DIR / "checkpoints" / "push_geojson_api"


def get_checkpoint_file(file_path: Path) -> Path:
    """Get the checkpoint file path for a given GeoJSON file."""
    filename = file_path.stem
    CHECKPOINTS_DIR.mkdir(parents=True, exist_ok=True)
    return CHECKPOINTS_DIR / f"{filename}_checkpoint.json"


def load_checkpoint(file_path: Path) -> int:
    """Load the last processed index from checkpoint file."""
    checkpoint_file = get_checkpoint_file(file_path)
    if checkpoint_file.exists():
        try:
            with open(checkpoint_file, 'r') as f:
                data = json.load(f)
                return data.get("last_index", 0)
        except Exception as e:
            log.warning("Failed to load checkpoint from %s: %s", checkpoint_file, e)
    return 0


def save_checkpoint(file_path: Path, last_index: int) -> None:
    """Save the last processed index to checkpoint file."""
    checkpoint_file = get_checkpoint_file(file_path)
    try:
        with open(checkpoint_file, 'w') as f:
            json.dump({"last_index": last_index}, f)
        log.info("Checkpoint saved: last_index=%d for %s", last_index, file_path.name)
    except Exception as e:
        log.error("Failed to save checkpoint to %s: %s", checkpoint_file, e)


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


def set_log_level(level: int) -> None:
    """Set the log level for all handlers."""
    for handler in log.handlers:
        handler.setLevel(level)
    log.setLevel(level)


def convert_decimals(obj: Any) -> Any:
    """Recursively convert Decimal objects to floats for JSON serialization."""
    if isinstance(obj, Decimal):
        return float(obj)
    elif isinstance(obj, dict):
        return {k: convert_decimals(v) for k, v in obj.items()}
    elif isinstance(obj, list):
        return [convert_decimals(item) for item in obj]
    else:
        return obj


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
            response_text = response.text[:400].strip()
            log.warning("HTTP 408 from %s %s. Retrying in %ss...", method, url, wait)
            if response_text:
                log.warning("Response body from %s %s: %s", method, url, response_text)
            time.sleep(wait)
            wait = min(wait * 2, MAX_RETRY_WAIT)
            continue

        if response.status_code in (429, 500, 502, 503, 504):
            response_text = response.text[:400].strip()
            log.warning(
                "HTTP %s from %s %s. Retrying in %ss...",
                response.status_code,
                method,
                url,
                wait,
            )
            if response_text:
                log.warning("Response body from %s %s: %s", method, url, response_text)
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


def validate_geometry(geometry: Dict[str, Any]) -> Tuple[bool, Optional[str]]:
    """Validate geometry for invalid values (NaN, Infinity, etc)."""
    def check_coord(coord: Any) -> Optional[str]:
        if isinstance(coord, (int, float)):
            if coord != coord:  # NaN check
                return "NaN coordinate found"
            if coord == float('inf') or coord == float('-inf'):
                return "Infinity coordinate found"
        return None

    def check_coords_recursive(coords: Any) -> Optional[str]:
        if isinstance(coords, (list, tuple)):
            if len(coords) > 0 and isinstance(coords[0], (int, float)):
                # This is a coordinate pair [lon, lat] or [lon, lat, elevation]
                for coord in coords:
                    error = check_coord(coord)
                    if error:
                        return error
            else:
                # This is a nested structure
                for item in coords:
                    error = check_coords_recursive(item)
                    if error:
                        return error
        return None

    coords = geometry.get("coordinates", [])
    return (True, None) if not check_coords_recursive(coords) else (False, check_coords_recursive(coords))


def prepare_feature_for_api(feature: Dict[str, Any]) -> Optional[Tuple[str, str, Dict[str, Any]]]:
    """
    Prepare a GeoJSON feature for API submission.
    Returns (endpoint, resource_type, feature_dict) or None if feature is invalid.
    """
    properties = feature.get("properties", {})
    feature_type = detect_feature_type(properties)

    # Validate feature has required geometry
    geometry = feature.get("geometry")
    if not geometry or not geometry.get("type") or geometry.get("type") not in (
        "Point", "LineString", "Polygon", "MultiPoint", "MultiLineString", "MultiPolygon"
    ):
        return None  # Invalid geometry, skip this feature

    # Validate geometry values for NaN, Infinity, etc
    is_valid, error_msg = validate_geometry(geometry)
    if not is_valid:
        feature_id = feature.get("properties", {}).get("id", "unknown")
        feature_name = feature.get("properties", {}).get("name", "unknown")
        log.error(
            "Invalid geometry for feature '%s' (%s): %s",
            feature_name,
            feature_id,
            error_msg,
        )
        return None

    # Generate normalized name if needed
    normalized_name = generate_normalized_name(properties, feature_type)

    # Validate that normalized name is not empty
    if not normalized_name or not str(normalized_name).strip():
        return None  # Could not generate valid name, skip this feature

    # Create a copy of the feature and update the name
    prepared_feature = feature.copy()
    prepared_properties = properties.copy()
    prepared_properties["name"] = normalized_name
    prepared_feature["properties"] = prepared_properties

    # Convert any Decimal objects to floats for JSON serialization
    prepared_feature = convert_decimals(prepared_feature)

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


def extract_feature_id(feature: Dict[str, Any]) -> Optional[str]:
    properties = feature.get("properties", {})

    candidate = properties.get("id", feature.get("id"))
    if candidate is None:
        return None

    candidate_str = str(candidate).strip()
    return candidate_str or None


def get_update_endpoint(resource_type: str, feature_id: str) -> str:
    if resource_type == "resorts":
        return f"{RESORTS_ENDPOINT}/{feature_id}"
    if resource_type == "lifts":
        return f"{LIFTS_ENDPOINT}/{feature_id}"
    if resource_type == "slopes":
        return f"{SLOPES_ENDPOINT}/{feature_id}"
    raise ValueError(f"Unsupported resource type for update: {resource_type}")


def get_feature_label(resource_type: str, feature: Dict[str, Any]) -> str:
    properties = feature.get("properties", {})
    feature_id = extract_feature_id(feature) or "unknown-id"
    feature_name = properties.get("name")

    if feature_name:
        return f"{resource_type} '{feature_name}' ({feature_id})"
    return f"{resource_type} {feature_id}"


def compute_feature_retry_wait(attempt: int) -> int:
    return min(DEFAULT_RETRY_WAIT * (2 ** max(0, attempt - 1)), MAX_RETRY_WAIT)


def response_indicates_transient_failure(response: Optional[requests.Response]) -> bool:
    if response is None:
        return True

    if response.status_code in TRANSIENT_STATUS_CODES:
        return True

    body = response.text.lower()
    return any(hint in body for hint in TRANSIENT_ERROR_HINTS)


def try_upsert_feature(
    endpoint: str,
    resource_type: str,
    feature: Dict[str, Any],
) -> Tuple[bool, bool]:
    response = request_with_timeout_retry("POST", endpoint, json_body=feature, headers=HEADERS)

    if response is None:
        return False, True

    if response.status_code in (200, 201):
        log.info("SUCCESS: %s feature (HTTP %s)", resource_type, response.status_code)
        return True, False

    if response.status_code == 400:
        # Bad request - likely a data validation issue
        feature_id = extract_feature_id(feature)
        feature_name = feature.get("properties", {}).get("name", "unknown")
        log.error(
            "BAD REQUEST (400) for %s feature '%s' (%s): %s",
            resource_type,
            feature_name,
            feature_id,
            response.text[:500],
        )
        # Log the feature for debugging
        log.debug("Feature data: %s", json.dumps(feature, indent=2))
        return False, False

    if response.status_code == 409:
        feature_id = extract_feature_id(feature)
        if not feature_id:
            log.error(
                "FAILED: %s feature conflict, but no feature id was present",
                resource_type,
            )
            return False, False

        update_endpoint = get_update_endpoint(resource_type, feature_id)
        update_response = request_with_timeout_retry(
            "PUT",
            update_endpoint,
            json_body=feature,
            headers=HEADERS,
        )

        if update_response is None:
            return False, True

        if update_response.status_code in (200, 201):
            log.info(
                "UPDATED: %s feature %s after conflict (HTTP %s)",
                resource_type,
                feature_id,
                update_response.status_code,
            )
            return True, False

        if response_indicates_transient_failure(update_response):
            log.warning(
                "Transient update failure for %s - HTTP %s: %s",
                get_feature_label(resource_type, feature),
                update_response.status_code,
                update_response.text[:200],
            )
            return False, True

        log.error(
            "FAILED UPDATE: %s feature %s - HTTP %s: %s",
            resource_type,
            feature_id,
            update_response.status_code,
            update_response.text[:200],
        )
        return False, False

    if response_indicates_transient_failure(response):
        log.warning(
            "Transient push failure for %s - HTTP %s: %s",
            get_feature_label(resource_type, feature),
            response.status_code,
            response.text[:200],
        )
        return False, True

    # For 500+ errors, log the full response
    if response.status_code >= 500:
        feature_id = extract_feature_id(feature)
        feature_name = feature.get("properties", {}).get("name", "unknown")
        log.error(
            "SERVER ERROR (%s) for %s feature '%s' (%s): %s",
            response.status_code,
            resource_type,
            feature_name,
            feature_id,
            response.text[:1000],
        )

    log.error(
        "FAILED: %s feature - HTTP %s: %s",
        resource_type,
        response.status_code,
        response.text[:200],
    )
    return False, False


def push_feature_with_recovery(
    endpoint: str,
    resource_type: str,
    feature: Dict[str, Any],
) -> bool:
    feature_label = get_feature_label(resource_type, feature)

    for attempt in range(1, MAX_FEATURE_PUSH_ATTEMPTS + 1):
        success, should_retry = try_upsert_feature(endpoint, resource_type, feature)
        if success:
            return True

        if not should_retry:
            return False

        if attempt == MAX_FEATURE_PUSH_ATTEMPTS:
            break

        wait = compute_feature_retry_wait(attempt)
        log.warning(
            "Retrying %s in %ss (attempt %s/%s)...",
            feature_label,
            wait,
            attempt + 1,
            MAX_FEATURE_PUSH_ATTEMPTS,
        )
        time.sleep(wait)

    log.error(
        "Gave up on %s after %s attempts.",
        feature_label,
        MAX_FEATURE_PUSH_ATTEMPTS,
    )
    return False


def load_geojson_features(file_path: Path) -> Iterator[Dict[str, Any]]:
    """Load features from a GeoJSON file (FeatureCollection or Feature) iteratively."""
    try:
        with open(file_path, 'rb') as f:  # ijson needs binary mode
            # Parse the features array
            features = ijson.items(f, 'features.item')
            for feature in features:
                if feature.get("type") == "Feature":
                    yield feature

    except Exception as e:
        log.error("Failed to load GeoJSON from %s: %s", file_path, e)


def execute_push(
    file_path: Path,
    dry_run: bool = False,
    limit: Optional[int] = None,
    start_index: int = 1,
    batch_size: int = 10,
    debug_mode: bool = False,
) -> None:
    """Push GeoJSON features to the API with batching and type detection."""
    log.info("Processing GeoJSON file: %s", file_path)

    # Load checkpoint
    checkpoint_index = load_checkpoint(file_path)
    if start_index == 1 and checkpoint_index > 0:
        log.info("Resuming from checkpoint: last processed index %d", checkpoint_index)
        start_index = checkpoint_index + 1

    log.info("Starting at feature index %d", start_index)

    # Group features by endpoint for batching
    batches: Dict[str, List[Tuple[str, Dict[str, Any]]]] = {}
    skipped_count = 0
    processed_count = 0
    max_features = 10000  # Process only 10000 features per run
    max_processed_index = start_index - 1

    for index, feature in enumerate(load_geojson_features(file_path), start=1):
        if index < start_index:
            continue

        if processed_count >= max_features:
            log.info("Reached maximum features per run (%d). Stopping.", max_features)
            break

        if limit and processed_count >= limit:
            break

        max_processed_index = index

        try:
            result = prepare_feature_for_api(feature)
            if result is None:
                # Feature is invalid or cannot be prepared, skip it
                feature_id = feature.get("properties", {}).get("id", "unknown")
                log.debug("Skipped invalid feature #%s (%s)", index, feature_id)
                skipped_count += 1
                continue

            endpoint, resource_type, prepared_feature = result
            if debug_mode:
                feature_id = extract_feature_id(prepared_feature)
                feature_name = prepared_feature.get("properties", {}).get("name", "unknown")
                geometry_type = prepared_feature.get("geometry", {}).get("type", "unknown")
                log.debug(
                    "[#%d] %s feature '%s' (%s) - geometry: %s",
                    index,
                    resource_type,
                    feature_name,
                    feature_id,
                    geometry_type,
                )
            if endpoint not in batches:
                batches[endpoint] = []
            batches[endpoint].append((resource_type, prepared_feature))

            # Process batches when they reach the batch size
            if len(batches[endpoint]) >= batch_size:
                if process_batch(endpoint, batches[endpoint], dry_run):
                    save_checkpoint(file_path, max_processed_index)
                batches[endpoint] = []

            processed_count += 1

        except Exception as e:
            feature_id = feature.get("properties", {}).get("id", "unknown")
            log.error("Failed to process feature #%s (%s): %s", index, feature_id, e)
            continue

    # Process remaining batches
    for endpoint, batch in batches.items():
        if batch:
            if process_batch(endpoint, batch, dry_run):
                save_checkpoint(file_path, max_processed_index)

    if skipped_count > 0:
        log.info("Skipped %d features due to invalid data", skipped_count)

    log.info("Processed %d features. Next run will start from index %d", processed_count, max_processed_index + 1)


def process_batch(endpoint: str, batch: List[Tuple[str, Dict[str, Any]]], dry_run: bool) -> bool:
    """Process a batch of features for a single endpoint."""
    if dry_run:
        for resource_type, feature in batch:
            name = feature.get("properties", {}).get("name", "unnamed")
            log.info("DRY RUN: Would send %s feature '%s' to %s", resource_type, name, endpoint)
        return True

    log.info("Sending batch of %d features to %s", len(batch), endpoint)

    success_count = 0
    for resource_type, feature in batch:
        if push_feature_with_recovery(endpoint, resource_type, feature):
            success_count += 1
        time.sleep(REQUEST_DELAY)

    return success_count == len(batch)


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
        "--index",
        type=int,
        default=1,
        help="Start processing at this 1-based feature index (default: 1).",
    )
    parser.add_argument(
        "--batch-size",
        type=int,
        default=10,
        help="Number of features to batch together per API request (default: 10).",
    )
    parser.add_argument(
        "--debug",
        action="store_true",
        help="Enable debug mode with detailed feature logging.",
    )
    args = parser.parse_args()

    if args.debug:
        set_log_level(logging.DEBUG)
        log.debug("Debug mode enabled")

    if args.index < 1:
        parser.error("--index must be >= 1")

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
        execute_push(
            file_path,
            dry_run=args.dry_run,
            limit=args.limit,
            start_index=args.index,
            batch_size=args.batch_size,
            debug_mode=args.debug,
        )

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
