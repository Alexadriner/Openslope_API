# Spitzingsee Scraper

Scraper for Spitzingsee ski resort data from Alpenplus.com.

## Data Source

- **URL**: https://sdds4.intermaps.com/alpenplus/snowreport_alpenplus.aspx
- **Type**: Shared iframe with all Alpenplus resorts
- **Update Frequency**: Every 24 hours (as per requirements)

## Data Collected

- Lift counts (open/total)
- Slope counts (open/total) 
- Snow depths (valley/mountain)
- New snow (24h)
- Temperatures (valley/mountain)

## Usage

```bash
# Run once
python -m scripts.website_scrapers.alpenplus.spitzingsee.collector --once

# Run with 24-hour interval
python -m scripts.website_scrapers.alpenplus.spitzingsee.collector --interval-seconds 86400

# Run without API sync
python -m scripts.website_scrapers.alpenplus.spitzingsee.collector --no-sync-api
```

## Configuration

- **Resort ID**: `spitzingsee`
- **Scraper Name**: `alpenplus_spitzingsee`
- **Base URL**: `https://www.alpenplus.com`
- **Data URL**: `https://sdds4.intermaps.com/alpenplus/snowreport_alpenplus.aspx`

## Output

Data is saved to:
- Checkpoints: `checkpoints/website_scrapers/alpenplus_spitzingsee/`
- Logs: `logs/website_scrapers/alpenplus_spitzingsee/`