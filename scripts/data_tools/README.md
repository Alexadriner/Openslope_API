# Openslope Data Collection Tools

This directory contains the unified data collection system for fetching ski slopes and lifts from OpenStreetMap and syncing them to the Openslope REST API.

## Overview

The data collection system consists of three main components:

1. **config.py** - Shared configuration for all tools
2. **collect_and_sync.py** - Single worker that processes a longitude range
3. **parallel_launcher.py** - Launches multiple parallel workers

Additionally, the `server/wipe_table/` directory contains a Rust tool for backing up and clearing tables.

## Quick Start

### Collecting Data (Recommended)

Launch multiple parallel workers to collect data worldwide:

```bash
# Start 10 parallel workers
python scripts/data_tools/parallel_launcher.py --instances 10

# Check progress
cat scripts/data_tools/progress_*.json
```

### Single Worker

Run a single worker for a specific longitude range:

```bash
python scripts/data_tools/collect_and_sync.py \
    --instance-id 1 \
    --lng-min -180 \
    --lng-max -144
```

## Configuration

Edit `scripts/data_tools/config.py` to customize:

```python
# API Configuration
API_BASE_URL = "http://localhost:8080"
API_KEY = "your-api-key-here"

# Overpass API
OVERPASS_URL = "https://overpass-api.de/api/interpreter"
OVERPASS_TIMEOUT = 90

# Rate Limiting
REQUEST_DELAY = 0.1  # Seconds between API calls
HTTP_RETRIES = 3

# Processing
BATCH_SIZE = 500  # Features per batch before syncing
DEFAULT_INSTANCES = 4  # Default number of parallel workers
```

## Components

### parallel_launcher.py

Launches N parallel worker instances, distributing the global longitude range (-180 to +180) evenly.

```bash
# Usage
python parallel_launcher.py --instances N [--dry-run] [--clear]

# Options
--instances N    Number of parallel workers (default: 4)
--dry-run        Preview assignments without starting workers
--clear          Clear all progress files
```

**Example output for --instances 3:**
```
Instance  1:  longitude -180.00° to  -60.00°
Instance  2:  longitude  -60.00° to   60.00°
Instance  3:  longitude   60.00° to  180.00°
```

### collect_and_sync.py

Single worker that queries OSM for slopes and lifts within a longitude range and syncs them to the API.

```bash
# Usage
python collect_and_sync.py --instance-id ID --lng-min MIN --lng-max MAX [--dry-run] [--clear]

# Options
--instance-id ID   Unique ID for this worker
--lng-min MIN      Minimum longitude (inclusive)
--lng-max MAX      Maximum longitude (exclusive)
--dry-run          Preview API calls without executing
--clear            Clear all progress files
```

### Progress Tracking

Each worker saves progress to `progress_<instance_id>.json`:

```json
{
  "instance_id": 1,
  "lng_min": -180.0,
  "lng_max": -144.0,
  "last_processed_osm_id": "way/12345678",
  "last_run_timestamp": "2026-02-21T10:30:00Z",
  "slopes_synced": 1500,
  "lifts_synced": 300,
  "slopes_updated": 50,
  "lifts_updated": 10,
  "skipped": []
}
```

On restart, workers resume from `last_processed_osm_id`.

## Data Collection Details

### Slopes

Collected from OSM ways and relations with `piste:type=*` tags.

**OSM tags mapped to API fields:**
- `name` → `name`
- `piste:difficulty` → `difficulty` (mapped: novice→green, easy→green, intermediate→blue, advanced→red, expert→black)
- `piste:grooming` → `grooming_status`
- `lit` → `night_skiing` (boolean)
- `piste:type=racing` → `race_slope` (boolean)
- Full geometry → `slope_path_json` + `direction`

### Lifts

Collected from OSM ways and relations with `aerialway=*` tags.

**OSM tags mapped to API fields:**
- `name` → `name`
- `aerialway` → `lift_type` (mapped: chair_lift→chairlift, drag_lift→draglift, etc.)
- `capacity:hourly` → `capacity_per_hour`
- `seats` / `occupancy` → `seats`
- `bubble` → `bubble` (boolean)
- `heated_seats` → `heated_seats` (boolean)
- Full geometry → start/end coordinates

### Resort Assignment

**Important:** The `resort_id` field is set to `"unassigned"` for all collected data. Resort assignment is a separate future step that will match slopes/lifts to resorts based on geographic proximity.

## Idempotency

The system is fully idempotent:
- Uses OSM ID (e.g., `way/12345`) as unique identifier via `source_entity_id`
- Before inserting, checks if record with same `source_entity_id` exists
- If exists: updates via PUT; if not: creates via POST
- Running twice produces the same database state

## Error Handling

- Bad records are logged and skipped, never crash the worker
- HTTP retries with exponential backoff (3 retries by default)
- Progress saved after each batch
- Skipped records tracked with error reason in progress file

## Wipe Table Tool

Located in `server/wipe_table/`, this Rust binary backs up and deletes all records from a table.

```bash
cd server/wipe_table

# Backup and delete all slopes
cargo run -- --table slopes

# Dry run (backup only)
cargo run -- --table slopes --dry-run

# Custom backup directory
cargo run -- --table lifts --backup-dir ./backups
```

**Safety features:**
- Creates timestamped JSON backup before any deletion
- Verifies backup is non-empty before proceeding
- Requires explicit user confirmation ("y") before deleting
- Dry-run mode for testing

## Troubleshooting

### Workers not starting
- Check that the API server is running on the configured URL
- Verify API key is correct in config.py
- Ensure Python dependencies are installed (`requests`, `urllib3`)

### Overpass API timeouts
- Increase `OVERPASS_TIMEOUT` in config.py
- Reduce `BATCH_SIZE` for smaller Overpass queries
- Consider using a local Overpass instance

### Progress not advancing
- Check `progress_*.json` files for `last_processed_osm_id`
- Clear progress with `--clear` flag to restart from beginning
- Check for skipped records in the `skipped` array

### API rate limiting
- Increase `REQUEST_DELAY` in config.py
- Reduce number of parallel instances
- Use staggered startup in parallel_launcher.py