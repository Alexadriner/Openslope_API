#!/usr/bin/env python3
"""
Bergfex API Collector Launcher

This script launches the Bergfex collector for daily data collection.
It can be scheduled to run once per day using cron or Windows Task Scheduler.

Usage:
    python bergfex_launcher.py [--resort-id RESORT_ID] [--api-key API_KEY]
"""

import argparse
import logging
import os
import subprocess
import sys
from pathlib import Path

# =========================
# PATHS
# =========================
ROOT_DIR = Path(__file__).resolve().parents[3]
SCRIPT_PATH = Path(__file__).resolve().parent / "bergfex_collector.py"

# =========================
# CONFIG
# =========================
PYTHON = sys.executable
LOG_DIR = ROOT_DIR / "logs" / "api_collector"

# =========================
# UTILS
# =========================
def setup_logging():
    """Setup logging configuration"""
    os.makedirs(LOG_DIR, exist_ok=True)
    log_filename = "bergfex_launcher.log"
    log_path = LOG_DIR / log_filename

    logging.basicConfig(
        level=logging.INFO,
        format="%(asctime)s | %(levelname)s | %(message)s",
        handlers=[
            logging.FileHandler(str(log_path), encoding="utf-8"),
            logging.StreamHandler(sys.stdout),
        ],
    )
    return logging.getLogger("bergfex_launcher")

def run_collector(resort_id: str, api_key: Optional[str] = None) -> bool:
    """Run the Bergfex collector"""
    logger = logging.getLogger("bergfex_launcher")
    
    # Build command
    cmd = [PYTHON, str(SCRIPT_PATH), "--resort-id", resort_id]
    
    if api_key:
        cmd.extend(["--api-key", api_key])
    
    logger.info(f"Running collector with command: {' '.join(cmd)}")
    
    try:
        # Run the collector
        result = subprocess.run(
            cmd,
            cwd=str(ROOT_DIR),
            capture_output=True,
            text=True,
            timeout=300  # 5 minute timeout
        )
        
        # Log output
        if result.stdout:
            logger.info(f"Collector output: {result.stdout}")
        if result.stderr:
            logger.error(f"Collector errors: {result.stderr}")
        
        # Check result
        if result.returncode == 0:
            logger.info("Collector completed successfully")
            return True
        else:
            logger.error(f"Collector failed with return code: {result.returncode}")
            return False
            
    except subprocess.TimeoutExpired:
        logger.error("Collector timed out after 5 minutes")
        return False
    except Exception as e:
        logger.error(f"Error running collector: {e}")
        return False

def main():
    """Main entry point"""
    logger = setup_logging()
    
    parser = argparse.ArgumentParser(description="Bergfex API Collector Launcher")
    parser.add_argument("--resort-id", default="sonnenbuehl-genkingen", 
                       help="Resort ID (default: sonnenbuehl-genkingen)")
    parser.add_argument("--api-key", help="SkiAPI key (overrides environment/file)")
    
    args = parser.parse_args()
    
    logger.info(f"Starting Bergfex launcher for resort: {args.resort_id}")
    
    # Run collector
    success = run_collector(args.resort_id, args.api_key)
    
    if success:
        logger.info("Launcher completed successfully")
        sys.exit(0)
    else:
        logger.error("Launcher failed")
        sys.exit(1)

if __name__ == "__main__":
    main()