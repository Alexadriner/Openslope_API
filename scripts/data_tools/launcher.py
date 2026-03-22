"""
Ski Scraper Launcher

This script launches multiple worker processes for the ski scraper system,
enabling parallel processing of ski resort data collection and processing.

The launcher is designed to work with the collect_geojson.py script and
distributes work across multiple workers to improve performance and
handle large datasets efficiently.

## Features

- **Parallel Processing**: Launches multiple worker processes simultaneously
- **Load Distribution**: Distributes work evenly across workers
- **Progress Management**: Includes progress clearing functionality
- **Debug Support**: Special debug mode for testing launcher logic
- **Graceful Startup**: Staggered worker startup to prevent API overload

## Usage

### Basic Usage
```bash
python scripts/data_tools/launcher.py
```

### Debug Mode
```bash
# Start workers and immediately exit to test launcher storage logic
python scripts/data_tools/launcher.py --save_debug
```

### Progress Management
```bash
# Clear all progress files
python scripts/data_tools/launcher.py --clear

# Clear specific worker progress files
python scripts/data_tools/launcher.py --clear 0 1 2
```

## Configuration

The launcher includes several configurable parameters:

- `NUM_WORKERS`: Number of parallel worker processes (default: 10)
- `START_DELAY`: Delay between worker startups in seconds (default: 15)
- `SCRIPT_PATH`: Path to the worker script (ski_scraper.py)
- `BASE_DIR`: Project root directory
- `PYTHON`: Python executable to use

## Worker Distribution

The launcher distributes work by:
1. Assigning each worker a unique ID (0 to NUM_WORKERS-1)
2. Passing the total number of workers to each process
3. Using modulo arithmetic in the worker script to divide work

For example, with 10 workers processing 100 resorts:
- Worker 0 processes resorts 0, 10, 20, 30, 40, 50, 60, 70, 80, 90
- Worker 1 processes resorts 1, 11, 21, 31, 41, 51, 61, 71, 81, 91
- And so on...

## Command Line Arguments

- `--clear [worker_ids...]`: Clear progress files (no args = all, specific IDs = those workers)
- `--save_debug`: Enable debug mode that starts workers but immediately exits

## Error Handling

The launcher includes:
- **Process Management**: Tracks all worker processes
- **Graceful Waiting**: Waits for all workers to complete
- **Startup Delays**: Prevents overwhelming APIs with simultaneous requests
- **Progress Cleanup**: Safe removal of progress files

## Integration

This launcher works with:
- `collect_geojson.py`: Main data collection script
- `ski_scraper.py`: Worker script (referenced in SCRIPT_PATH)
- Progress tracking system in checkpoints directory
- Single resort file management

## Performance Considerations

- **Worker Count**: Adjust NUM_WORKERS based on system resources
- **Start Delay**: Prevents API rate limiting during startup
- **Memory Usage**: Each worker loads its own data, monitor system memory
- **Network Load**: Staggered startup reduces concurrent API requests

## Author
OpenSlope Team

## Version
1.0.0
"""

import subprocess
import sys
import time
import argparse
from pathlib import Path
import shutil

# =========================
# CONFIGURATION
# =========================
# Number of parallel worker processes
NUM_WORKERS = 10

# Delay between worker startups in seconds (prevents API overload)
START_DELAY = 15

# Path to the worker script
SCRIPT_PATH = Path(__file__).resolve().parent / "ski_scraper.py"

# Project root directory
BASE_DIR = Path(__file__).resolve().parents[2]

# =========================
# PYTHON EXECUTABLE
# =========================
# Use the same Python executable that's running this launcher
PYTHON = sys.executable


def clear_launcher_progress(worker_ids=None):
    """
    Clear progress files for the launcher.
    
    Args:
        worker_ids (list, optional): List of worker IDs to clear. 
                                   If None, clears all progress files.
    """
    checkpoint_dir = BASE_DIR / "checkpoints" / "collect_geojson"
    
    if worker_ids is None:
        # Clear all progress files
        if checkpoint_dir.exists():
            shutil.rmtree(checkpoint_dir)
            print("Cleared all launcher progress files")
        else:
            print("No launcher progress directory found")
    else:
        # Clear specific worker progress files
        for worker_id in worker_ids:
            progress_file = checkpoint_dir / f"worker_{worker_id}_progress.json"
            if progress_file.exists():
                progress_file.unlink()
                print(f"Cleared launcher progress for worker {worker_id}")
            else:
                print(f"No launcher progress file found for worker {worker_id}")
    
    # Also clear the single resort file
    single_resort_file = BASE_DIR / "current_resort_geojson.json"
    if single_resort_file.exists():
        single_resort_file.unlink()
        print("Cleared single resort file")


def main():
    """
    Main launcher function that parses arguments and starts worker processes.
    """
    parser = argparse.ArgumentParser(description="Launcher for ski scraper workers")
    parser.add_argument("--clear", nargs="*", type=int, help="Clear progress files. If no IDs provided, clears all progress files. If IDs provided, clears only those worker progress files.")
    parser.add_argument("--save_debug", action="store_true", help="Enable save debug mode - start workers but immediately exit to test launcher storage logic without processing")
    
    args = parser.parse_args()
    
    # Handle clear command first
    if args.clear is not None:
        if len(args.clear) == 0:
            # Clear all progress files
            clear_launcher_progress()
            print("All launcher progress files cleared. You can now start from scratch.")
        else:
            # Clear specific worker progress files
            clear_launcher_progress(args.clear)
            print(f"Launcher progress files cleared for workers: {args.clear}")
        sys.exit(0)
    
    # Handle save debug mode
    if args.save_debug:
        print("SAVE DEBUG MODE ENABLED: Starting workers and then immediately exiting to test launcher storage logic")
        
        processes = []
        print(f"Starte {NUM_WORKERS} Worker...\n")

        for i in range(NUM_WORKERS):
            print(f"-> Starte Worker {i}")

            cmd = [
                PYTHON,
                str(SCRIPT_PATH),
                str(i),
                str(NUM_WORKERS),
            ]

            p = subprocess.Popen(cmd, cwd=str(BASE_DIR))
            processes.append(p)

            # Kleine Pause, um API-Last zu verteilen.
            time.sleep(START_DELAY)

        print("\nAlle Worker gestartet.")
        print("SAVE DEBUG MODE: Exiting immediately to test launcher storage logic without processing")
        
        # Don't wait for processes to complete - exit immediately
        return
    
    # Normal operation: start workers and wait for completion
    processes = []

    print(f"Starte {NUM_WORKERS} Worker...\n")

    for i in range(NUM_WORKERS):
        print(f"-> Starte Worker {i}")

        cmd = [
            PYTHON,
            str(SCRIPT_PATH),
            str(i),
            str(NUM_WORKERS),
        ]

        p = subprocess.Popen(cmd, cwd=str(BASE_DIR))
        processes.append(p)

        # Kleine Pause, um API-Last zu verteilen.
        time.sleep(START_DELAY)

    print("\nAlle Worker gestartet.")

    # Wait for all workers to complete
    for p in processes:
        p.wait()

    print("\nAlle Worker beendet.")


if __name__ == "__main__":
    main()