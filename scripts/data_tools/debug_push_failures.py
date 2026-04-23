#!/usr/bin/env python3
"""
Debug script to identify which features fail when pushing to the API.
Tests features individually to isolate problematic ones.
"""

import json
import logging
from pathlib import Path
from typing import Any, Dict, List, Tuple
import requests
from config import API_BASE_URL, HEADERS, IMPORT_RESORTS_ENDPOINT

# Setup logging
logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s [%(levelname)s] %(message)s",
    datefmt="%Y-%m-%d %H:%M:%S"
)
log = logging.getLogger("debug_push")

ROOT_DIR = Path(__file__).resolve().parents[2]
DATA_DIR = ROOT_DIR / "data" / "geodata"


def analyze_feature_properties(properties: Dict[str, Any]) -> Dict[str, Any]:
    """Analyze feature properties for potential issues."""
    issues = []
    
    # Check name
    name = properties.get("name")
    if not name:
        issues.append("Missing name")
    elif isinstance(name, str):
        # Check for non-ASCII characters
        try:
            name.encode('ascii')
        except UnicodeEncodeError:
            issues.append(f"Non-ASCII name: {name}")
        
        # Check length
        if len(name) > 255:
            issues.append(f"Name too long: {len(name)} chars")
    
    # Check places structure
    places = properties.get("places", [])
    if not isinstance(places, list):
        issues.append(f"places is not a list: {type(places)}")
    elif places:
        for i, place in enumerate(places):
            if not isinstance(place, dict):
                issues.append(f"places[{i}] is not a dict: {type(place)}")
            else:
                # Check required fields
                if "iso3166_1Alpha2" not in place:
                    issues.append(f"places[{i}] missing iso3166_1Alpha2")
                if "localized" not in place:
                    issues.append(f"places[{i}] missing localized")
    
    # Check for null/None values that might cause issues
    for key, value in properties.items():
        if value is None:
            continue
        if isinstance(value, str) and not value.strip():
            issues.append(f"Empty string in {key}")
        elif isinstance(value, list) and len(value) == 0:
            pass  # Empty lists are usually okay
    
    return {
        "has_issues": len(issues) > 0,
        "issues": issues,
        "properties_count": len(properties),
    }


def prepare_feature_for_api(feature: Dict[str, Any]) -> Dict[str, Any]:
    """Prepare feature for API (same as in push_geojson_api.py)"""
    # For resorts, we just return the feature as-is for testing
    return feature


def test_feature_push(feature: Dict[str, Any]) -> Tuple[bool, str]:
    """Test pushing a single feature to the API."""
    properties = feature.get("properties", {})
    feature_id = properties.get("id", "unknown")
    feature_name = properties.get("name", "unknown")
    
    try:
        response = requests.post(
            IMPORT_RESORTS_ENDPOINT,
            json=feature,
            headers=HEADERS,
            timeout=10
        )
        
        if response.status_code in (200, 201):
            return True, f"Success (HTTP {response.status_code})"
        elif response.status_code == 409:
            return True, f"Conflict (HTTP 409) - would UPDATE"
        else:
            return False, f"HTTP {response.status_code}: {response.text[:200]}"
    
    except Exception as e:
        return False, str(e)


def main():
    file_path = DATA_DIR / "ski_areas.geojson"
    
    if not file_path.exists():
        log.error("File not found: %s", file_path)
        return 1
    
    log.info("Loading GeoJSON file: %s", file_path)
    
    with open(file_path, 'r', encoding='utf-8') as f:
        data = json.load(f)
    
    features = data.get("features", [])
    log.info("Found %d features", len(features))
    
    # Analyze features
    problematic_features = []
    successful_features = []
    
    for idx, feature in enumerate(features[:20], 1):  # Test first 20
        properties = feature.get("properties", {})
        feature_id = properties.get("id", "unknown")
        feature_name = properties.get("name", "unknown")
        
        analysis = analyze_feature_properties(properties)
        
        if analysis["has_issues"]:
            problematic_features.append({
                "index": idx,
                "id": feature_id,
                "name": feature_name,
                "issues": analysis["issues"]
            })
            log.warning("Feature #%d '%s' (%s) has issues: %s", 
                       idx, feature_name, feature_id, analysis["issues"])
        
        # Test the push
        log.info("Testing feature #%d: %s", idx, feature_name)
        success, message = test_feature_push(feature)
        
        if success:
            successful_features.append({
                "index": idx,
                "id": feature_id,
                "name": feature_name,
                "message": message
            })
            log.info("  ✓ %s", message)
        else:
            problematic_features.append({
                "index": idx,
                "id": feature_id,
                "name": feature_name,
                "push_error": message,
                "issues": analysis.get("issues", [])
            })
            log.error("  ✗ %s", message)
    
    # Summary
    log.info("\n" + "="*60)
    log.info("SUMMARY")
    log.info("="*60)
    log.info("Successful: %d", len(successful_features))
    log.info("Failed: %d", len(problematic_features))
    
    if problematic_features:
        log.info("\nProblematic features:")
        for item in problematic_features:
            log.info("  #%d: %s (%s)", item["index"], item["name"], item["id"])
            if "issues" in item:
                for issue in item["issues"]:
                    log.info("      - %s", issue)
            if "push_error" in item:
                log.info("      - Push error: %s", item["push_error"])
    
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
