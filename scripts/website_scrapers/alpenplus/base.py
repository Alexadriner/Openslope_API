"""
Alpenplus Base Scraper

Base class for Alpenplus ski resort scrapers.

All Alpenplus resorts share the same data source:
https://sdds4.intermaps.com/alpenplus/snowreport_alpenplus.aspx

This base class handles the common logic for extracting data from the shared iframe.
Individual resort scrapers only need to specify their resort name and ID mapping.

Author: OpenSlope Team
"""

import logging
import re
from datetime import datetime
from typing import Any, Dict, List, Optional
from urllib.parse import urljoin

from scripts.website_scrapers.base import ScraperConfig, WebsiteScraperBase


class AlpenplusBaseScraper(WebsiteScraperBase):
    """
    Base class for Alpenplus ski resort scrapers.
    
    All Alpenplus resorts share the same data source:
    https://sdds4.intermaps.com/alpenplus/snowreport_alpenplus.aspx
    
    This base class handles the common logic for extracting data from the shared iframe.
    Individual resort scrapers only need to specify their resort name and ID mapping.
    """

    # Shared snow report URL for all Alpenplus resorts
    SNOW_REPORT_URL = "https://sdds4.intermaps.com/alpenplus/snowreport_alpenplus.aspx"

    def __init__(self, config: ScraperConfig, resort_name: str, resort_id: str):
        """
        Initialize the Alpenplus base scraper.
        
        Args:
            config: Scraper configuration object
            resort_name: Human-readable name of the resort
            resort_id: Internal ID used in the Alpenplus system
        """
        super().__init__(config)
        self.resort_name = resort_name
        self.resort_id = resort_id
        self.logger = logging.getLogger(f"website_scraper.alpenplus.{resort_name.lower()}")

    def fetch_raw_payload(self, resort_id: str) -> dict[str, Any]:
        """
        Fetch raw HTML from the shared Alpenplus snow report page.
        
        Args:
            resort_id: The resort ID (not used in this implementation since all resorts share the same URL)
        
        Returns:
            dict: Raw payload containing HTML and metadata
        """
        html = self.get_html(self.SNOW_REPORT_URL)
        return {
            "report_url": self.SNOW_REPORT_URL,
            "html": html,
            "resort_name": self.resort_name,
            "resort_id": self.resort_id,
        }

    def normalize_payload(self, resort_id: str, raw_payload: dict[str, Any]) -> dict[str, Any]:
        """
        Extract and normalize data for the specific resort from the shared HTML.
        
        Args:
            resort_id: The resort ID (not used in this implementation)
            raw_payload: Raw payload containing HTML and metadata
        
        Returns:
            dict: Normalized payload with resort, lift, and slope data
        """
        html = raw_payload.get("html", "")
        
        # Find the resort section in HTML
        resort_section = self._extract_resort_section(html)
        if not resort_section:
            self.logger.warning(f"Could not find section for {self.resort_name}")
            return self._empty_payload()

        # Extract data using regex patterns
        data = self._extract_resort_data(resort_section)
        
        # Build normalized payload according to API structure
        resort_data = {
            "official_website": self.config.base_url,
            "lift_status_url": self.SNOW_REPORT_URL,
            "slope_status_url": self.SNOW_REPORT_URL,
            "snow_report_url": self.SNOW_REPORT_URL,
            "weather_url": self.SNOW_REPORT_URL,
            "status_provider": "alpenplus_intermaps",
            "status_last_scraped_at": datetime.now().isoformat(),
            "lifts_open_count": data.get("lifts_open"),
            "slopes_open_count": data.get("slopes_open"),
            "snow_depth_valley_cm": data.get("snow_depth_valley"),
            "snow_depth_mountain_cm": data.get("snow_depth_mountain"),
            "new_snow_24h_cm": data.get("new_snow_24h"),
            "temperature_valley_c": data.get("temperature_valley"),
            "temperature_mountain_c": data.get("temperature_mountain"),
        }

        # Extract lift and slope information
        lifts = self._extract_lifts(resort_section)
        slopes = self._extract_slopes(resort_section)

        return {
            "resort": resort_data,
            "lifts": lifts,
            "slopes": slopes
        }

    def _extract_resort_section(self, html: str) -> Optional[str]:
        """
        Extract the HTML section for this specific resort.
        
        Args:
            html: Full HTML content from the snow report page
        
        Returns:
            Optional[str]: HTML section for the resort, or None if not found
        """
        # Look for the anchor and extract content until next resort or end
        pattern = rf'<div class="anchor" id="{self.resort_id}"></div>(.*?)(?=<div class="anchor"|$)'
        match = re.search(pattern, html, re.DOTALL | re.IGNORECASE)
        return match.group(1) if match else None

    def _extract_resort_data(self, html_section: str) -> Dict[str, Any]:
        """
        Extract basic resort data from the HTML section.
        
        Args:
            html_section: HTML section containing data for this resort
        
        Returns:
            Dict[str, Any]: Extracted resort data
        """
        data = {}
        
        # Extract lift counts
        lift_match = re.search(r'Anlagen\s*<br[^>]*>\s*(\d+) von (\d+)', html_section)
        if lift_match:
            data['lifts_open'] = int(lift_match.group(1))
            data['lifts_total'] = int(lift_match.group(2))
        else:
            data['lifts_open'] = None
            data['lifts_total'] = None
        
        # Extract slope counts
        slope_match = re.search(r'Pisten\s*<br[^>]*>\s*(\d+) von (\d+)', html_section)
        if slope_match:
            data['slopes_open'] = int(slope_match.group(1))
            data['slopes_total'] = int(slope_match.group(2))
        else:
            data['slopes_open'] = None
            data['slopes_total'] = None
        
        # Extract snow depths
        snow_match = re.search(r'Schneehöhe \(Berg/Tal\)\s*<br[^>]*>\s*(\d+)/(\d+) cm', html_section)
        if snow_match:
            data['snow_depth_mountain'] = int(snow_match.group(1))
            data['snow_depth_valley'] = int(snow_match.group(2))
        else:
            data['snow_depth_mountain'] = None
            data['snow_depth_valley'] = None
        
        # Extract new snow
        new_snow_match = re.search(r'Neuschnee \(Berg/Tal\)\s*<br[^>]*>\s*(\d+)/(\d+) cm', html_section)
        if new_snow_match:
            data['new_snow_mountain'] = int(new_snow_match.group(1))
            data['new_snow_valley'] = int(new_snow_match.group(2))
            # API expects new_snow_24h_cm, we'll use the mountain value if available
            data['new_snow_24h'] = data['new_snow_mountain'] if data['new_snow_mountain'] is not None else data['new_snow_valley']
        else:
            data['new_snow_mountain'] = None
            data['new_snow_valley'] = None
            data['new_snow_24h'] = None
        
        # Extract temperatures
        temp_match = re.search(r'-?(\d+)/-?(\d+)°C', html_section)
        if temp_match:
            data['temperature_valley'] = int(temp_match.group(1))
            data['temperature_mountain'] = int(temp_match.group(2))
        else:
            data['temperature_valley'] = None
            data['temperature_mountain'] = None
        
        return data

    def _extract_lifts(self, html_section: str) -> List[Dict[str, Any]]:
        """
        Extract lift information from the HTML section.
        
        Args:
            html_section: HTML section containing data for this resort
        
        Returns:
            List[Dict[str, Any]]: List of lift data dictionaries
        """
        lifts = []
        
        # This is a simplified extraction - in a real implementation,
        # you would need to parse the specific lift data from the HTML
        # For now, we'll create placeholder entries based on the counts
        
        data = self._extract_resort_data(html_section)
        total_lifts = data.get('lifts_total', 0) or 0
        open_lifts = data.get('lifts_open', 0) or 0
        
        # Ensure we have valid integers
        try:
            total_lifts = int(total_lifts)
            open_lifts = int(open_lifts)
        except (ValueError, TypeError):
            total_lifts = 0
            open_lifts = 0
        
        for i in range(total_lifts):
            status = "open" if i < open_lifts else "closed"
            lifts.append({
                "source_entity_id": f"{self.resort_id}_lift_{i+1}",
                "name": f"Lift {i+1}",
                "operational_status": status,
                "operational_note": None,
                "status_updated_at": None,
                "status_source_url": self.SNOW_REPORT_URL,
            })
        
        return lifts

    def _extract_slopes(self, html_section: str) -> List[Dict[str, Any]]:
        """
        Extract slope information from the HTML section.
        
        Args:
            html_section: HTML section containing data for this resort
        
        Returns:
            List[Dict[str, Any]]: List of slope data dictionaries
        """
        slopes = []
        
        # Similar to lifts, this is simplified
        data = self._extract_resort_data(html_section)
        total_slopes = data.get('slopes_total', 0) or 0
        open_slopes = data.get('slopes_open', 0) or 0
        
        # Ensure we have valid integers
        try:
            total_slopes = int(total_slopes)
            open_slopes = int(open_slopes)
        except (ValueError, TypeError):
            total_slopes = 0
            open_slopes = 0
        
        for i in range(total_slopes):
            status = "open" if i < open_slopes else "closed"
            slopes.append({
                "source_entity_id": f"{self.resort_id}_slope_{i+1}",
                "name": f"Slope {i+1}",
                "operational_status": status,
                "grooming_status": "unknown",
                "operational_note": None,
                "status_updated_at": None,
                "status_source_url": self.SNOW_REPORT_URL,
            })
        
        return slopes

    def _empty_payload(self) -> Dict[str, Any]:
        """
        Return an empty payload when resort data cannot be found.
        
        Returns:
            Dict[str, Any]: Empty payload with default values
        """
        return {
            "resort": {
                "official_website": self.config.base_url,
                "lift_status_url": self.SNOW_REPORT_URL,
                "slope_status_url": self.SNOW_REPORT_URL,
                "snow_report_url": self.SNOW_REPORT_URL,
                "weather_url": self.SNOW_REPORT_URL,
                "status_provider": "alpenplus_intermaps",
                "status_last_scraped_at": datetime.now().isoformat(),
                "lifts_open_count": None,
                "slopes_open_count": None,
                "snow_depth_valley_cm": None,
                "snow_depth_mountain_cm": None,
                "new_snow_24h_cm": None,
                "temperature_valley_c": None,
                "temperature_mountain_c": None,
            },
            "lifts": [],
            "slopes": []
        }