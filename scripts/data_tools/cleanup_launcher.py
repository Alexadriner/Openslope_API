import subprocess
import sys
import time
import argparse
from pathlib import Path

# =========================
# CONFIG
# =========================
NUM_WORKERS = 10
START_DELAY = 2
SCRIPT_PATH = Path(__file__).resolve().parent / "cleanup_ski_data.py"
REASSIGN_SCRIPT_PATH = Path(__file__).resolve().parent / "reassign_entities_by_resort_cluster.py"
UPDATE_RESORT_COORDS_SCRIPT_PATH = Path(__file__).resolve().parent / "update_resort_coordinates.py"
MERGE_SCRIPT_PATH = Path(__file__).resolve().parent / "merge_similar_slopes.py"
BASE_DIR = Path(__file__).resolve().parents[2]

# =========================
# PYTHON EXECUTABLE
# =========================
PYTHON = sys.executable


def run_cleanup_workers(num_workers, start_delay):
    processes = []

    print(f"Starte {num_workers} Cleanup-Worker...\n")

    for i in range(num_workers):
        print(f"-> Starte Cleanup-Worker {i}")

        cmd = [
            PYTHON,
            str(SCRIPT_PATH),
            str(i),
            str(num_workers),
        ]

        p = subprocess.Popen(cmd, cwd=str(BASE_DIR))
        processes.append(p)
        time.sleep(start_delay)

    print("\nAlle Cleanup-Worker gestartet.")

    rc = 0
    for p in processes:
        code = p.wait()
        if code != 0 and rc == 0:
            rc = code

    print("\nAlle Cleanup-Worker beendet.")
    return rc


def run_cluster_reassign(cluster_km, switch_margin_m):
    print("\nStarte Cluster-Reassignment (eindeutige Resort-Zuordnung)...")
    cmd = [
        PYTHON,
        str(REASSIGN_SCRIPT_PATH),
        "--cluster-km",
        str(cluster_km),
        "--switch-margin-m",
        str(switch_margin_m),
    ]
    return subprocess.call(cmd, cwd=str(BASE_DIR))


def run_resort_coordinate_update():
    print("\nStarte Resort-Koordinaten-Update (OSM -> Zentrum -> unveraendert)...")
    cmd = [
        PYTHON,
        str(UPDATE_RESORT_COORDS_SCRIPT_PATH),
    ]
    return subprocess.call(cmd, cwd=str(BASE_DIR))


def run_merge_similar_slopes(merge_distance_m):
    print("\nStarte Merge Similar Slopes...")
    cmd = [
        PYTHON,
        str(MERGE_SCRIPT_PATH),
        "--distance-m",
        str(merge_distance_m),
        "--apply",
    ]
    return subprocess.call(cmd, cwd=str(BASE_DIR))


def main():
    parser = argparse.ArgumentParser(
        description="Run cleanup workers, then cluster reassignment and slope merge."
    )
    parser.add_argument("--workers", type=int, default=NUM_WORKERS)
    parser.add_argument("--start-delay", type=float, default=START_DELAY)
    parser.add_argument("--cluster-km", type=float, default=9.0)
    parser.add_argument("--switch-margin-m", type=float, default=250.0)
    parser.add_argument("--merge-distance-m", type=float, default=45.0)
    parser.add_argument("--skip-resort-coords", action="store_true")
    parser.add_argument("--skip-reassign", action="store_true")
    parser.add_argument("--skip-merge", action="store_true")
    args = parser.parse_args()

    rc = run_cleanup_workers(
        num_workers=max(1, args.workers),
        start_delay=max(0.0, args.start_delay),
    )
    if rc != 0:
        print(f"\nCleanup-Worker failed (exit={rc}). Breche Post-Processing ab.")
        sys.exit(rc)

    if not args.skip_resort_coords:
        rc = run_resort_coordinate_update()
        if rc != 0:
            print(f"\nResort-Koordinaten-Update fehlgeschlagen (exit={rc}).")
            sys.exit(rc)

    if not args.skip_reassign:
        rc = run_cluster_reassign(
            cluster_km=args.cluster_km,
            switch_margin_m=args.switch_margin_m,
        )
        if rc != 0:
            print(f"\nCluster-Reassignment fehlgeschlagen (exit={rc}).")
            sys.exit(rc)

    if not args.skip_merge:
        rc = run_merge_similar_slopes(args.merge_distance_m)
        if rc != 0:
            print(f"\nMerge Similar Slopes fehlgeschlagen (exit={rc}).")
            sys.exit(rc)

    print("\nCleanup-Launcher abgeschlossen.")


if __name__ == "__main__":
    main()
