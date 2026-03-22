//! OpenSlope API Status Routes
//!
//! This module handles all HTTP requests related to scraping status and operational
//! data in the OpenSlope API. It provides access to scraping run information and
//! real-time status snapshots for ski resorts.
//!
//! # Route Overview
//!
//! The status module provides the following endpoints:
//!
//! - **GET /scrape-runs**: List all scraping runs with optional resort filtering
//! - **GET /scrape-runs/{id}**: Get detailed information about a specific scraping run
//! - **GET /status-snapshots**: List all status snapshots with optional resort filtering
//! - **GET /status-snapshots/by_resort/{resort_id}**: Get all status snapshots for a specific resort
//!
//! # Data Models
//!
//! The module defines several data structures for handling status information:
//!
//! - **ScrapeRunResponse**: Information about a scraping operation
//! - **ResortStatusSnapshotResponse**: Real-time operational status of a resort
//! - **SnapshotMetric**: Lift and slope count metrics
//! - **SnowSnapshot**: Snow depth and new snow measurements
//! - **TemperatureSnapshot**: Temperature readings at valley and mountain levels
//!
//! # Key Features
//!
//! - **Scraping History**: Complete history of all scraping operations
//! - **Status Snapshots**: Real-time operational data snapshots
//! - **Resort Filtering**: All endpoints support optional resort-specific filtering
//! - **Pagination Control**: Configurable result limits with safety bounds
//! - **Timestamp Handling**: Proper ISO 8601 timestamp formatting
//!
//! # Scraping Run Information
//!
//! Each scraping run contains:
//! - **Run ID**: Unique identifier for the scraping operation
//! - **Resort ID**: Which resort was scraped
//! - **Source Name**: Data source being scraped (e.g., "alpenplus", "mtnfeed")
//! - **Timing**: Start and finish timestamps
//! - **Success Status**: Whether the scrape completed successfully
//! - **HTTP Status**: HTTP response code from the source
//! - **Message**: Any error messages or additional information
//!
//! # Status Snapshot Data
//!
//! Each status snapshot includes:
//! - **Snapshot ID**: Unique identifier for the snapshot
//! - **Run ID**: Which scraping run generated this snapshot
//! - **Resort ID**: Which resort this snapshot represents
//! - **Timestamp**: When the snapshot was taken
//! - **Lift Metrics**: Open/total lift counts
//! - **Slope Metrics**: Open/total slope counts
//! - **Snow Data**: Depth measurements and new snow
//! - **Temperature Data**: Valley and mountain temperature readings
//!
//! # Pagination and Limits
//!
//! - **Default Limit**: 100 records per request
//! - **Maximum Limit**: 500 records per request
//! - **Minimum Limit**: 1 record per request
//! - **Safety Bounds**: Limits are clamped to prevent excessive data transfer
//!
//! # Timestamp Format
//!
//! All timestamps are returned in ISO 8601 format:
//! - Format: `YYYY-MM-DDTHH:MM:SSZ`
//! - Timezone: UTC (Zulu time)
//! - Example: `2026-03-22T20:30:45Z`
//!
//! # Error Handling
//!
//! - **Database Errors**: Return 500 Internal Server Error
//! - **Missing Resources**: Return 404 Not Found
//! - **Invalid Parameters**: Return 400 Bad Request
//! - **Consistent Logging**: All errors are logged with context
//!
//! # Performance Considerations
//!
//! - **Efficient Queries**: Optimized SQL with proper column selection
//! - **Index Usage**: Queries designed to use database indexes effectively
//! - **Result Limiting**: Automatic pagination to prevent large result sets
//! - **Optional Filtering**: Resort-specific queries for better performance
//!
//! # Usage Examples
//!
//! ```rust
//! // Get all scraping runs
//! GET /api/v1/scrape-runs
//!
//! // Get scraping runs for specific resort
//! GET /api/v1/scrape-runs?resort_id=resort_abc
//!
//! // Get scraping runs with custom limit
//! GET /api/v1/scrape-runs?limit=50
//!
//! // Get specific scraping run
//! GET /api/v1/scrape-runs/123
//!
//! // Get all status snapshots
//! GET /api/v1/status-snapshots
//!
//! // Get status snapshots for specific resort
//! GET /api/v1/status-snapshots/by_resort/resort_abc
//!
//! // Get status snapshots with custom limit
//! GET /api/v1/status-snapshots?limit=25
//! ```
//!
//! Author: OpenSlope Team
//! Version: 1.0.0

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;

#[derive(Deserialize)]
pub struct StatusQuery {
    pub resort_id: Option<String>,
    pub limit: Option<i64>,
}

#[derive(Serialize)]
pub struct ScrapeRunResponse {
    pub id: i64,
    pub resort_id: String,
    pub source_name: String,
    pub started_at: String,
    pub finished_at: Option<String>,
    pub success: bool,
    pub http_status: Option<i32>,
    pub message: Option<String>,
}

#[derive(Serialize)]
pub struct ResortStatusSnapshotResponse {
    pub id: i64,
    pub run_id: i64,
    pub resort_id: String,
    pub snapshot_time: String,
    pub lifts: SnapshotMetric,
    pub slopes: SnapshotMetric,
    pub snow: SnowSnapshot,
    pub temperature: TemperatureSnapshot,
}

#[derive(Serialize)]
pub struct SnapshotMetric {
    pub open_count: Option<i32>,
    pub total_count: Option<i32>,
}

#[derive(Serialize)]
pub struct SnowSnapshot {
    pub valley_cm: Option<i16>,
    pub mountain_cm: Option<i16>,
    pub new_snow_24h_cm: Option<i16>,
}

#[derive(Serialize)]
pub struct TemperatureSnapshot {
    pub valley_c: Option<f64>,
    pub mountain_c: Option<f64>,
}

fn clamp_limit(value: Option<i64>) -> i64 {
    value.unwrap_or(100).clamp(1, 500)
}

pub async fn get_scrape_runs(
    db: web::Data<MySqlPool>,
    query: web::Query<StatusQuery>,
) -> impl Responder {
    let limit = clamp_limit(query.limit);
    let resort_id = query.resort_id.clone();

    let result = sqlx::query!(
        r#"
        SELECT id, resort_id, source_name,
               DATE_FORMAT(started_at, '%Y-%m-%dT%H:%i:%sZ') AS started_at,
               DATE_FORMAT(finished_at, '%Y-%m-%dT%H:%i:%sZ') AS finished_at,
               success, http_status, message
        FROM scrape_runs
        WHERE (? IS NULL OR resort_id = ?)
        ORDER BY started_at DESC
        LIMIT ?
        "#,
        resort_id,
        resort_id,
        limit
    )
    .fetch_all(db.get_ref())
    .await;

    match result {
        Ok(rows) => {
            let response: Vec<ScrapeRunResponse> = rows
                .into_iter()
                .map(|row| ScrapeRunResponse {
                    id: row.id,
                    resort_id: row.resort_id,
                    source_name: row.source_name,
                    started_at: row.started_at.unwrap_or_else(|| "".to_string()),
                    finished_at: row.finished_at,
                    success: row.success != 0,
                    http_status: row.http_status,
                    message: row.message,
                })
                .collect();

            HttpResponse::Ok().json(response)
        }
        Err(err) => {
            eprintln!("GET /scrape-runs error: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_scrape_run(
    db: web::Data<MySqlPool>,
    id: web::Path<i64>,
) -> impl Responder {
    let result = sqlx::query!(
        r#"
        SELECT id, resort_id, source_name,
               DATE_FORMAT(started_at, '%Y-%m-%dT%H:%i:%sZ') AS started_at,
               DATE_FORMAT(finished_at, '%Y-%m-%dT%H:%i:%sZ') AS finished_at,
               success, http_status, message
        FROM scrape_runs
        WHERE id = ?
        "#,
        *id
    )
    .fetch_optional(db.get_ref())
    .await;

    match result {
        Ok(Some(row)) => HttpResponse::Ok().json(ScrapeRunResponse {
            id: row.id,
            resort_id: row.resort_id,
            source_name: row.source_name,
            started_at: row.started_at.unwrap_or_else(|| "".to_string()),
            finished_at: row.finished_at,
            success: row.success != 0,
            http_status: row.http_status,
            message: row.message,
        }),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(err) => {
            eprintln!("GET /scrape-runs/{{id}} error: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_status_snapshots(
    db: web::Data<MySqlPool>,
    query: web::Query<StatusQuery>,
) -> impl Responder {
    let limit = clamp_limit(query.limit);
    let resort_id = query.resort_id.clone();

    let result = sqlx::query!(
        r#"
        SELECT id, run_id, resort_id,
               DATE_FORMAT(snapshot_time, '%Y-%m-%dT%H:%i:%sZ') AS snapshot_time,
               lifts_open_count, lifts_total_count,
               slopes_open_count, slopes_total_count,
               snow_depth_valley_cm, snow_depth_mountain_cm,
               new_snow_24h_cm,
               CAST(temperature_valley_c AS DOUBLE) AS temperature_valley_c,
               CAST(temperature_mountain_c AS DOUBLE) AS temperature_mountain_c
        FROM resort_status_snapshots
        WHERE (? IS NULL OR resort_id = ?)
        ORDER BY snapshot_time DESC
        LIMIT ?
        "#,
        resort_id,
        resort_id,
        limit
    )
    .fetch_all(db.get_ref())
    .await;

    match result {
        Ok(rows) => {
            let response: Vec<ResortStatusSnapshotResponse> = rows
                .into_iter()
                .map(|row| ResortStatusSnapshotResponse {
                    id: row.id,
                    run_id: row.run_id,
                    resort_id: row.resort_id,
                    snapshot_time: row.snapshot_time.unwrap_or_else(|| "".to_string()),
                    lifts: SnapshotMetric {
                        open_count: row.lifts_open_count,
                        total_count: row.lifts_total_count,
                    },
                    slopes: SnapshotMetric {
                        open_count: row.slopes_open_count,
                        total_count: row.slopes_total_count,
                    },
                    snow: SnowSnapshot {
                        valley_cm: row.snow_depth_valley_cm,
                        mountain_cm: row.snow_depth_mountain_cm,
                        new_snow_24h_cm: row.new_snow_24h_cm,
                    },
                    temperature: TemperatureSnapshot {
                        valley_c: row.temperature_valley_c,
                        mountain_c: row.temperature_mountain_c,
                    },
                })
                .collect();

            HttpResponse::Ok().json(response)
        }
        Err(err) => {
            eprintln!("GET /status-snapshots error: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_status_snapshots_by_resort(
    db: web::Data<MySqlPool>,
    resort_id: web::Path<String>,
    query: web::Query<StatusQuery>,
) -> impl Responder {
    let limit = clamp_limit(query.limit);

    let result = sqlx::query!(
        r#"
        SELECT id, run_id, resort_id,
               DATE_FORMAT(snapshot_time, '%Y-%m-%dT%H:%i:%sZ') AS snapshot_time,
               lifts_open_count, lifts_total_count,
               slopes_open_count, slopes_total_count,
               snow_depth_valley_cm, snow_depth_mountain_cm,
               new_snow_24h_cm,
               CAST(temperature_valley_c AS DOUBLE) AS temperature_valley_c,
               CAST(temperature_mountain_c AS DOUBLE) AS temperature_mountain_c
        FROM resort_status_snapshots
        WHERE resort_id = ?
        ORDER BY snapshot_time DESC
        LIMIT ?
        "#,
        resort_id.into_inner(),
        limit
    )
    .fetch_all(db.get_ref())
    .await;

    match result {
        Ok(rows) => {
            let response: Vec<ResortStatusSnapshotResponse> = rows
                .into_iter()
                .map(|row| ResortStatusSnapshotResponse {
                    id: row.id,
                    run_id: row.run_id,
                    resort_id: row.resort_id,
                    snapshot_time: row.snapshot_time.unwrap_or_else(|| "".to_string()),
                    lifts: SnapshotMetric {
                        open_count: row.lifts_open_count,
                        total_count: row.lifts_total_count,
                    },
                    slopes: SnapshotMetric {
                        open_count: row.slopes_open_count,
                        total_count: row.slopes_total_count,
                    },
                    snow: SnowSnapshot {
                        valley_cm: row.snow_depth_valley_cm,
                        mountain_cm: row.snow_depth_mountain_cm,
                        new_snow_24h_cm: row.new_snow_24h_cm,
                    },
                    temperature: TemperatureSnapshot {
                        valley_c: row.temperature_valley_c,
                        mountain_c: row.temperature_mountain_c,
                    },
                })
                .collect();

            HttpResponse::Ok().json(response)
        }
        Err(err) => {
            eprintln!("GET /resorts/{{resort_id}}/status-snapshots error: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}
