#!/usr/bin/env python3
"""
Bergfex API Collector for Sonnenbühl-Genkingen Ski Resort

This script collects data from the Bergfex API for the Sonnenbühl-Genkingen ski resort
and stores it in the SkiAPI database. It uses only GET requests as required.

Usage:
    python bergfex_collector.py [--resort-id RESORT_ID] [--api-key API_KEY]
"""

import argparse
import asyncio
import json
import logging
import os
import sys
import time
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Any

import aiohttp
import requests

# =========================
# PATHS
# =========================
ROOT_DIR = Path(__file__).resolve().parents[3]
LOG_DIR = ROOT_DIR / "logs" / "api_collector"
CHECKPOINTS_DIR = ROOT_DIR / "checkpoints" / "bergfex"

# =========================
# CONFIG
# =========================
BERGFEX_API_BASE = "https://www.bergfex.at/api/v1"
SONNENBUEHL_GENKINGEN_ID = "sonnenbuehl-genkingen"  # Default resort ID
API_KEY_FILE = ROOT_DIR / "api_key.env"

# API endpoints from the documentation
RESORT_ENDPOINT = "/skiresorts/{resort_id}"
LIFTS_ENDPOINT = "/skiresorts/{resort_id}/lifts"
SLOPES_ENDPOINT = "/skiresorts/{resort_id}/slopes"
WEATHER_ENDPOINT = "/skiresorts/{resort_id}/weather"
SNOW_ENDPOINT = "/skiresorts/{resort_id}/snow"

# SkiAPI configuration
SKI_API_BASE_URL = "http://localhost:8080"
SKI_API_KEY = None

HEADERS = {
    "Content-Type": "application/json",
    "User-Agent": "BergfexCollector/1.0"
}

# =========================
# ENUM MAPPINGS
# =========================
# Lift type mapping from Bergfex to SkiAPI
BERGFEX_LIFT_TYPE_MAP = {
    "gondola": "gondola",
    "cable_car": "cable_car", 
    "chair_lift": "chairlift",
    "mixed_lift": "chairlift",
    "t-bar": "draglift",
    "j-bar": "draglift", 
    "platter": "draglift",
    "rope_tow": "draglift",
    "magic_carpet": "magic_carpet",
    "surface_lift": "draglift",
    "ski_lift": "chairlift"
}

# Slope difficulty mapping from Bergfex to SkiAPI
BERGFEX_SLOPE_DIFFICULTY_MAP = {
    "novice": "green",
    "easy": "blue", 
    "intermediate": "red",
    "advanced": "black",
    "expert": "black",
    "beginner": "green",
    "family": "blue"
}

# Status mapping
BERGFEX_STATUS_MAP = {
    "open": "open",
    "closed": "closed", 
    "maintenance": "maintenance",
    "unknown": "unknown"
}

# =========================
# UTILS
# =========================
def setup_logging():
    """Setup logging configuration"""
    os.makedirs(LOG_DIR, exist_ok=True)
    log_filename = datetime.now().strftime("bergfex_collector_%Y-%m-%d_%H-%M-%S.log")
    log_path = LOG_DIR / log_filename

    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s | %(levelname)s | %(message)s",
        handlers=[
            logging.FileHandler(str(log_path), encoding="utf-8"),
            logging.StreamHandler(sys.stdout),
        ],
    )
    return logging.getLogger("bergfex_collector")

def load_api_key():
    """Load API key from environment or file"""
    global SKI_API_KEY
    
    # Try environment variable first
    SKI_API_KEY = os.getenv("SKI_API_KEY")
    
    if not SKI_API_KEY and API_KEY_FILE.exists():
        with open(API_KEY_FILE, 'r') as f:
            SKI_API_KEY = f.read().strip()
    
    if not SKI_API_KEY:
        logger.error("No API key found. Please set SKI_API_KEY environment variable or create api_key.env file")
        sys.exit(1)
    
    HEADERS["Authorization"] = f"Bearer {SKI_API_KEY}"

def normalize_name(name: Optional[str]) -> Optional[str]:
    """Normalize name for consistency"""
    if not name:
        return None
    return name.strip()

def get_skiapi_headers():
    """Get headers for SkiAPI requests"""
    return {
        "Content-Type": "application/json",
        "Authorization": f"Bearer {SKI_API_KEY}"
    }

# =========================
# BERGFEX API CLIENT
# =========================
class BergfexApiClient:
    """Client for Bergfex API"""
    
    def __init__(self, resort_id: str):
        self.resort_id = resort_id
        self.base_url = BERGFEX_API_BASE
        self.session = None
    
    async def __aenter__(self):
        self.session = aiohttp.ClientSession(headers=HEADERS)
        return self
    
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self.session:
            await self.session.close()
    
    async def get_resort_info(self) -> Optional[Dict[str, Any]]:
        """Get resort information"""
        url = f"{self.base_url}{RESORT_ENDPOINT.format(resort_id=self.resort_id)}"
        try:
            async with self.session.get(url) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    logger.warning(f"Failed to get resort info: {response.status}")
                    return None
        except Exception as e:
            logger.error(f"Error getting resort info: {e}")
            return None
    
    async def get_lifts(self) -> List[Dict[str, Any]]:
        """Get lift information"""
        url = f"{self.base_url}{LIFTS_ENDPOINT.format(resort_id=self.resort_id)}"
        try:
            async with self.session.get(url) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    logger.warning(f"Failed to get lifts: {response.status}")
                    return []
        except Exception as e:
            logger.error(f"Error getting lifts: {e}")
            return []
    
    async def get_slopes(self) -> List[Dict[str, Any]]:
        """Get slope information"""
        url = f"{self.base_url}{SLOPES_ENDPOINT.format(resort_id=self.resort_id)}"
        try:
            async with self.session.get(url) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    logger.warning(f"Failed to get slopes: {response.status}")
                    return []
        except Exception as e:
            logger.error(f"Error getting slopes: {e}")
            return []
    
    async def get_weather(self) -> Optional[Dict[str, Any]]:
        """Get weather information"""
        url = f"{self.base_url}{WEATHER_ENDPOINT.format(resort_id=self.resort_id)}"
        try:
            async with self.session.get(url) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    logger.warning(f"Failed to get weather: {response.status}")
                    return None
        except Exception as e:
            logger.error(f"Error getting weather: {e}")
            return None
    
    async def get_snow_info(self) -> Optional[Dict[str, Any]]:
        """Get snow information"""
        url = f"{self.base_url}{SNOW_ENDPOINT.format(resort_id=self.resort_id)}"
        try:
            async with self.session.get(url) as response:
                if response.status == 200:
                    return await response.json()
                else:
                    logger.warning(f"Failed to get snow info: {response.status}")
                    return None
        except Exception as e:
            logger.error(f"Error getting snow info: {e}")
            return None

# =========================
# SKI API CLIENT
# =========================
class SkiApiClient:
    """Client for SkiAPI"""
    
    def __init__(self):
        self.base_url = SKI_API_BASE_URL
    
    def create_or_update_resort(self, resort_data: Dict[str, Any]) -> bool:
        """Create or update resort in SkiAPI"""
        try:
            # Try to update first
            response = requests.put(
                f"{self.base_url}/resorts/{resort_data['id']}",
                json=resort_data,
                headers=get_skiapi_headers(),
                timeout=30
            )
            
            if response.status_code == 404:
                # Resort doesn't exist, create it
                response = requests.post(
                    f"{self.base_url}/resorts",
                    json=resort_data,
                    headers=get_skiapi_headers(),
                    timeout=30
                )
            
            if response.status_code in (200, 201):
                logger.info(f"Successfully updated resort: {resort_data['name']}")
                return True
            else:
                logger.error(f"Failed to update resort {resort_data['name']}: {response.status_code}")
                logger.error(response.text)
                return False
                
        except Exception as e:
            logger.error(f"Error updating resort: {e}")
            return False
    
    def update_lift(self, lift_data: Dict[str, Any]) -> bool:
        """Update lift in SkiAPI"""
        try:
            response = requests.put(
                f"{self.base_url}/lifts/{lift_data['id']}",
                json=lift_data,
                headers=get_skiapi_headers(),
                timeout=30
            )
            
            if response.status_code in (200, 201):
                logger.debug(f"Updated lift: {lift_data.get('name', 'Unknown')}")
                return True
            else:
                logger.error(f"Failed to update lift: {response.status_code}")
                logger.error(response.text)
                return False
                
        except Exception as e:
            logger.error(f"Error updating lift: {e}")
            return False
    
    def update_slope(self, slope_data: Dict[str, Any]) -> bool:
        """Update slope in SkiAPI"""
        try:
            response = requests.put(
                f"{self.base_url}/slopes/{slope_data['id']}",
                json=slope_data,
                headers=get_skiapi_headers(),
                timeout=30
            )
            
            if response.status_code in (200, 201):
                logger.debug(f"Updated slope: {slope_data.get('name', 'Unknown')}")
                return True
            else:
                logger.error(f"Failed to update slope: {response.status_code}")
                logger.error(response.text)
                return False
                
        except Exception as e:
            logger.error(f"Error updating slope: {e}")
            return False

# =========================
# DATA PROCESSING
# =========================
def process_resort_data(bergfex_resort: Dict[str, Any]) -> Dict[str, Any]:
    """Process Bergfex resort data into SkiAPI format"""
    # Extract coordinates
    lat = None
    lon = None
    if 'coordinates' in bergfex_resort:
        coords = bergfex_resort['coordinates']
        lat = coords.get('lat')
        lon = coords.get('lon')
    
    # Extract altitude information
    village_altitude = None
    min_altitude = None
    max_altitude = None
    
    if 'altitude' in bergfex_resort:
        altitude = bergfex_resort['altitude']
        village_altitude = altitude.get('village')
        min_altitude = altitude.get('min')
        max_altitude = altitude.get('max')
    
    # Extract location information
    country = bergfex_resort.get('country', 'Germany')  # Sonnenbühl-Genkingen is in Germany
    region = bergfex_resort.get('region')
    continent = 'Europe'
    
    return {
        "id": bergfex_resort.get('id', SONNENBUEHL_GENKINGEN_ID),
        "name": normalize_name(bergfex_resort.get('name')),
        "country": country,
        "region": normalize_name(region),
        "continent": continent,
        "latitude": lat,
        "longitude": lon,
        "village_altitude_m": village_altitude,
        "min_altitude_m": min_altitude,
        "max_altitude_m": max_altitude,
        "ski_area_name": normalize_name(bergfex_resort.get('skiAreaName')),
        "ski_area_type": "alpine",
        "official_website": bergfex_resort.get('website'),
        "lift_status_url": None,  # Not available from Bergfex
        "slope_status_url": None,  # Not available from Bergfex
        "snow_report_url": None,  # Not available from Bergfex
        "weather_url": None,  # Not available from Bergfex
        "status_provider": "bergfex",
        "status_last_scraped_at": datetime.now().isoformat(),
        "lifts_open_count": None,  # Will be calculated from lifts
        "slopes_open_count": None,  # Will be calculated from slopes
        "snow_depth_valley_cm": None,
        "snow_depth_mountain_cm": None,
        "new_snow_24h_cm": None,
        "temperature_valley_c": None,
        "temperature_mountain_c": None,
    }

def process_lift_data(bergfex_lift: Dict[str, Any], resort_id: str) -> Optional[Dict[str, Any]]:
    """Process Bergfex lift data into SkiAPI format"""
    # Map lift type
    lift_type = bergfex_lift.get('type')
    if not lift_type:
        logger.warning(f"Lift missing type: {bergfex_lift.get('name', 'Unknown')}")
        return None
    
    mapped_lift_type = BERGFEX_LIFT_TYPE_MAP.get(lift_type)
    if not mapped_lift_type:
        logger.warning(f"Unknown lift type '{lift_type}' for lift: {bergfex_lift.get('name', 'Unknown')}")
        return None
    
    # Map status
    status = bergfex_lift.get('status', 'unknown')
    operational_status = BERGFEX_STATUS_MAP.get(status, 'unknown')
    
    # Extract coordinates
    start_lat = None
    start_lon = None
    end_lat = None
    end_lon = None
    
    if 'coordinates' in bergfex_lift:
        coords = bergfex_lift['coordinates']
        if 'start' in coords:
            start_lat = coords['start'].get('lat')
            start_lon = coords['start'].get('lon')
        if 'end' in coords:
            end_lat = coords['end'].get('lat')
            end_lon = coords['end'].get('lon')
    
    return {
        "id": bergfex_lift.get('id'),
        "resort_id": resort_id,
        "name": normalize_name(bergfex_lift.get('name')),
        "lift_type": mapped_lift_type,
        "capacity_per_hour": bergfex_lift.get('capacity'),
        "seats": bergfex_lift.get('seats'),
        "bubble": bergfex_lift.get('bubble', False),
        "heated_seats": bergfex_lift.get('heatedSeats', False),
        "year_built": bergfex_lift.get('yearBuilt'),
        "altitude_start_m": bergfex_lift.get('altitudeStart'),
        "altitude_end_m": bergfex_lift.get('altitudeEnd'),
        "lat_start": start_lat,
        "lon_start": start_lon,
        "lat_end": end_lat,
        "lon_end": end_lon,
        "operational_status": operational_status,
        "operational_note": bergfex_lift.get('statusNote'),
        "planned_open_time": None,  # Not available from Bergfex
        "planned_close_time": None,  # Not available from Bergfex
        "status_updated_at": datetime.now().isoformat(),
    }

def process_slope_data(bergfex_slope: Dict[str, Any], resort_id: str) -> Optional[Dict[str, Any]]:
    """Process Bergfex slope data into SkiAPI format"""
    # Map difficulty
    difficulty = bergfex_slope.get('difficulty')
    if not difficulty:
        logger.warning(f"Slope missing difficulty: {bergfex_slope.get('name', 'Unknown')}")
        return None
    
    mapped_difficulty = BERGFEX_SLOPE_DIFFICULTY_MAP.get(difficulty)
    if not mapped_difficulty:
        logger.warning(f"Unknown difficulty '{difficulty}' for slope: {bergfex_slope.get('name', 'Unknown')}")
        return None
    
    # Map status
    status = bergfex_slope.get('status', 'unknown')
    operational_status = BERGFEX_STATUS_MAP.get(status, 'unknown')
    
    # Extract coordinates
    start_lat = None
    start_lon = None
    end_lat = None
    end_lon = None
    
    if 'coordinates' in bergfex_slope:
        coords = bergfex_slope['coordinates']
        if 'start' in coords:
            start_lat = coords['start'].get('lat')
            start_lon = coords['start'].get('lon')
        if 'end' in coords:
            end_lat = coords['end'].get('lat')
            end_lon = coords['end'].get('lon')
    
    # Extract length
    length_m = None
    if 'length' in bergfex_slope:
        length_m = bergfex_slope['length'].get('meters')
    
    return {
        "id": bergfex_slope.get('id'),
        "resort_id": resort_id,
        "name": normalize_name(bergfex_slope.get('name')),
        "difficulty": mapped_difficulty,
        "length_m": length_m,
        "lat_start": start_lat,
        "lon_start": start_lon,
        "lat_end": end_lat,
        "lon_end": end_lon,
        "path_geojson": None,  # Not available from Bergfex
        "operational_status": operational_status,
        "grooming_status": bergfex_slope.get('groomingStatus', 'unknown'),
        "operational_note": bergfex_slope.get('statusNote'),
        "status_updated_at": datetime.now().isoformat(),
    }

def process_weather_data(bergfex_weather: Dict[str, Any], resort_data: Dict[str, Any]) -> Dict[str, Any]:
    """Process Bergfex weather data and update resort data"""
    if not bergfex_weather:
        return resort_data
    
    # Extract temperature information
    temperature_valley = None
    temperature_mountain = None
    
    if 'temperature' in bergfex_weather:
        temp = bergfex_weather['temperature']
        temperature_valley = temp.get('valley')
        temperature_mountain = temp.get('mountain')
    
    # Extract snow information
    snow_depth_valley = None
    snow_depth_mountain = None
    new_snow_24h = None
    
    if 'snow' in bergfex_weather:
        snow = bergfex_weather['snow']
        snow_depth_valley = snow.get('depthValley')
        snow_depth_mountain = snow.get('depthMountain')
        new_snow_24h = snow.get('newSnow24h')
    
    # Update resort data with weather information
    resort_data.update({
        "temperature_valley_c": temperature_valley,
        "temperature_mountain_c": temperature_mountain,
        "snow_depth_valley_cm": snow_depth_valley,
        "snow_depth_mountain_cm": snow_depth_mountain,
        "new_snow_24h_cm": new_snow_24h,
        "status_last_scraped_at": datetime.now().isoformat(),
    })
    
    return resort_data

def calculate_open_counts(lifts: List[Dict[str, Any]], slopes: List[Dict[str, Any]], resort_data: Dict[str, Any]) -> Dict[str, Any]:
    """Calculate open lift and slope counts"""
    open_lifts = sum(1 for lift in lifts if lift.get('operational_status') == 'open')
    open_slopes = sum(1 for slope in slopes if slope.get('operational_status') == 'open')
    
    resort_data.update({
        "lifts_open_count": open_lifts,
        "slopes_open_count": open_slopes,
    })
    
    return resort_data

# =========================
# MAIN COLLECTION LOGIC
# =========================
async def collect_bergfex_data(resort_id: str) -> bool:
    """Main function to collect data from Bergfex API"""
    logger.info(f"Starting data collection for resort: {resort_id}")
    
    # Initialize API clients
    ski_api = SkiApiClient()
    
    try:
        async with BergfexApiClient(resort_id) as bergfex_api:
            # Collect data from Bergfex
            logger.info("Fetching resort information...")
            resort_info = await bergfex_api.get_resort_info()
            if not resort_info:
                logger.error(f"Failed to get resort information for {resort_id}")
                return False
            
            logger.info("Fetching lifts information...")
            lifts_data = await bergfex_api.get_lifts()
            
            logger.info("Fetching slopes information...")
            slopes_data = await bergfex_api.get_slopes()
            
            logger.info("Fetching weather information...")
            weather_data = await bergfex_api.get_weather()
            
            # Process data
            logger.info("Processing collected data...")
            
            # Process resort data
            processed_resort = process_resort_data(resort_info)
            
            # Process lifts
            processed_lifts = []
            for lift in lifts_data:
                processed_lift = process_lift_data(lift, processed_resort['id'])
                if processed_lift:
                    processed_lifts.append(processed_lift)
            
            # Process slopes
            processed_slopes = []
            for slope in slopes_data:
                processed_slope = process_slope_data(slope, processed_resort['id'])
                if processed_slope:
                    processed_slopes.append(processed_slope)
            
            # Process weather data
            processed_resort = process_weather_data(weather_data, processed_resort)
            
            # Calculate open counts
            processed_resort = calculate_open_counts(processed_lifts, processed_slopes, processed_resort)
            
            # Save data to SkiAPI
            logger.info("Updating SkiAPI with collected data...")
            
            # Update resort
            success = ski_api.create_or_update_resort(processed_resort)
            if not success:
                logger.error("Failed to update resort in SkiAPI")
                return False
            
            # Update lifts
            lifts_success = True
            for lift in processed_lifts:
                if not ski_api.update_lift(lift):
                    lifts_success = False
                    logger.error(f"Failed to update lift: {lift.get('name', 'Unknown')}")
            
            # Update slopes
            slopes_success = True
            for slope in processed_slopes:
                if not ski_api.update_slope(slope):
                    slopes_success = False
                    logger.error(f"Failed to update slope: {slope.get('name', 'Unknown')}")
            
            if lifts_success and slopes_success:
                logger.info("Successfully collected and updated all data")
                return True
            else:
                logger.error("Some data failed to update")
                return False
                
    except Exception as e:
        logger.error(f"Error during data collection: {e}")
        return False

# =========================
# CHECKPOINTING
# =========================
def save_checkpoint(data: Dict[str, Any], filename: str):
    """Save data to checkpoint file"""
    os.makedirs(CHECKPOINTS_DIR, exist_ok=True)
    checkpoint_file = CHECKPOINTS_DIR / filename
    
    with open(checkpoint_file, 'w', encoding='utf-8') as f:
        json.dump(data, f, indent=2, ensure_ascii=False)
    
    logger.info(f"Checkpoint saved: {checkpoint_file}")

def load_checkpoint(filename: str) -> Optional[Dict[str, Any]]:
    """Load data from checkpoint file"""
    checkpoint_file = CHECKPOINTS_DIR / filename
    
    if not checkpoint_file.exists():
        return None
    
    try:
        with open(checkpoint_file, 'r', encoding='utf-8') as f:
            return json.load(f)
    except Exception as e:
        logger.error(f"Error loading checkpoint {checkpoint_file}: {e}")
        return None

# =========================
# MAIN
# =========================
def main():
    """Main entry point"""
    global logger
    
    parser = argparse.ArgumentParser(description="Bergfex API Collector for Sonnenbühl-Genkingen")
    parser.add_argument("--resort-id", default=SONNENBUEHL_GENKINGEN_ID, 
                       help="Resort ID (default: sonnenbuehl-genkingen)")
    parser.add_argument("--api-key", help="SkiAPI key (overrides environment/file)")
    parser.add_argument("--checkpoint", action="store_true", 
                       help="Save collected data to checkpoint files")
    
    args = parser.parse_args()
    
    # Setup
    logger = setup_logging()
    load_api_key()
    
    if args.api_key:
        global SKI_API_KEY
        SKI_API_KEY = args.api_key
        HEADERS["Authorization"] = f"Bearer {SKI_API_KEY}"
    
    logger.info(f"Starting Bergfex collector for resort: {args.resort_id}")
    logger.info(f"SkiAPI URL: {SKI_API_BASE_URL}")
    
    # Run collection
    start_time = time.time()
    success = asyncio.run(collect_bergfex_data(args.resort_id))
    duration = time.time() - start_time
    
    if success:
        logger.info(f"Collection completed successfully in {duration:.2f} seconds")
        sys.exit(0)
    else:
        logger.error("Collection failed")
        sys.exit(1)

if __name__ == "__main__":
    main()