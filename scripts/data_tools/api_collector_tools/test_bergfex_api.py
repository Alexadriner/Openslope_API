#!/usr/bin/env python3
"""
Test script for Bergfex API to find the correct resort ID
"""

import asyncio
import aiohttp
import logging
import sys
from pathlib import Path

# Setup logging
logging.basicConfig(level=logging.INFO, format="%(asctime)s | %(levelname)s | %(message)s")
logger = logging.getLogger(__name__)

BERGFEX_API_BASE = "https://www.bergfex.at/api/v1"

async def test_resort_endpoint(resort_id: str, auth_token: Optional[str] = None):
    """Test a specific resort endpoint"""
    url = f"{BERGFEX_API_BASE}/skiresorts/{resort_id}"
    headers = {"User-Agent": "BergfexTest/1.0"}
    
    if auth_token:
        headers["Authorization"] = f"Bearer {auth_token}"
    
    async with aiohttp.ClientSession(headers=headers) as session:
        try:
            async with session.get(url) as response:
                logger.info(f"Testing resort ID: {resort_id} {'with auth' if auth_token else 'without auth'}")
                logger.info(f"Status: {response.status}")
                
                if response.status == 200:
                    data = await response.json()
                    logger.info(f"Success! Resort name: {data.get('name', 'Unknown')}")
                    return True
                else:
                    logger.warning(f"Failed with status {response.status}")
                    return False
        except Exception as e:
            logger.error(f"Error testing {resort_id}: {e}")
            return False

async def test_known_resorts():
    """Test some known resort IDs"""
    # Common German/Austrian resort patterns
    test_ids = [
        "sonnenbuehl-genkingen",
        "sonnenbuehl",
        "genkingen",
        "sonnenbuehl_genkingen", 
        "sonnenbuehl-genkingen-ski",
        "sonnenbuehl-ski",
        "genkingen-ski",
        "sonnenbuehl-genkingen-ski-resort",
        "sonnenbuehl-ski-resort",
        "genkingen-ski-resort"
    ]
    
    logger.info("Testing various resort ID patterns...")
    
    for resort_id in test_ids:
        success = await test_resort_endpoint(resort_id)
        if success:
            logger.info(f"Found working resort ID: {resort_id}")
            return resort_id
        await asyncio.sleep(1)  # Be respectful to the API
    
    return None

async def list_available_resorts():
    """Try to find a list of available resorts"""
    # Try common endpoints that might list resorts
    endpoints = [
        "/public/skiresorts",
        "/public/resorts",
        "/skiresorts",
        "/resorts",
        "/api/skiresorts",
        "/api/resorts",
        "/v1/skiresorts",
        "/v1/resorts"
    ]
    
    headers = {"User-Agent": "BergfexTest/1.0"}
    
    async with aiohttp.ClientSession(headers=headers) as session:
        for endpoint in endpoints:
            url = f"{BERGFEX_API_BASE}{endpoint}"
            try:
                async with session.get(url) as response:
                    logger.info(f"Testing endpoint: {endpoint}")
                    logger.info(f"Status: {response.status}")
                    
                    if response.status == 200:
                        data = await response.json()
                        logger.info(f"Success! Response keys: {list(data.keys()) if isinstance(data, dict) else 'List'}")
                        if isinstance(data, list) and data:
                            logger.info(f"First item: {data[0]}")
                        return True
                    elif response.status == 401:
                        logger.warning(f"Authentication required for {endpoint}")
                    elif response.status == 403:
                        logger.warning(f"Access forbidden for {endpoint}")
                    else:
                        logger.warning(f"Failed with status {response.status}")
            except Exception as e:
                logger.error(f"Error testing {endpoint}: {e}")
            
            await asyncio.sleep(1)
    
    return False

async def test_with_authentication(resort_id: str, auth_token: Optional[str] = None):
    """Test with authentication token"""
    url = f"{BERGFEX_API_BASE}/skiresorts/{resort_id}"
    headers = {"User-Agent": "BergfexTest/1.0"}
    
    if auth_token:
        headers["Authorization"] = f"Bearer {auth_token}"
    
    async with aiohttp.ClientSession(headers=headers) as session:
        try:
            async with session.get(url) as response:
                logger.info(f"Testing resort ID: {resort_id} with auth")
                logger.info(f"Status: {response.status}")
                
                if response.status == 200:
                    data = await response.json()
                    logger.info(f"Success! Resort name: {data.get('name', 'Unknown')}")
                    return True
                elif response.status == 401:
                    logger.warning("Authentication failed - invalid token")
                elif response.status == 403:
                    logger.warning("Access forbidden - insufficient permissions")
                else:
                    logger.warning(f"Failed with status {response.status}")
                return False
        except Exception as e:
            logger.error(f"Error testing {resort_id} with auth: {e}")
            return False

async def main():
    """Main test function"""
    logger.info("Starting Bergfex API test...")
    
    # First try to list available resorts
    logger.info("Testing for resort listing endpoints...")
    has_listing = await list_available_resorts()
    
    if not has_listing:
        logger.info("No resort listing found, testing individual IDs...")
        found_id = await test_known_resorts()
        
        if found_id:
            logger.info(f"Use this ID in your collector: {found_id}")
        else:
            logger.error("Could not find a working resort ID")
            logger.info("Please check the Bergfex API documentation for the correct resort ID")
            logger.info("Documentation URL: https://swagger.bergfex.at/#/specifications/public/skiresorts.yaml")
    
    logger.info("Test completed")

if __name__ == "__main__":
    asyncio.run(main())