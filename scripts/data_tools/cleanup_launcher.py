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
# CONFIG (must be defined first!)
# =========================
NUM_WORKERS = 30
START_DELAY = 1
SCRIPT_PATH = Path(__file__).resolve().parent / "cleanup_ski_data.py"
REASSIGN_SCRIPT_PATH = Path(__file__).resolve().parent / "reassign_entities_by_resort_cluster.py"
UPDATE_RESORT_COORDS_SCRIPT_PATH = Path(__file__).resolve().parent / "update_resort_coordinates.py"
ENRICH_SLOPE_PATHS_SCRIPT_PATH = Path(__file__).resolve().parent / "enrich_slope_paths_from_osm.py"
MERGE_SCRIPT_PATH = Path(__file__).resolve().parent / "merge_similar_slopes.py"
BASE_DIR = Path(__file__).resolve().parents[2]
CHECKPOINT_DIR = BASE_DIR / "checkpoints" / "cleanup"
LOG_DIR = BASE_DIR / "logs" / "cleanup_launcher"
PROGRESS_FILE = CHECKPOINT_DIR / "launcher_progress.json"
CLEANUP_PROGRESS_FILE = "cleanup_progress.json"

os.makedirs(LOG_DIR, exist_ok=True)


# =========================
# LOGGING SETUP
# =========================
def setup_logging():
    """Konfiguriert Logging für den Launcher."""
    timestamp = datetime.now().strftime("%Y-%m-%d_%H-%M-%S")
    log_file = LOG_DIR / f"launcher_{timestamp}.log"
    
    # Logger erstellen
    logger = logging.getLogger("cleanup_launcher")
    logger.setLevel(logging.INFO)
    
    # File Handler
    file_handler = logging.FileHandler(log_file, encoding="utf-8")
    file_handler.setLevel(logging.INFO)
    
    # Console Handler
    console_handler = logging.StreamHandler()
    console_handler.setLevel(logging.INFO)
    
    # Formatter
    formatter = logging.Formatter("%(asctime)s | %(levelname)s | %(message)s")
    file_handler.setFormatter(formatter)
    console_handler.setFormatter(formatter)
    
    logger.addHandler(file_handler)
    logger.addHandler(console_handler)
    
    return logger


# Globaler Logger
launcher_logger = setup_logging()


def log_info(msg):
    """Loggt eine Info-Nachricht."""
    print(msg)
    launcher_logger.info(msg)


def log_warning(msg):
    """Loggt eine Warnung."""
    print(f"WARNING: {msg}")
    launcher_logger.warning(msg)


def log_error(msg):
    """Loggt einen Fehler."""
    print(f"ERROR: {msg}")
    launcher_logger.error(msg)

# =========================
# PYTHON EXECUTABLE
# =========================
PYTHON = sys.executable
STAGES = [
    "cleanup_workers",
    "resort_coords",
    "reassign",
    "enrich_slope_paths",
    "merge",
    "done",
]


def save_progress(stage, status="running", last_value=None):
    CHECKPOINT_DIR.mkdir(parents=True, exist_ok=True)
    payload = {
        "stage": stage,
        "status": status,
        "updated_at": datetime.now(timezone.utc).isoformat(),
        "last_value": last_value,
    }
    with open(PROGRESS_FILE, "w", encoding="utf-8") as f:
        f.write(json.dumps(payload, ensure_ascii=True))


def load_progress():
    try:
        with open(PROGRESS_FILE, "r", encoding="utf-8") as f:
            raw = f.read().strip()
            if not raw:
                return STAGES[0]

            # Backward compatibility: old format used plain stage text.
            if raw in STAGES:
                return raw

            data = json.loads(raw)
            stage = str(data.get("stage", "")).strip()
            if stage in STAGES:
                return stage
    except (FileNotFoundError, json.JSONDecodeError):
        pass
    return STAGES[0]


def next_stage(current_stage):
    try:
        idx = STAGES.index(current_stage)
    except ValueError:
        return STAGES[0]
    if idx >= len(STAGES) - 1:
        return STAGES[-1]
    return STAGES[idx + 1]


def run_cleanup_workers(num_workers, start_delay):
    processes = []

    log_info(f"Starte {num_workers} Cleanup-Worker...")

    for i in range(num_workers):
        log_info(f"  -> Starte Cleanup-Worker {i}")

        cmd = [
            PYTHON,
            str(SCRIPT_PATH),
            str(i),
            str(num_workers),
        ]

        p = subprocess.Popen(cmd, cwd=str(BASE_DIR))
        processes.append(p)
        time.sleep(start_delay)

    log_info("Alle Cleanup-Worker gestartet.")

    rc = 0
    for p in processes:
        code = p.wait()
        if code != 0 and rc == 0:
            rc = code

    log_info("Alle Cleanup-Worker beendet.")

    if rc == 0:
        log_info("Cleanup erfolgreich. Bereinige worker-spezifische Checkpoints...")
        checkpoint_files = list(CHECKPOINT_DIR.glob("cleanup_progress_worker_*_of_*.json"))
        for f in checkpoint_files:
            try:
                os.remove(f)
                log_info(f"  -> Gelöscht: {f.name}")
            except OSError as e:
                log_warning(f"  -> Konnte {f.name} nicht löschen: {e}")

        # Auch den Standard-Checkpoint löschen, falls vorhanden
        standard_checkpoint = CHECKPOINT_DIR / CLEANUP_PROGRESS_FILE
        if standard_checkpoint.exists():
            try:
                os.remove(standard_checkpoint)
                log_info(f"  -> Gelöscht: {standard_checkpoint.name}")
            except OSError as e:
                log_warning(f"  -> Konnte {standard_checkpoint.name} nicht löschen: {e}")

    return rc


def run_cluster_reassign(cluster_km, switch_margin_m):
    log_info("Starte Cluster-Reassignment (eindeutige Resort-Zuordnung)...")
    launcher_logger.info(f"  Parameter: cluster_km={cluster_km}, switch_margin_m={switch_margin_m}")
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
    log_info("Starte Resort-Koordinaten-Update (OSM -> Zentrum -> unverändert)...")
    cmd = [
        PYTHON,
        str(UPDATE_RESORT_COORDS_SCRIPT_PATH),
    ]
    return subprocess.call(cmd, cwd=str(BASE_DIR))


def run_merge_similar_slopes(merge_distance_m):
    log_info("Starte Merge Similar Slopes...")
    launcher_logger.info(f"  Parameter: distance_m={merge_distance_m}")
    cmd = [
        PYTHON,
        str(MERGE_SCRIPT_PATH),
        "--distance-m",
        str(merge_distance_m),
        "--apply",
    ]
    return subprocess.call(cmd, cwd=str(BASE_DIR))


def run_enrich_slope_paths():
    log_info("Starte OSM-Pistenverlauf-Enrichment...")
    cmd = [
        PYTHON,
        str(ENRICH_SLOPE_PATHS_SCRIPT_PATH),
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
    parser.add_argument("--skip-enrich-slope-paths", action="store_true")
    parser.add_argument("--skip-merge", action="store_true")
    parser.add_argument("--reset-progress", action="store_true")
    args = parser.parse_args()

    log_info("=" * 60)
    log_info("Cleanup Launcher gestartet")
    log_info("=" * 60)
    log_info(f"Parameter: workers={args.workers}, cluster_km={args.cluster_km}")
    log_info(f"           merge_distance_m={args.merge_distance_m}")
    log_info(f"Progress-Datei: {PROGRESS_FILE}")
    log_info("=" * 60)

    if args.reset_progress:
        save_progress(
            STAGES[0],
            status="reset",
            last_value={"last_completed_stage": None},
        )
        log_info("Fortschritt zurückgesetzt.")

    start_stage = load_progress()
    save_progress(
        start_stage,
        status="resume",
        last_value={"message": "launcher_started_or_resumed"},
    )

    if start_stage == "done":
        log_info("Cleanup-Launcher ist laut Fortschrittsdatei bereits abgeschlossen.")
        log_info(f"Für Neustart: --reset-progress (Datei: {PROGRESS_FILE})")
        return

    log_info(f"Fortschritt: starte/resume ab Stage '{start_stage}'")

    stage_runners = [
        (
            "cleanup_workers",
            True,
            lambda: run_cleanup_workers(
                num_workers=max(1, args.workers),
                start_delay=max(0.0, args.start_delay),
            ),
            "Cleanup-Worker failed",
        ),
        (
            "resort_coords",
            not args.skip_resort_coords,
            run_resort_coordinate_update,
            "Resort-Koordinaten-Update fehlgeschlagen",
        ),
        (
            "reassign",
            not args.skip_reassign,
            lambda: run_cluster_reassign(
                cluster_km=args.cluster_km,
                switch_margin_m=args.switch_margin_m,
            ),
            "Cluster-Reassignment fehlgeschlagen",
        ),
        (
            "enrich_slope_paths",
            not args.skip_enrich_slope_paths,
            run_enrich_slope_paths,
            "OSM-Pistenverlauf-Enrichment fehlgeschlagen",
        ),
        (
            "merge",
            not args.skip_merge,
            lambda: run_merge_similar_slopes(args.merge_distance_m),
            "Merge Similar Slopes fehlgeschlagen",
        ),
    ]

    running = False
    for stage_name, enabled, runner, error_prefix in stage_runners:
        if not running:
            if stage_name != start_stage:
                continue
            running = True

        if not enabled:
            log_info(f"Stage '{stage_name}' wird per Flag übersprungen.")
            save_progress(
                next_stage(stage_name),
                status="skipped",
                last_value={"last_completed_stage": stage_name},
            )
            continue

        log_info("-" * 40)
        log_info(f"Stage: {stage_name}")
        log_info("-" * 40)
        
        save_progress(
            stage_name,
            status="running",
            last_value={"current_stage": stage_name},
        )
        rc = runner()
        
        if rc != 0:
            save_progress(
                stage_name,
                status="failed",
                last_value={"failed_stage": stage_name, "exit_code": rc},
            )
            log_error(f"{error_prefix} (exit={rc})")
            log_error(f"Fortschritt gespeichert bei Stage '{stage_name}'")
            sys.exit(rc)

        save_progress(
            next_stage(stage_name),
            status="ok",
            last_value={"last_completed_stage": stage_name},
        )
        log_info(f"Stage '{stage_name}' erfolgreich abgeschlossen.")

    log_info("=" * 60)
    log_info("Cleanup Launcher abgeschlossen!")
    log_info("=" * 60)


if __name__ == "__main__":
    main()
