//! Wipe Table Tool for Openslope API
//!
//! This tool creates a JSON backup of all records in a specified table
//! via the Openslope REST API, then deletes all records from that table.
//!
//! ## Safety Features
//!
//! - **Backup First**: All records are backed up to a timestamped JSON file before deletion
//! - **Verification**: Backup file is verified to be non-empty before proceeding
//! - **Confirmation**: User must explicitly confirm deletion with "y" input
//! - **API Only**: No direct database access; all operations via REST API
//!
//! ## Usage
//!
//! ```bash
//! # Basic usage
//! cargo run -- --table slopes
//!
//! # With custom backup directory
//! cargo run -- --table lifts --backup-dir ./backups
//!
//! # Dry run (preview without deleting)
//! cargo run -- --table slopes --dry-run
//! ```
//!
//! ## Supported Tables
//!
//! - `slopes` - All ski slopes
//! - `lifts` - All ski lifts
//! - `resorts` - All ski resorts
//!
//! ## Output
//!
//! - Backup file: `backups/<table_name>_<YYYY-MM-DD_HH-MM-SS>.json`
//! - Console summary with record counts
//!
//! ## Author
//! Openslope Team
//!
//! ## Version
//! 1.0.0

use chrono::Local;
use dotenvy::dotenv;
use serde_json::Value;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

// =============================================================================
// Configuration
// =============================================================================

/// Default API base URL
const DEFAULT_API_BASE_URL: &str = "http://localhost:8080";

/// Default API key (should be overridden via environment or config)
const DEFAULT_API_KEY: &str = "R3StTY4OfadeFJZurXdZ1pZMVbWB3zWuL6FnuPGIbvA";

/// Valid table names that can be wiped
const VALID_TABLES: &[&str] = &["slopes", "lifts", "resorts"];

// =============================================================================
// API Client
// =============================================================================

/// Creates a reqwest client with appropriate headers
fn create_client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::new()
}

/// Build the API headers with authorization
fn build_headers(api_key: &str) -> reqwest::header::HeaderMap {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "Content-Type",
        reqwest::header::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        "Authorization",
        reqwest::header::HeaderValue::from_str(&format!("Bearer {}", api_key))
            .expect("Invalid API key header value"),
    );
    headers
}

/// Fetch all records from a table via the API
/// Uses pagination if needed (API returns all records in one call for simplicity)
fn fetch_all_records(api_base_url: &str, table: &str, api_key: &str) -> Result<Vec<Value>, String> {
    let client = create_client();
    let headers = build_headers(api_key);
    let url = format!("{}/{}", api_base_url, table);

    println!("Fetching all records from {}...", url);

    match client.get(&url).headers(headers).send() {
        Ok(response) => {
            if !response.status().is_success() {
                return Err(format!(
                    "API returned status {}: {}",
                    response.status(),
                    response.text().unwrap_or_default()
                ));
            }

            match response.json::<Value>() {
                Ok(Value::Array(records)) => Ok(records),
                Ok(other) => Err(format!("Unexpected response format: {:?}", other)),
                Err(e) => Err(format!("Failed to parse JSON response: {}", e)),
            }
        }
        Err(e) => Err(format!("Failed to fetch records: {}", e)),
    }
}

/// Delete a single record by ID via the API
fn delete_record(api_base_url: &str, table: &str, id: i64, api_key: &str) -> Result<(), String> {
    let client = create_client();
    let headers = build_headers(api_key);
    let url = format!("{}/{}/{}", api_base_url, table, id);

    match client.delete(&url).headers(headers).send() {
        Ok(response) => {
            if response.status().is_success() || response.status() == 204 {
                Ok(())
            } else {
                Err(format!(
                    "Delete failed with status {}: {}",
                    response.status(),
                    response.text().unwrap_or_default()
                ))
            }
        }
        Err(e) => Err(format!("Delete request failed: {}", e)),
    }
}

// =============================================================================
// Backup Functions
// =============================================================================

/// Create backup directory if it doesn't exist
fn ensure_backup_dir(backup_dir: &PathBuf) -> Result<(), String> {
    match fs::create_dir_all(backup_dir) {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("Failed to create backup directory: {}", e)),
    }
}

/// Generate a timestamped backup filename
fn generate_backup_filename(table: &str) -> String {
    let timestamp = Local::now().format("%Y-%m-%d_%H-%M-%S");
    format!("{}_{}.json", table, timestamp)
}

/// Save records to a JSON backup file
fn save_backup(backup_dir: &PathBuf, table: &str, records: &[Value]) -> Result<PathBuf, String> {
    let filename = generate_backup_filename(table);
    let filepath = backup_dir.join(&filename);

    let json_content = serde_json::to_string_pretty(records)
        .map_err(|e| format!("Failed to serialize JSON: {}", e))?;

    fs::write(&filepath, &json_content)
        .map_err(|e| format!("Failed to write backup file: {}", e))?;

    // Verify the file was written and is non-empty
    let metadata =
        fs::metadata(&filepath).map_err(|e| format!("Failed to verify backup file: {}", e))?;

    if metadata.len() == 0 {
        fs::remove_file(&filepath).ok();
        return Err("Backup file is empty - aborting".to_string());
    }

    println!("Backup saved to: {}", filepath.display());
    Ok(filepath)
}

// =============================================================================
// User Interaction
// =============================================================================

/// Prompt user for confirmation
fn prompt_confirmation() -> bool {
    print!("\nAre you sure you want to delete all records? [y/N]: ");
    io::stdout().flush().ok();

    let mut input = String::new();
    io::stdin().read_line(&mut input).ok();

    input.trim().to_lowercase() == "y"
}

// =============================================================================
// CLI Argument Parsing
// =============================================================================

struct CliArgs {
    table: String,
    backup_dir: PathBuf,
    dry_run: bool,
    api_base_url: String,
    api_key: String,
}

fn parse_args() -> Result<CliArgs, String> {
    let args: Vec<String> = std::env::args().collect();

    dotenv().ok();
    if env::var("API_BASE_URL").is_err() || env::var("API_KEY").is_err() {
        dotenvy::from_filename("../.env").ok();
    }
    if env::var("API_BASE_URL").is_err() || env::var("API_KEY").is_err() {
        dotenvy::from_filename("../../.env").ok();
    }

    let mut table: Option<String> = None;
    let mut backup_dir = PathBuf::from("./backups");
    let mut dry_run = false;
    let mut api_base_url =
        env::var("API_BASE_URL").unwrap_or_else(|_| DEFAULT_API_BASE_URL.to_string());
    let mut api_key = env::var("API_KEY").unwrap_or_else(|_| DEFAULT_API_KEY.to_string());

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--table" => {
                i += 1;
                if i < args.len() {
                    table = Some(args[i].clone());
                } else {
                    return Err("--table requires a value".to_string());
                }
            }
            "--backup-dir" => {
                i += 1;
                if i < args.len() {
                    backup_dir = PathBuf::from(&args[i]);
                } else {
                    return Err("--backup-dir requires a value".to_string());
                }
            }
            "--dry-run" => {
                dry_run = true;
            }
            "--api-base-url" => {
                i += 1;
                if i < args.len() {
                    api_base_url = args[i].clone();
                } else {
                    return Err("--api-base-url requires a value".to_string());
                }
            }
            "--api-key" => {
                i += 1;
                if i < args.len() {
                    api_key = args[i].clone();
                } else {
                    return Err("--api-key requires a value".to_string());
                }
            }
            "--help" | "-h" => {
                print_help();
                std::process::exit(0);
            }
            _ => {
                return Err(format!("Unknown argument: {}", args[i]));
            }
        }
        i += 1;
    }

    let table = table.ok_or("Missing required argument: --table")?;

    // Validate table name
    if !VALID_TABLES.contains(&table.as_str()) {
        return Err(format!(
            "Invalid table '{}'. Valid tables: {}",
            table,
            VALID_TABLES.join(", ")
        ));
    }

    Ok(CliArgs {
        table,
        backup_dir,
        dry_run,
        api_base_url,
        api_key,
    })
}

fn print_help() {
    println!(
        r#"
Wipe Table Tool for Openslope API

USAGE:
    cargo run -- --table <TABLE_NAME> [OPTIONS]

OPTIONS:
    --table <TABLE>       Name of the table to wipe (required)
                          Valid tables: slopes, lifts, resorts
    --backup-dir <DIR>    Directory for backup files (default: ./backups)
    --dry-run             Preview backup without deleting anything
    --api-base-url <URL>   API base URL (default from API_BASE_URL env or http://localhost:8080)
    --api-key <KEY>       API key for authorization (default from API_KEY env)
    --help, -h            Show this help message

EXAMPLES:
    # Backup and delete all slopes
    cargo run -- --table slopes

    # Backup and delete all lifts with custom backup directory
    cargo run -- --table lifts --backup-dir ./my_backups

    # Dry run (create backup only, no deletion)
    cargo run -- --table slopes --dry-run

SAFETY:
    - All records are backed up to a timestamped JSON file before deletion
    - Backup file is verified to be non-empty before proceeding
    - User confirmation is required before any deletion
    - Dry-run mode allows testing without making changes
"#
    );
}

// =============================================================================
// Main Execution
// =============================================================================

fn main() {
    println!("========================================");
    println!("  Openslope Wipe Table Tool v1.0.0");
    println!("========================================\n");

    // Parse command line arguments
    let args = match parse_args() {
        Ok(args) => args,
        Err(e) => {
            eprintln!("Error: {}", e);
            eprintln!("\nRun with --help for usage information.");
            std::process::exit(1);
        }
    };

    println!("Configuration:");
    println!("  Table:        {}", args.table);
    println!("  Backup dir:   {}", args.backup_dir.display());
    println!(
        "  Dry run:      {}",
        if args.dry_run { "yes" } else { "no" }
    );
    println!("  API URL:      {}\n", args.api_base_url);

    // Step 1: Fetch all records from the table
    let records = match fetch_all_records(&args.api_base_url, &args.table, &args.api_key) {
        Ok(records) => records,
        Err(e) => {
            eprintln!("Error fetching records: {}", e);
            std::process::exit(1);
        }
    };

    let total_records = records.len();
    println!(
        "Found {} records in table '{}'\n",
        total_records, args.table
    );

    if total_records == 0 {
        println!("Table is empty. Nothing to do.");
        return;
    }

    // Step 2: Create backup
    println!("Creating backup...");
    ensure_backup_dir(&args.backup_dir).unwrap_or_else(|e| {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    });

    let backup_path = match save_backup(&args.backup_dir, &args.table, &records) {
        Ok(path) => path,
        Err(e) => {
            eprintln!("Error creating backup: {}", e);
            std::process::exit(1);
        }
    };

    println!(
        "Backup verified: {} bytes\n",
        fs::metadata(&backup_path).map(|m| m.len()).unwrap_or(0)
    );

    // Step 3: Dry run - stop here if no deletion requested
    if args.dry_run {
        println!("DRY RUN MODE: No records were deleted.");
        println!("Backup created successfully at: {}", backup_path.display());
        return;
    }

    // Step 4: Ask for confirmation
    if !prompt_confirmation() {
        println!("Deletion cancelled by user.");
        println!("Backup preserved at: {}", backup_path.display());
        return;
    }

    // Step 5: Delete all records
    println!("\nDeleting {} records...", total_records);

    let mut deleted = 0;
    let mut errors = 0;

    for record in &records {
        // Extract ID from record - different tables may have different ID field names
        let id = record.get("id").and_then(|v| v.as_i64()).or_else(|| {
            record
                .get("id")
                .and_then(|v| Value::as_str(v).and_then(|s| s.parse().ok()))
        });

        match id {
            Some(id) => match delete_record(&args.api_base_url, &args.table, id, &args.api_key) {
                Ok(()) => {
                    deleted += 1;
                    if deleted % 100 == 0 || deleted == total_records {
                        print!("\r  Progress: {}/{} deleted...", deleted, total_records);
                        io::stdout().flush().ok();
                    }
                }
                Err(e) => {
                    errors += 1;
                    eprintln!("\n  Error deleting record {}: {}", id, e);
                }
            },
            None => {
                errors += 1;
                eprintln!("  Warning: Record missing ID field: {:?}", record);
            }
        }
    }

    println!("\n");

    // Step 6: Print summary
    println!("========================================");
    println!("  Summary");
    println!("========================================");
    println!("  Table:           {}", args.table);
    println!("  Total records:   {}", total_records);
    println!("  Deleted:         {}", deleted);
    println!("  Errors:          {}", errors);
    println!("  Backup file:     {}", backup_path.display());
    println!("========================================");

    if errors > 0 {
        eprintln!("\nWarning: {} records could not be deleted.", errors);
        std::process::exit(1);
    }
}
