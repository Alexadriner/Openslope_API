"""
Parallel Instance Launcher for Openslope Data Collection

This script manages multiple parallel worker instances of collect_and_sync.py,
distributing the global longitude range (-180 to +180) across N workers.

Each worker runs independently and processes its assigned longitude segment.
Workers do not block each other and run fully in parallel.

## Usage

    # Launch with default number of instances (4)
    python parallel_launcher.py

    # Launch with specific number of instances
    python parallel_launcher.py --instances 10

    # Launch with dry-run mode (preview without executing)
    python parallel_launcher.py --instances 5 --dry-run

    # Clear all progress files
    python parallel_launcher.py --clear

## How It Works

1. The launcher divides the longitude range [-180, +180] into N equal segments
2. Each worker is assigned one segment and runs independently
3. Workers are started with a small delay between them to prevent API overload
4. The launcher does NOT wait for workers to complete (fire-and-forget)

Example for --instances 3:
    Instance 1: -180 to -60
    Instance 2:  -60 to +60
    Instance 3:  +60 to +180

## Author
Openslope Team

## Version
1.0.0
"""

import argparse
import subprocess
import sys
import time
from pathlib import Path

# Import shared configuration
from config import DEFAULT_INSTANCES, PROGRESS_DIR


# =============================================================================
# Configuration
# =============================================================================

# Delay between worker startups in seconds (prevents API overload)
START_DELAY = 5

# Path to the worker script
WORKER_SCRIPT = Path(__file__).resolve().parent / "collect_and_sync.py"

# Python executable to use
PYTHON = sys.executable


# =============================================================================
# Helper Functions
# =============================================================================


def calculate_longitude_segments(num_instances: int) -> list[tuple[float, float]]:
    """
    Divide the longitude range [-180, +180] into N equal segments.
    
    Args:
        num_instances: Number of segments to create
        
    Returns:
        List of (lng_min, lng_max) tuples for each segment
    """
    total_range = 360.0  # -180 to +180
    segment_size = total_range / num_instances
    
    segments = []
    for i in range(num_instances):
        lng_min = -180.0 + (i * segment_size)
        lng_max = lng_min + segment_size
        segments.append((lng_min, lng_max))
    
    return segments


def clear_all_progress() -> None:
    """Clear all progress files from the progress directory."""
    progress_dir = Path(PROGRESS_DIR)
    if progress_dir.exists():
        for f in progress_dir.glob("progress_*.json"):
            f.unlink()
            print(f"Cleared: {f.name}")
    print("All progress files cleared.")


# =============================================================================
# Main Launcher
# =============================================================================


def launch_workers(num_instances: int, dry_run: bool = False) -> None:
    """
    Launch N parallel worker instances.
    
    Each worker is started as an independent subprocess with its assigned
    longitude range. Workers run fully in parallel.
    
    Args:
        num_instances: Number of worker processes to start
        dry_run: If True, show what would be done without executing
    """
    # Calculate longitude segments
    segments = calculate_longitude_segments(num_instances)
    
    print("=" * 70)
    print("Openslope Data Collection - Parallel Launcher")
    print("=" * 70)
    print(f"Number of instances: {num_instances}")
    if dry_run:
        print("*** DRY RUN MODE - No workers will be started ***")
    print("=" * 70)
    print("\nInstance assignments:")
    print("-" * 70)
    
    for i, (lng_min, lng_max) in enumerate(segments, 1):
        print(f"  Instance {i:2d}:  longitude {lng_min:7.2f}° to {lng_max:7.2f}°")
    
    print("-" * 70)
    
    if dry_run:
        print("\nDry run complete. No workers were started.")
        return
    
    print(f"\nStarting {num_instances} workers...")
    print(f"Python: {PYTHON}")
    print(f"Worker script: {WORKER_SCRIPT}")
    print()
    
    processes = []
    
    for i, (lng_min, lng_max) in enumerate(segments, 1):
        instance_id = i  # Use 1-based instance IDs
        
        cmd = [
            PYTHON,
            str(WORKER_SCRIPT),
            "--instance-id", str(instance_id),
            "--lng-min", str(lng_min),
            "--lng-max", str(lng_max),
        ]
        
        print(f"-> Starting worker {i}/{num_instances} (PID will be assigned)...")
        
        try:
            # Start worker as independent subprocess
            # Do NOT wait for completion - fire and forget
            process = subprocess.Popen(
                cmd,
                cwd=str(Path(__file__).resolve().parent),
                stdout=subprocess.DEVNULL,
                stderr=subprocess.DEVNULL,
            )
            processes.append(process)
            print(f"   Worker {i} started with PID {process.pid}")
            
            # Small delay to prevent API overload during startup
            if i < num_instances:
                time.sleep(START_DELAY)
                
        except Exception as e:
            print(f"   ERROR starting worker {i}: {e}")
    
    print()
    print("=" * 70)
    print(f"All {num_instances} workers have been started.")
    print("Workers are running independently in the background.")
    print("Check progress files (progress_*.json) for status updates.")
    print("=" * 70)


# =============================================================================
# CLI Entry Point
# =============================================================================


def main():
    parser = argparse.ArgumentParser(
        description="Openslope Parallel Data Collection Launcher",
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog="""
Examples:
  # Launch with default instances (4)
  python parallel_launcher.py

  # Launch with 10 instances
  python parallel_launcher.py --instances 10

  # Dry run to preview assignments
  python parallel_launcher.py --instances 5 --dry-run

  # Clear all progress files
  python parallel_launcher.py --clear
        """,
    )
    parser.add_argument(
        "--instances",
        type=int,
        default=DEFAULT_INSTANCES,
        help=f"Number of parallel worker instances (default: {DEFAULT_INSTANCES})",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Preview worker assignments without starting them",
    )
    parser.add_argument(
        "--clear",
        action="store_true",
        help="Clear all progress files and exit",
    )
    
    args = parser.parse_args()
    
    if args.clear:
        clear_all_progress()
        return
    
    # Validate instances count
    if args.instances < 1:
        print("Error: Number of instances must be at least 1")
        sys.exit(1)
    
    if args.instances > 36:
        print("Warning: More than 36 instances may cause issues with longitude precision")
        response = input("Continue anyway? [y/N]: ")
        if response.lower() != "y":
            print("Aborted.")
            sys.exit(0)
    
    launch_workers(args.instances, args.dry_run)


if __name__ == "__main__":
    main()