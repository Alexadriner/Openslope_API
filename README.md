# 🎿 Openslope API

> ⚠️ Disclaimer
>
> This project is **not intended for public use or deployment**. The backend relies on a private database that is not publicly accessible. The repository exists solely to showcase the project's architecture and code.

A full-stack application that aggregates, cleans, and serves ski resort data through a web interface with an interactive map. The project combines a Python-based data scraper, a Rust-powered backend server, and a JavaScript/HTML frontend to provide information about ski resorts.

Data is sourced from [OpenSkiMap](https://openskimap.org/).

---

## 📁 Project Structure

```
Openslope_API/
├── scripts/              # Utility and helper scripts
├── server/               # Rust backend server (REST API)
├── testing/leaflet/      # Leaflet.js map testing/prototyping
├── website/              # Frontend (HTML, CSS, JavaScript)
├── ski_scraper.py        # Web scraper for ski resort data
├── cleanup_ski_data.py   # Data cleaning and normalization
├── cleanup_launcher.py   # Launcher for the cleanup pipeline
├── launcher.py           # Main application launcher
├── ski-resorts.csv       # Ski resort dataset
└── tmp_*.json / *.html   # Temporary scraped data files
```

---

## 🛠️ Tech Stack

| Layer | Technology |
| --- | --- |
| Backend | Rust |
| Scraper | Python |
| Frontend | HTML, CSS, JavaScript |
| Map | Leaflet.js |

---

## 🚀 Getting Started

### Prerequisites

* [Python 3.x](https://www.python.org/downloads/)
* [Rust & Cargo](https://www.rust-lang.org/tools/install)

### Installation

1. **Clone the repository:**

   ```
   git clone https://github.com/Alexadriner/Openslope_API.git
   cd Openslope_API
   ```
2. **Install Python dependencies:**

   ```
   pip install -r requirements.txt
   ```
3. **Build the Rust server:**

   ```
   cd server
   cargo build --release
   ```

---

## ▶️ Usage

### 1. Data Collection from OpenStreetMap

The unified data collection system fetches all ski slopes and lifts worldwide from OpenStreetMap via the Overpass API and syncs them to the Openslope REST API.

#### Running the Parallel Launcher (Recommended)

`parallel_launcher.py` starts multiple parallel worker instances to distribute the workload:

```bash
# Launch with default number of instances (4)
python scripts/data_tools/parallel_launcher.py

# Launch with specific number of instances
python scripts/data_tools/parallel_launcher.py --instances 10

# Dry run to preview worker assignments
python scripts/data_tools/parallel_launcher.py --dry-run

# Clear all progress files
python scripts/data_tools/parallel_launcher.py --clear
```

Each worker processes a specific longitude range and runs independently. Progress is tracked in `progress_<instance_id>.json` files.

#### Running a Single Worker

```bash
# Run worker with specific longitude range
python scripts/data_tools/collect_and_sync.py --instance-id 1 --lng-min -180 --lng-max -144

# Dry run mode (preview without executing)
python scripts/data_tools/collect_and_sync.py --instance-id 1 --lng-min -180 --lng-max -144 --dry-run

# Clear progress files
python scripts/data_tools/collect_and_sync.py --clear
```

#### Configuration

Edit `scripts/data_tools/config.py` to customize:
- API base URL and authentication key
- Overpass API endpoint
- Batch size and retry settings
- Request delay for rate limiting

### 2. Wipe Table Tool

The `wipe_table` Rust binary creates a JSON backup of all records in a table and then deletes them via the REST API.

```bash
cd server/wipe_table

# Backup and delete all slopes
cargo run -- --table slopes

# Backup and delete all lifts with custom backup directory
cargo run -- --table lifts --backup-dir ./my_backups

# Dry run (create backup only, no deletion)
cargo run -- --table slopes --dry-run
```

**Safety features:**
- All records are backed up to a timestamped JSON file before deletion
- Backup file is verified to be non-empty before proceeding
- User confirmation is required before any deletion

### 3. Legacy Scrapers

The original resort scrapers are still available for scraping live status data from resort websites:

```bash
# Launch 20 parallel instances
python launcher.py

# Single instance
python ski_scraper.py
```

### 4. Data Cleanup

```bash
# Launch 20 parallel cleanup instances
python cleanup_launcher.py

# Single cleanup instance
python cleanup_ski_data.py
```

### 5. Start the Website

Navigate to the `website` folder and start the dev server:

```bash
cd website
npm run dev
```

---

## 🗺️ Features

* **Data Scraping** – Automatically fetches ski resort information (lift status, snow conditions, etc.) from external sources
* **Data Cleaning** – Normalizes and deduplicates resort data into a consistent CSV format
* **REST API** – High-performance Rust backend serves resort data as JSON
* **Interactive Map** – Leaflet.js-powered frontend visualizes resorts geographically
* **Web Interface** – Browse and explore ski resort details via a web UI

---

## 🔌 API Reference

The REST API is served by the Rust backend. All endpoints return JSON.

### Resorts

#### `GET /resorts`

Returns a list of all ski resorts.

**Example Response:**
```json
[
  {
    "id": "kreuzberg",
    "name": "Kreuzberg",
    "geography": {
      "continent": null,
      "country": "Germany",
      "region": "Rhön-Grabfeld",
      "coordinates": {
        "latitude": 50.3456,
        "longitude": 10.0458
      }
    },
    "altitude": {
      "village_m": null,
      "min_m": null,
      "max_m": null
    },
    "ski_area": {
      "name": "Kreuzberg",
      "area_type": "alpine"
    },
    "sources": {
      "official_website": "https://www.skilifte-kreuzberg.de",
      "lift_status_url": "https://www.skilifte-kreuzberg.de/",
      "slope_status_url": "https://www.skilifte-kreuzberg.de/",
      "snow_report_url": "https://www.skilifte-kreuzberg.de/",
      "weather_url": "https://www.skilifte-kreuzberg.de/",
      "status_provider": "skilifte_kreuzberg_homepage"
    },
    "live_status": {
      "last_scraped_at": "2026-02-21T00:00:00Z",
      "lifts_open_count": 0,
      "slopes_open_count": null,
      "snow_depth_valley_cm": 20,
      "snow_depth_mountain_cm": 25,
      "new_snow_24h_cm": null,
      "temperature_valley_c": 8,
      "temperature_mountain_c": null
    }
  }
]
```

---

#### `GET /resorts/:resort_id`

Returns a single resort by its ID.

**URL Parameter:** `resort_id` — the resort's unique string identifier (e.g. `kreuzberg`)

**Example Request:** `GET /resorts/kreuzberg`

**Example Response:**
```json
{
  "id": "kreuzberg",
  "name": "Kreuzberg",
  "geography": {
    "continent": null,
    "country": "Germany",
    "region": "Rhön-Grabfeld",
    "coordinates": {
      "latitude": 50.3456,
      "longitude": 10.0458
    }
  },
  "altitude": {
    "village_m": null,
    "min_m": null,
    "max_m": null
  },
  "ski_area": {
    "name": "Kreuzberg",
    "area_type": "alpine"
  },
  "sources": {
    "official_website": "https://www.skilifte-kreuzberg.de",
    "lift_status_url": "https://www.skilifte-kreuzberg.de/",
    "slope_status_url": "https://www.skilifte-kreuzberg.de/",
    "snow_report_url": "https://www.skilifte-kreuzberg.de/",
    "weather_url": "https://www.skilifte-kreuzberg.de/",
    "status_provider": "skilifte_kreuzberg_homepage"
  },
  "live_status": {
    "last_scraped_at": "2026-02-21T00:00:00Z",
    "lifts_open_count": 0,
    "slopes_open_count": null,
    "snow_depth_valley_cm": 20,
    "snow_depth_mountain_cm": 25,
    "new_snow_24h_cm": null,
    "temperature_valley_c": 8,
    "temperature_mountain_c": null
  }
}
```

---

### Slopes

#### `GET /slopes`

Returns a list of all slopes across all resorts.

**Example Response:**
```json
[
  {
    "id": 328375,
    "resort_id": "kreuzberg",
    "name": "Blick 2",
    "display": {
      "normalized_name": null,
      "difficulty": "blue"
    },
    "geometry": {
      "start": {
        "latitude": 50.375992,
        "longitude": 9.982904
      },
      "end": {
        "latitude": 50.379936,
        "longitude": 9.98445
      },
      "path": null,
      "direction": null
    },
    "specs": {
      "length_m": null,
      "vertical_drop_m": null,
      "average_gradient": null,
      "max_gradient": null,
      "snowmaking": false,
      "night_skiing": false,
      "family_friendly": false,
      "race_slope": false
    },
    "source": {
      "system": "osm",
      "entity_id": null,
      "source_url": null
    },
    "status": {
      "operational_status": "unknown",
      "grooming_status": "unknown",
      "note": null,
      "updated_at": null
    }
  }
]
```

---

#### `GET /slopes/:slope_id`

Returns a single slope by its numeric ID.

**URL Parameter:** `slope_id` — the slope's unique numeric identifier (e.g. `328375`)

**Example Request:** `GET /slopes/328375`

**Example Response:**
```json
{
  "id": 328375,
  "resort_id": "kreuzberg",
  "name": "Blick 2",
  "display": {
    "normalized_name": null,
    "difficulty": "blue"
  },
  "geometry": {
    "start": {
      "latitude": 50.375992,
      "longitude": 9.982904
    },
    "end": {
      "latitude": 50.379936,
      "longitude": 9.98445
    },
    "path": null,
    "direction": null
  },
  "specs": {
    "length_m": null,
    "vertical_drop_m": null,
    "average_gradient": null,
    "max_gradient": null,
    "snowmaking": false,
    "night_skiing": false,
    "family_friendly": false,
    "race_slope": false
  },
  "source": {
    "system": "osm",
    "entity_id": null,
    "source_url": null
  },
  "status": {
    "operational_status": "unknown",
    "grooming_status": "unknown",
    "note": null,
    "updated_at": null
  }
}
```

---

#### `GET /slopes/by_resort/:resort_id`

Returns all slopes belonging to a specific resort.

**URL Parameter:** `resort_id` — the resort's unique string identifier (e.g. `kreuzberg`)

**Example Request:** `GET /slopes/by_resort/kreuzberg`

**Example Response:** Array of slope objects — same structure as `GET /slopes`.

---

### Lifts

#### `GET /lifts`

Returns a list of all lifts across all resorts.

**Example Response:**
```json
[
  {
    "id": 82805,
    "resort_id": "kreuzberg",
    "name": "Blicklift",
    "display": {
      "normalized_name": "blicklift",
      "lift_type": "draglift"
    },
    "geometry": {
      "start": {
        "latitude": 50.38006,
        "longitude": 9.98445
      },
      "end": {
        "latitude": 50.37582,
        "longitude": 9.98287
      }
    },
    "specs": {
      "capacity_per_hour": null,
      "seats": null,
      "bubble": false,
      "heated_seats": false,
      "year_built": null,
      "altitude_start_m": 577,
      "altitude_end_m": 881
    },
    "source": {
      "system": "osm",
      "entity_id": "blicklift",
      "source_url": "https://www.skilifte-kreuzberg.de/"
    },
    "status": {
      "operational_status": "closed",
      "note": "geschlossen",
      "planned_open_time": null,
      "planned_close_time": null,
      "updated_at": null
    }
  }
]
```

---

#### `GET /lifts/:lift_id`

Returns a single lift by its numeric ID.

**URL Parameter:** `lift_id` — the lift's unique numeric identifier (e.g. `82805`)

**Example Request:** `GET /lifts/82805`

**Example Response:**
```json
{
  "id": 82805,
  "resort_id": "kreuzberg",
  "name": "Blicklift",
  "display": {
    "normalized_name": "blicklift",
    "lift_type": "draglift"
  },
  "geometry": {
    "start": {
      "latitude": 50.38006,
      "longitude": 9.98445
    },
    "end": {
      "latitude": 50.37582,
      "longitude": 9.98287
    }
  },
  "specs": {
    "capacity_per_hour": null,
    "seats": null,
    "bubble": false,
    "heated_seats": false,
    "year_built": null,
    "altitude_start_m": 577,
    "altitude_end_m": 881
  },
  "source": {
    "system": "osm",
    "entity_id": "blicklift",
    "source_url": "https://www.skilifte-kreuzberg.de/"
  },
  "status": {
    "operational_status": "closed",
    "note": "geschlossen",
    "planned_open_time": null,
    "planned_close_time": null,
    "updated_at": null
  }
}
```

---

#### `GET /lifts/by_resort/:resort_id`

Returns all lifts belonging to a specific resort.

**URL Parameter:** `resort_id` — the resort's unique string identifier (e.g. `kreuzberg`)

**Example Request:** `GET /lifts/by_resort/kreuzberg`

**Example Response:** Array of lift objects — same structure as `GET /lifts`.

---

## 📄 Data

Resort data is sourced from [OpenSkiMap](https://openskimap.org/), a community-driven, open-source project based on OpenStreetMap data. The scraper fetches this data, which is then cleaned, normalized, and stored in a private database. The `ski-resorts.csv` in this repository serves as an example snapshot. Temporary files (prefixed with `tmp_`) are generated during the scraping process.

For some resorts, live status data (lift status, snow conditions, etc.) is additionally sourced directly from the resort's own website or API. In these cases, the data source is indicated in the `sources.status_provider` field of the resort object as well as in the corresponding API response.

---

## ⚠️ Disclaimer

This project is **not intended for public use or deployment**. The backend relies on a private database that is not publicly accessible. The repository exists solely to showcase the project's architecture and code.
