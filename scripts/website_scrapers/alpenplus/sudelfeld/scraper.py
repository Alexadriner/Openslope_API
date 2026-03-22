from scripts.website_scrapers.alpenplus.base import AlpenplusBaseScraper
from scripts.website_scrapers.base import ScraperConfig


class SudelfeldScraper(AlpenplusBaseScraper):
    """
    Scraper for Sudelfeld ski resort.
    
    Uses the shared Alpenplus snow report page and extracts Sudelfeld-specific data.
    """

    def __init__(self) -> None:
        config = ScraperConfig(
            scraper_name="alpenplus_sudelfeld",
            base_url="https://www.alpenplus.com",
        )
        super().__init__(
            config=config,
            resort_name="Sudelfeld",
            resort_id="sudelfeld"  # This matches the anchor ID in the HTML
        )