from scripts.website_scrapers.alpenplus.base import AlpenplusBaseScraper
from scripts.website_scrapers.base import ScraperConfig


class SpitzingseeScraper(AlpenplusBaseScraper):
    """
    Scraper for Spitzingsee ski resort.
    
    Uses the shared Alpenplus snow report page and extracts Spitzingsee-specific data.
    """

    def __init__(self) -> None:
        config = ScraperConfig(
            scraper_name="alpenplus_spitzingsee",
            base_url="https://www.alpenplus.com",
        )
        super().__init__(
            config=config,
            resort_name="Spitzingsee",
            resort_id="spitzingsee"  # This matches the anchor ID in the HTML
        )