from scripts.website_scrapers.alpenplus.base import AlpenplusBaseScraper
from scripts.website_scrapers.base import ScraperConfig


class WallbergScraper(AlpenplusBaseScraper):
    """
    Scraper for Wallberg ski resort.
    
    Uses the shared Alpenplus snow report page and extracts Wallberg-specific data.
    """

    def __init__(self) -> None:
        config = ScraperConfig(
            scraper_name="alpenplus_wallberg",
            base_url="https://www.alpenplus.com",
        )
        super().__init__(
            config=config,
            resort_name="Wallberg",
            resort_id="wallberg"  # This matches the anchor ID in the HTML
        )