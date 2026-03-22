# Alpenplus Website Scrapers

Collection of scrapers for Alpenplus ski resorts: Brauneck, Spitzingsee, Sudelfeld, and Wallberg.

## Overview

All Alpenplus resorts share the same data source:
- **URL**: https://sdds4.intermaps.com/alpenplus/snowreport_alpenplus.aspx
- **Type**: Shared iframe containing data for all 4 resorts
- **Update Frequency**: Every 24 hours (as per requirements)

## Resorts

This package includes scrapers for the following ski resorts:

1. **Brauneck** (`alpenplus_brauneck`)
2. **Spitzingsee** (`alpenplus_spitzingsee`) 
3. **Sudelfeld** (`alpenplus_sudelfeld`)
4. **Wallberg** (`alpenplus_wallberg`)

## Architecture

### Base Class
- **File**: `base.py`
- **Purpose**: Common logic for extracting data from the shared Alpenplus iframe
- **Features**: 
  - HTML parsing and resort section extraction
  - Data normalization according to API structure
  - Regex-based data extraction

### Individual Scrapers
Each resort has:
- **Scraper**: Inherits from `AlpenplusBaseScraper`, specifies resort name and ID
- **Collector**: Handles API synchronization and scheduling
- **README**: Usage instructions and configuration details

## Data Collected

The scrapers collect the following data (aligned with API requirements):

- **Lift Status**: Open count, total count
- **Slope Status**: Open count, total count  
- **Snow Data**: Valley/mountain depths, 24h new snow
- **Temperature**: Valley/mountain temperatures

## Usage

### Individual Resort Collection

```bash
# Brauneck
python -m scripts.website_scrapers.alpenplus.brauneck.collector --once

# Spitzingsee  
python -m scripts.website_scrapers.alpenplus.spitzingsee.collector --once

# Sudelfeld
python -m scripts.website_scrapers.alpenplus.sudelfeld.collector --once

# Wallberg
python -m scripts.website_scrapers.alpenplus.wallberg.collector --once
```

### 24-Hour Scheduling

```bash
# Run with 24-hour interval (86400 seconds)
python -m scripts.website_scrapers.alpenplus.brauneck.collector --interval-seconds 86400

# Or use the launch_collectors.py with specific resorts
python scripts/website_scrapers/launch_collectors.py --only alpenplus_brauneck,alpenplus_spitzingsee,alpenplus_sudelfeld,alpenplus_wallberg --interval-seconds 86400
```

### Launch All Alpenplus Scrapers

```bash
# Launch all Alpenplus scrapers with 24-hour interval
python scripts/website_scrapers/launch_collectors.py --only alpenplus_brauneck,alpenplus_spitzingsee,alpenplus_sudelfeld,alpenplus_wallberg --interval-seconds 86400
```

## Configuration

### Environment Variables
- `API_BASE_URL`: API endpoint (default: `http://localhost:8080`)
- `API_KEY`: Authentication key (default: `R3StTY4OfadeFJZurXdZ1pZMVbWB3zWuL6FnuPGIbvA`)

### Collector Options
- `--resort-id`: Resort identifier (default: resort name)
- `--interval-seconds`: Collection interval (default: 300s)
- `--once`: Run once instead of continuous loop
- `--no-sync-api`: Skip API synchronization

## Output

Data is saved to:
- **Checkpoints**: `checkpoints/website_scrapers/alpenplus_{resort}/`
- **Logs**: `logs/website_scrapers/alpenplus_{resort}/`

## API Integration

The scrapers integrate with the Openslope API to:
- Update resort status information
- Sync lift and slope operational status
- Maintain historical data snapshots

## Troubleshooting

### Common Issues

1. **Resort Section Not Found**: The base scraper may fail to locate the resort's HTML section. Check the iframe source for anchor IDs.
2. **API Sync Failures**: Ensure API credentials are correct and the API is running.
3. **Rate Limiting**: The base scraper includes rate limiting (0.6s between requests).

### Debug Mode

Run with `--no-sync-api` to test scraping without API calls:
```bash
python -m scripts.website_scrapers.alpenplus.brauneck.collector --once --no-sync-api
```

## Maintenance

- Monitor logs for scraping errors
- Verify API connectivity regularly
- Check for changes to the Alpenplus iframe structure
- Update regex patterns if HTML structure changes