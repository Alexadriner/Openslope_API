from scripts.website_scrapers.alpenplus.base import AlpenplusBaseScraper
from scripts.website_scrapers.base import ScraperConfig


class BrauneckScraper(AlpenplusBaseScraper):
    """
    Scraper for Brauneck-Wegscheid ski resort.
    
    Uses the shared Alpenplus snow report page and extracts Brauneck-specific data.
    """

    def __init__(self) -> None:
        config = ScraperConfig(
            scraper_name="alpenplus_brauneck",
            base_url="https://www.alpenplus.com",
        )
        super().__init__(
            config=config,
            resort_name="Brauneck",
            resort_id="brauneck"  # This matches the anchor ID in the HTML
        )