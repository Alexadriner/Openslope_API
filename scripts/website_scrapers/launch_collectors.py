"""
Website Scrapers Launcher

This module provides a parallel launcher for running multiple website collectors
simultaneously. It discovers all available collector modules, manages their
execution, and handles graceful shutdown.

The launcher is designed to work with the modular collector architecture where
each ski resort has its own collector module that can be run independently or
in parallel with others.

Usage:
    python scripts/website_scrapers/launch_collectors.py [options]

Author: OpenSlope Team
"""

import argparse
import signal
import subprocess
import sys
import time
from pathlib import Path


# Base directory for the website scrapers
BASE_DIR = Path(__file__).resolve().parent


def discover_collectors() -> list[str]:
    """
    Discover all available collector modules in the website scrapers directory.
    
    Returns:
        list[str]: List of collector module names (resort slugs)
    """
    collectors: list[str] = []
    
    # Find collectors in the main directory (e.g., kreuzberg, palisades_tahoe)
    for collector_file in sorted(BASE_DIR.glob("*/collector.py")):
        resort_slug = collector_file.parent.name
        # Skip modules that start with underscore (private/hidden)
        if resort_slug.startswith("_"):
            continue
        collectors.append(resort_slug)
    
    # Find Alpenplus subdirectory collectors (e.g., alpenplus_brauneck)
    for collector_file in sorted(BASE_DIR.glob("alpenplus/*/collector.py")):
        resort_slug = f"alpenplus_{collector_file.parent.name}"
        collectors.append(resort_slug)
    
    return collectors


def build_command(
    resort_slug: str, interval_seconds: int, once: bool, no_sync_api: bool
) -> list[str]:
    """
    Build the command to execute a specific collector module.
    
    Args:
        resort_slug: The slug identifier for the resort
        interval_seconds: Polling interval in seconds
        once: Whether to run only once instead of continuously
        no_sync_api: Whether to skip API synchronization
    
    Returns:
        list[str]: Command arguments for subprocess execution
    """
    # Handle Alpenplus subdirectory collectors
    if resort_slug.startswith("alpenplus_"):
        resort_name = resort_slug[10:]  # Remove "alpenplus_" prefix
        module = f"scripts.website_scrapers.alpenplus.{resort_name}.collector"
    else:
        module = f"scripts.website_scrapers.{resort_slug}.collector"
    
    # Build command with module path
    # Note: Do not force --resort-id from folder name.
    # Some collectors use a different canonical resort id than the module slug.
    cmd = [sys.executable, "-m", module]
    
    # Add optional parameters
    if interval_seconds is not None:
        cmd.extend(["--interval-seconds", str(interval_seconds)])
    if once:
        cmd.append("--once")
    if no_sync_api:
        cmd.append("--no-sync-api")
    
    return cmd


def terminate_processes(processes: dict[str, subprocess.Popen]) -> None:
    """
    Gracefully terminate all running collector processes.
    
    Args:
        processes: Dictionary mapping process names to Popen objects
    """
    # First attempt: graceful termination
    for name, proc in processes.items():
        if proc.poll() is None:  # Process is still running
            print(f"[launcher] Stopping {name} (pid={proc.pid}) ...")
            proc.terminate()

    # Wait up to 8 seconds for graceful shutdown
    deadline = time.time() + 8
    while time.time() < deadline:
        if all(proc.poll() is not None for proc in processes.values()):
            return
        time.sleep(0.2)

    # Force kill any remaining processes
    for name, proc in processes.items():
        if proc.poll() is None:
            print(f"[launcher] Killing {name} (pid={proc.pid}) ...")
            proc.kill()


def parse_csv_list(value: str | None) -> set[str]:
    """
    Parse a comma-separated string into a set of stripped values.
    
    Args:
        value: Comma-separated string or None
    
    Returns:
        set[str]: Set of non-empty, stripped values
    """
    if not value:
        return set()
    return {part.strip() for part in value.split(",") if part.strip()}


def main() -> int:
    """
    Main entry point for the launcher.
    
    Parses command line arguments, discovers collectors, and manages their execution.
    
    Returns:
        int: Exit code (0 for success, non-zero for failure)
    """
    # Set up argument parser
    parser = argparse.ArgumentParser(
        description="Launch all website collector.py instances in parallel."
    )
    
    # Command line arguments
    parser.add_argument(
        "--interval-seconds",
        type=int,
        default=300,
        help="Polling interval passed to each collector (default: 300).",
    )
    parser.add_argument(
        "--once",
        action="store_true",
        help="Run each collector only once.",
    )
    parser.add_argument(
        "--no-sync-api",
        action="store_true",
        help="Pass --no-sync-api to all collectors.",
    )
    parser.add_argument(
        "--only",
        type=str,
        default="",
        help="Comma-separated resort slugs to start (e.g. kreuzberg,palisades_tahoe).",
    )
    parser.add_argument(
        "--skip",
        type=str,
        default="",
        help="Comma-separated resort slugs to skip.",
    )
    
    args = parser.parse_args()

    # Discover all available collectors
    all_collectors = discover_collectors()
    if not all_collectors:
        print("[launcher] No collector.py files found in scripts/website_scrapers/*")
        return 1

    # Parse filter lists
    only = parse_csv_list(args.only)
    skip = parse_csv_list(args.skip)

    # Filter collectors based on --only and --skip parameters
    selected = [name for name in all_collectors if (not only or name in only) and name not in skip]
    if not selected:
        print("[launcher] No collectors selected after applying --only/--skip filters.")
        return 1

    # Display discovered and selected collectors
    print(f"[launcher] Found collectors: {', '.join(all_collectors)}")
    print(f"[launcher] Starting: {', '.join(selected)}")

    # Dictionary to track running processes
    processes: dict[str, subprocess.Popen] = {}

    def _handle_shutdown(signum: int, _frame) -> None:
        """
        Signal handler for graceful shutdown.
        
        Args:
            signum: Signal number received
            _frame: Signal frame (unused)
        """
        print(f"\n[launcher] Received signal {signum}, shutting down collectors ...")
        terminate_processes(processes)
        raise SystemExit(130)

    # Set up signal handlers for graceful shutdown
    signal.signal(signal.SIGINT, _handle_shutdown)
    if hasattr(signal, "SIGTERM"):
        signal.signal(signal.SIGTERM, _handle_shutdown)

    try:
        # Start all selected collectors
        for resort_slug in selected:
            cmd = build_command(
                resort_slug=resort_slug,
                interval_seconds=max(60, args.interval_seconds),  # Minimum 60 seconds
                once=args.once,
                no_sync_api=args.no_sync_api,
            )
            # Run each collector as a subprocess from the project root
            proc = subprocess.Popen(cmd, cwd=str(BASE_DIR.parents[1]))
            processes[resort_slug] = proc
            print(f"[launcher] Started {resort_slug} (pid={proc.pid})")

        # Handle one-time execution mode
        if args.once:
            exit_code = 0
            for resort_slug, proc in processes.items():
                code = proc.wait()
                print(f"[launcher] {resort_slug} exited with code {code}")
                if code != 0:
                    exit_code = code
            return exit_code

        # Continuous monitoring mode
        while True:
            time.sleep(2)
            for resort_slug, proc in processes.items():
                code = proc.poll()
                if code is not None:
                    print(f"[launcher] {resort_slug} exited unexpectedly with code {code}.")
                    terminate_processes(processes)
                    return code
                    
    finally:
        # Ensure all processes are terminated on exit
        terminate_processes(processes)


if __name__ == "__main__":
    raise SystemExit(main())
