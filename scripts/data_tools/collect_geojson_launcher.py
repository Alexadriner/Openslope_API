#!/usr/bin/env python3
"""
Launcher for collect_geojson.py with 30 parallel instances.

Usage:
  python collect_geojson_launcher.py [--workers 30] [--start-delay 1] [--reset-progress]

Each instance processes a subset of ski areas based on its worker index:
  - Worker 0:  Ski areas 0, 30, 60, 90, ...
  - Worker 1:  Ski areas 1, 31, 61, 91, ...
  - ...
  - Worker 29: Ski areas 29, 59, 89, 119, ...

Progress is tracked per worker in JSON files under checkpoints/collect_geojson/.
The launcher resumes from the last checkpoint per worker.
"""

import argparse
import json
import logging
import os
import subprocess
import sys
import time
from datetime import datetime, timezone
from pathlib import Path


# =========================
# CONFIGURATION
# =========================
NUM_WORKERS = 30
START_DELAY = 1
SCRIPT_PATH = Path(__file__).resolve().parent / "collect_geojson.py"
BASE_DIR = Path(__file__).resolve().parents[2]
CHECKPOINT_DIR = BASE_DIR / "checkpoints" / "collect_geojson"
LOG_DIR = BASE_DIR / "logs" / "collect_geojson_launcher"
PROGRESS_FILE = CHECKPOINT_DIR / "launcher_progress.json"

os.makedirs(LOG_DIR, exist_ok=True)


# =========================
# LOGGING SETUP
# =========================
def setup_logging():
    """Configure logging for the launcher."""
    timestamp = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
    log_file = LOG_DIR / f"launcher_{timestamp}.log"
    
    logger = logging.getLogger("collect_geojson_launcher")
    logger.setLevel(logging.INFO)
    
    file_handler = logging.FileHandler(log_file, encoding="utf-8")
    file_handler.setLevel(logging.INFO)
    
    console_handler = logging.StreamHandler()
    console_handler.setLevel(logging.INFO)
    
    formatter = logging.Formatter("%(asctime)s | %(levelname)s | %(message)s")
    file_handler.setFormatter(formatter)
    console_handler.setFormatter(formatter)
    
    logger.addHandler(file_handler)
    logger.addHandler(console_handler)
    
    return logger


launcher_logger = setup_logging()


def log_info(msg):
    print(msg)
    launcher_logger.info(msg)


def log_warning(msg):
    print(f"WARNING: {msg}")
    launcher_logger.warning(msg)


def log_error(msg):
    print(f"ERROR: {msg}")
    launcher_logger.error(msg)


# =========================
# PYTHON EXECUTABLE
# =========================
PYTHON = sys.executable


# =========================
# PROGRESS MANAGEMENT
# =========================
def write_checkpoint(data, path):
    """Schreibt Checkpoint atomar, um korrupte Dateien zu vermeiden."""
    tmp_path = Path(f"{path}.tmp")

    with open(tmp_path, "w", encoding="utf-8") as f:
        json.dump(data, f)
        f.flush()
        os.fsync(f.fileno())

    os.replace(tmp_path, path)


def save_worker_progress(worker_id, processed_resorts):
    """Save progress for a specific worker."""
    CHECKPOINT_DIR.mkdir(parents=True, exist_ok=True)
    progress_file = CHECKPOINT_DIR / f"worker_{worker_id}_progress.json"
    payload = {
        "worker_id": worker_id,
        "processed_resorts": processed_resorts,
        "updated_at": datetime.now(timezone.utc).isoformat(),
    }
    write_checkpoint(payload, progress_file)


def load_worker_progress(worker_id):
    """Load progress for a specific worker."""
    progress_file = CHECKPOINT_DIR / f"worker_{worker_id}_progress.json"
    try:
        with open(progress_file, "r", encoding="utf-8") as f:
            raw = f.read().strip()
            if not raw:
                return []
            data = json.loads(raw)
            return data.get("processed_resorts", [])
    except (FileNotFoundError, json.JSONDecodeError):
        return []


def save_launcher_progress(stage, status="running", last_value=None):
    """Save overall launcher progress."""
    CHECKPOINT_DIR.mkdir(parents=True, exist_ok=True)
    payload = {
        "stage": stage,
        "status": status,
        "updated_at": datetime.now(timezone.utc).isoformat(),
        "last_value": last_value,
    }
    write_checkpoint(payload, PROGRESS_FILE)


def load_launcher_progress():
    """Load overall launcher progress."""
    try:
        with open(PROGRESS_FILE, "r", encoding="utf-8") as f:
            raw = f.read().strip()
            if not raw:
                return "running"
            data = json.loads(raw)
            stage = str(data.get("stage", "")).strip()
            return stage if stage else "running"
    except (FileNotFoundError, json.JSONDecodeError):
        return "running"


# =========================
# MAIN LAUNCHER LOGIC
# =========================
def run_collect_geojson_worker(worker_id, total_workers, start_delay):
    """Run a single collect_geojson worker instance."""
    log_info(f"Starting worker {worker_id}/{total_workers}...")
    time.sleep(start_delay * worker_id)
    
    cmd = [
        PYTHON,
        str(SCRIPT_PATH),
        str(worker_id),
        str(total_workers),
    ]
    
    return subprocess.call(cmd, cwd=str(BASE_DIR))


def main():
    parser = argparse.ArgumentParser(
        description="Launch parallel collect_geojson instances with progress tracking."
    )
    parser.add_argument("--workers", type=int, default=10)  # Reduced from 30 to 10
    parser.add_argument("--start-delay", type=float, default=3.0)  # Increased delay
    parser.add_argument("--reset-progress", action="store_true")
    parser.add_argument("--save-debug", action="store_true", help="Enable save debug mode - start workers but immediately exit to test launcher storage logic without processing")
    args = parser.parse_args()

    log_info("=" * 60)
    log_info("Collect GeoJSON Launcher gestartet")
    log_info("=" * 60)
    log_info(f"Parameter: workers={args.workers}, start_delay={args.start_delay}s")
    log_info(f"Progress-Verzeichnis: {CHECKPOINT_DIR}")
    log_info("=" * 60)

    # Handle save debug mode
    if args.save_debug:
        log_info("SAVE DEBUG MODE ENABLED: Starting workers and then immediately exiting to test launcher storage logic")
        
        if args.reset_progress:
            # Clean up all progress files
            for f in CHECKPOINT_DIR.glob("*.json"):
                try:
                    os.remove(f)
                    log_info(f"  -> Gelöscht: {f.name}")
                except OSError as e:
                    log_warning(f"  -> Konnte {f.name} nicht löschen: {e}")
            log_info("Fortschritt zurückgesetzt.")

        # Load processed resorts per worker
        worker_progress = {}
        for wid in range(args.workers):
            worker_progress[wid] = set(load_worker_progress(wid))

        # Determine which workers need to run
        workers_to_run = []
        for wid in range(args.workers):
            workers_to_run.append(wid)

        log_info(f"Workers to run: {workers_to_run}")

        # Run workers
        processes = []
        for wid in workers_to_run:
            log_info(f"Starting worker {wid}...")
            p = subprocess.Popen(
                [
                    PYTHON,
                    str(SCRIPT_PATH),
                    str(wid),
                    str(args.workers),
                    "--save_debug",
                ],
                cwd=str(BASE_DIR),
            )
            processes.append((wid, p))
            time.sleep(args.start_delay)

        log_info("\nAlle Worker gestartet.")
        log_info("SAVE DEBUG MODE: Exiting immediately to test launcher storage logic without processing")
        
        # Don't wait for processes to complete - exit immediately
        return

    if args.reset_progress:
        # Clean up all progress files
        for f in CHECKPOINT_DIR.glob("*.json"):
            try:
                os.remove(f)
                log_info(f"  -> Gelöscht: {f.name}")
            except OSError as e:
                log_warning(f"  -> Konnte {f.name} nicht löschen: {e}")
        log_info("Fortschritt zurückgesetzt.")

    # Load processed resorts per worker
    worker_progress = {}
    for wid in range(args.workers):
        worker_progress[wid] = set(load_worker_progress(wid))

    # Determine which workers need to run
    workers_to_run = []
    for wid in range(args.workers):
        # For now, we assume each worker should run its assigned ski areas
        # In a real implementation, you would load the full list of ski areas
        # and filter by worker_id % total_workers == worker_index
        workers_to_run.append(wid)

    log_info(f"Workers to run: {workers_to_run}")

    # Run workers
    processes = []
    for wid in workers_to_run:
        log_info(f"Starting worker {wid}...")
        p = subprocess.Popen(
            [
                PYTHON,
                str(SCRIPT_PATH),
                str(wid),
                str(args.workers),
            ],
            cwd=str(BASE_DIR),
        )
        processes.append((wid, p))
        time.sleep(args.start_delay)

    # Wait for all workers to complete
    log_info("Warte auf Abschluss aller Worker...")
    failed_workers = []
    for wid, p in processes:
        code = p.wait()
        if code != 0:
            failed_workers.append((wid, code))
            log_error(f"Worker {wid} failed with exit code {code}")
        else:
            log_info(f"Worker {wid} completed successfully")

    if failed_workers:
        log_error(f"{len(failed_workers)} Worker(s) failed. Exiting.")
        sys.exit(1)

    log_info("=" * 60)
    log_info("Collect GeoJSON Launcher abgeschlossen!")
    log_info("=" * 60)
    
    # Check for unprocessed slopes with auto-generated names
    log_info("Überprüfe unbenannte Pisten (auto-generierte Namen)...")
    check_unnamed_slopes()


def check_unnamed_slopes():
    """Check for slopes with auto-generated names that haven't been processed yet."""
    try:
        # Load all slopes from API
        slopes_endpoint = "http://localhost:8080/slopes"
        headers = {
            "Content-Type": "application/json",
            "Authorization": "Bearer R3StTY4OfadeFJZurXdZ1pZMVbWB3zWuL6FnuPGIbvA"
        }
        
        import requests
        response = requests.get(slopes_endpoint, headers=headers, timeout=30)
        response.raise_for_status()
        slopes = response.json()
        
        # Filter slopes with auto-generated names
        auto_generated_pattern = r'^(green|blue|red|black)\s+slope\s*\[\s*-?\d+(\.\d+)?\s*,\s*-?\d+(\.\d+)?\s*\]$'
        import re
        
        unnamed_slopes = []
        for slope in slopes:
            name = slope.get("name", "")
            if name and re.match(auto_generated_pattern, name.lower().strip()):
                unnamed_slopes.append({
                    "id": slope.get("id"),
                    "name": name,
                    "resort_id": slope.get("resort_id"),
                    "difficulty": slope.get("difficulty", "unknown")
                })
        
        if unnamed_slopes:
            log_info(f"Found {len(unnamed_slopes)} slopes with auto-generated names:")
            for slope in unnamed_slopes[:10]:  # Show first 10
                log_info(f"  - {slope['name']} (ID: {slope['id']}, Resort: {slope['resort_id']})")
            if len(unnamed_slopes) > 10:
                log_info(f"  ... and {len(unnamed_slopes) - 10} more")
        else:
            log_info("No slopes with auto-generated names found.")
            
    except Exception as e:
        log_error(f"Error checking unnamed slopes: {e}")


if __name__ == "__main__":
    main()
