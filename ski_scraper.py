#!/usr/bin/env python3
"""
Ski Scraper Entry Point

This is the main entry point for the ski data scraping system. It serves as a
wrapper script that imports and executes the main scraping functionality from
the data_tools module.

## Purpose

The ski_scraper.py script provides a simple command-line interface for running
the comprehensive ski data collection pipeline. It delegates the actual work
to the main() function in the scripts.data_tools.ski_scraper module.

## Usage

```bash
# Run the full scraping pipeline
python ski_scraper.py

# Run with specific arguments (passed through to the main module)
python ski_scraper.py --help
```

## Architecture

This script follows a modular design where:
- The main scraping logic is contained in `scripts.data_tools.ski_scraper`
- This entry point provides a clean interface for execution
- All command-line arguments are passed through to the main module

## Integration

This script is designed to work with:
- The parallel launcher system (`collect_geojson_launcher.py`)
- Worker-based processing for large datasets
- Progress tracking and checkpoint systems
- Multiple data source integration

## Dependencies

The script depends on the `scripts.data_tools.ski_scraper` module, which
contains the actual scraping implementation and data processing logic.

## Author
OpenSlope Team

## Version
1.0.0
"""

from scripts.data_tools.ski_scraper import main


if __name__ == "__main__":
    main()