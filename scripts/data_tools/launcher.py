import subprocess
import sys
import time
import argparse
from pathlib import Path
import shutil

# =========================
# CONFIG
# =========================
NUM_WORKERS = 10
START_DELAY = 15
SCRIPT_PATH = Path(__file__).resolve().parent / "ski_scraper.py"
BASE_DIR = Path(__file__).resolve().parents[2]

# =========================
# PYTHON EXECUTABLE
# =========================
PYTHON = sys.executable


def clear_launcher_progress(worker_ids=None):
    """
    Clear progress files for the launcher.
    If worker_ids is None, clears all progress files.
    If worker_ids is provided, clears only those worker progress files.
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

    for p in processes:
        p.wait()

    print("\nAlle Worker beendet.")


if __name__ == "__main__":
    main()
