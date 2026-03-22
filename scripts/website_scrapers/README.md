# Website Scrapers

This directory contains a collection of web scraping modules for collecting ski resort data from various sources. The system is designed to be modular, allowing different collectors to be run independently or in parallel.

## Overview

The website scrapers are organized into individual modules, each responsible for collecting data from a specific ski resort or group of resorts. The main launcher script (`launch_collectors.py`) can run multiple collectors in parallel, making it easy to collect data from multiple sources simultaneously.

## Structure

```
scripts/website_scrapers/
├── launch_collectors.py          # Main launcher script
├── base.py                      # Base classes and utilities
├── alpenplus/                   # Alpenplus resort group
│   ├── __init__.py
│   ├── base.py                  # Alpenplus-specific base classes
│   ├── README.md
│   ├── brauneck/               # Brauneck resort
│   │   ├── __init__.py
│   │   ├── collector.py
│   │   ├── scraper.py
│   │   └── README.md
│   ├── spitzingsee/            # Spitzingsee resort
│   │   ├── __init__.py
│   │   ├── collector.py
│   │   ├── scraper.py
│   │   └── README.md
│   ├── sudelfeld/              # Sudelfeld resort
│   │   ├── __init__.py
│   │   ├── collector.py
│   │   ├── scraper.py
│   │   └── README.md
│   └── wallberg/               # Wallberg resort
│       ├── __init__.py
│       ├── collector.py
│       ├── scraper.py
│       └── README.md
├── kreuzberg/                   # Kreuzberg resort
│   ├── __init__.py
│   ├── collector.py
│   ├── scraper.py
│   └── README.md
└── palisades_tahoe/            # Palisades Tahoe resort
    ├── __init__.py
    ├── collector.py
    ├── scraper.py
    └── README.md
```

## Usage

### Running All Collectors

To run all available collectors in parallel:

```bash
python scripts/website_scrapers/launch_collectors.py
```

### Running Specific Collectors

To run only specific collectors:

```bash
python scripts/website_scrapers/launch_collectors.py --only "kreuzberg,palisades_tahoe"
```

### Running with Custom Interval

To set a custom polling interval (in seconds):

```bash
python scripts/website_scrapers/launch_collectors.py --interval-seconds 600
```

### Running Once

To run collectors only once instead of continuously:

```bash
python scripts/website_scrapers/launch_collectors.py --once
```

### Skipping Collectors

To skip certain collectors:

```bash
python scripts/website_scrapers/launch_collectors.py --skip "kreuzberg,palisades_tahoe"
```

### Disabling API Sync

To run collectors without syncing to the API:

```bash
python scripts/website_scrapers/launch_collectors.py --no-sync-api
```

## Command Line Options

- `--interval-seconds`: Polling interval for each collector (default: 300 seconds)
- `--once`: Run each collector only once
- `--no-sync-api`: Pass --no-sync-api to all collectors
- `--only`: Comma-separated list of resort slugs to start
- `--skip`: Comma-separated list of resort slugs to skip

## Collector Architecture

Each collector follows a consistent architecture:

1. **Scraper Module**: Handles the actual web scraping logic, extracting data from the resort's website
2. **Collector Module**: Orchestrates the scraping process, handles scheduling, and manages data persistence
3. **Base Classes**: Provide common functionality and utilities for all collectors

## Adding a New Collector

To add a new collector for a ski resort:

1. Create a new directory under `scripts/website_scrapers/` with the resort name
2. Create the following files:
   - `__init__.py`: Empty initialization file
   - `scraper.py`: Web scraping logic
   - `collector.py`: Collection orchestration
   - `README.md`: Documentation for the collector
3. Implement the scraper and collector classes following the existing patterns
4. The launcher will automatically discover and include the new collector

## Error Handling

The launcher includes robust error handling:

- Graceful shutdown on SIGINT and SIGTERM
- Process monitoring and cleanup
- Exit code propagation from individual collectors
- Automatic termination of hanging processes

## Dependencies

The scrapers use standard Python libraries and may require additional dependencies for specific functionality. Check individual collector README files for specific requirements.

## Logging

Each collector writes logs to its own directory under `scripts/logs/website_scrapers/<resort_name>/`. Check these directories for detailed information about collector runs and any errors that occur.

## Notes

- Collectors are designed to be run from the project root directory
- The launcher automatically discovers all collector modules
- Alpenplus resorts are handled as a special case with their own subdirectory structure
- Each collector can be run independently or as part of the parallel launcher