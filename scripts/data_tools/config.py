"""
Shared Configuration for Openslope Data Collection Tools

This module contains all configuration values shared across the data collection
system including collect_and_sync.py and launcher.py.

Edit the values in the "USER CONFIGURATION" section to customize behavior.
"""

# =============================================================================
# USER CONFIGURATION <-- EDIT THESE VALUES
# =============================================================================

# Openslope API base URL (no trailing slash)
API_BASE_URL = "http://localhost:8080"

# API key for authentication (Bearer token)
API_KEY = "R3StTY4OfadeFJZurXdZ1pZMVbWB3zWuL6FnuPGIbvA"

# Overpass API endpoint for OSM queries
OVERPASS_URL = "https://overpass-api.de/api/interpreter"

# HTTP request delay between API calls in seconds (rate limiting)
REQUEST_DELAY = 0.1

# Number of retries for failed HTTP requests
HTTP_RETRIES = 3

# Overpass query timeout in seconds
OVERPASS_TIMEOUT = 90

# Batch size for processing features before syncing to API
BATCH_SIZE = 500

# Default number of parallel worker instances
DEFAULT_INSTANCES = 4

# Log level: logging.DEBUG, INFO, WARNING, ERROR
LOG_LEVEL = "INFO"


# =============================================================================
# DERIVED CONFIGURATION (do not edit below this line)
# =============================================================================

# Constructed API endpoints
SLOPES_ENDPOINT = f"{API_BASE_URL}/slopes"
LIFTS_ENDPOINT = f"{API_BASE_URL}/lifts"
RESORTS_ENDPOINT = f"{API_BASE_URL}/resorts"

# HTTP headers for API requests
HEADERS = {
    "Content-Type": "application/json",
    "Authorization": f"Bearer {API_KEY}",
}

# Progress file directory
PROGRESS_DIR = "scripts/data_tools"