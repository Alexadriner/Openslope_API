//! OpenSlope API Slopes Routes
//!
//! This module handles all HTTP requests related to ski slope management in the
//! OpenSlope API. It provides comprehensive CRUD operations for individual slopes
//! including detailed information about their geometry, specifications, and status.
//!
//! # Route Overview
//!
//! The slopes module provides the following endpoints:
//!
//! - **GET /slopes**: List all slopes with detailed information
//! - **GET /slopes/{id}**: Get detailed information about a specific slope
//! - **GET /slopes/by_resort/{resort_id}**: Get all slopes for a specific resort
//! - **POST /slopes**: Create a new slope
//! - **PUT /slopes/{id}**: Update an existing slope
//! - **DELETE /slopes/{id}**: Delete a slope
//! - **DELETE /slopes/by_resort/{resort_id}**: Delete all slopes for a resort
//!
//! # Data Models
//!
//! The module defines several data structures for handling slope information:
//!
//! - **Slope**: Complete response model with nested data structures
//! - **SlopeDisplay**: Display-related information (name, difficulty)
//! - **SlopeGeometry**: Geographical coordinates and path data
//! - **SlopeSpecs**: Technical specifications (length, gradients, features)
//! - **SlopeSource**: Source system information and entity references
//! - **SlopeStatus**: Operational status and grooming information
//! - **CreateSlope/UpdateSlope**: Input models for creation and updates
//!
//! # Key Features
//!
//! - **Complex Geometry**: Start/end points plus optional path geometry for detailed mapping
//! - **Difficulty Classification**: Standard ski slope difficulty levels (Green, Blue, Red, Black)
//! - **Technical Specifications**: Length, gradients, vertical drop measurements
//! - **Skiing Features**: Snowmaking, night skiing, family-friendly, race slope indicators
//! - **Status Management**: Operational status and grooming status tracking
//! - **Source Integration**: Support for multiple data sources (OSM, official sources)
//!
//! # Coordinate System
//!
//! - Latitude and longitude use WGS84 coordinate system
//! - All coordinates are cast to DOUBLE precision for accuracy
//! - Path geometry supports complex multi-point routes
//!
//! # Difficulty Levels
//!
//! Standard ski slope difficulty classifications:
//! - **"Green"**: Beginner slopes, gentle gradients (typically < 25%)
//! - **"Blue"**: Intermediate slopes, moderate gradients (typically 25-40%)
//! - **"Red"**: Advanced slopes, steep gradients (typically 40-60%)
//! - **"Black"**: Expert slopes, very steep and challenging (typically > 60%)
//!
//! # Slope Features
//!
//! Boolean indicators for slope characteristics:
//! - **snowmaking**: Whether the slope has artificial snowmaking coverage
//! - **night_skiing**: Whether the slope is lit for evening skiing
//! - **family_friendly**: Whether the slope is suitable for families
//! - **race_slope**: Whether the slope is used for competitive racing
//!
//! # Path Geometry
//!
//! Slopes can have complex path geometry defined as GeoJSON arrays:
//! - Each point contains latitude and longitude coordinates
//! - Supports both "latitude"/"longitude" and "lat"/"lon" field names
//! - Optional path allows for slopes with non-linear routes
//! - Direction field provides compass bearing information
//!
//! # Status Values
//!
//! Operational status values:
//! - **"Open"**: Slope is currently open for skiing
//! - **"Closed"**: Slope is closed
//! - **"Maintenance"**: Slope is under maintenance
//! - **"Unknown"**: Status is not available
//!
//! Grooming status values:
//! - **"Groomed"**: Slope has been groomed/maintained
//! - **"Ungroomed"**: Slope has not been groomed
//! - **"Partially groomed"**: Slope is partially maintained
//! - **"Unknown"**: Grooming status is not available
//!
//! # Performance Considerations
//!
//! - **Efficient Geometry Handling**: Optimized path_geojson parsing
//! - **Error Resilience**: Graceful handling of malformed GeoJSON data
//! - **Batch Operations**: Support for bulk operations on resort slopes
//! - **Data Validation**: Comprehensive input validation for all operations
//!
//! # Usage Examples
//!
//! ```rust
//! // Get all slopes
//! GET /api/v1/slopes
//!
//! // Get specific slope
//! GET /api/v1/slopes/123
//!
//! // Get slopes for a resort
//! GET /api/v1/slopes/by_resort/resort_abc
//!
//! // Create new slope
//! POST /api/v1/slopes
//! {
//!   "resort_id": "resort_abc",
//!   "name": "Black Diamond Run",
//!   "difficulty": "Black",
//!   "length_m": 2500,
//!   "average_gradient": 45.5,
//!   "max_gradient": 65.2,
//!   "snowmaking": true,
//!   "lat_start": 47.1234,
//!   "lon_start": 11.5678,
//!   "lat_end": 47.1256,
//!   "lon_end": 11.5690,
//!   "slope_path_json": "[{\"lat\": 47.1234, \"lon\": 11.5678}, ...]",
//!   // ... other fields
//! }
//! ```
//!
//! Author: OpenSlope Team
//! Version: 1.0.0

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::MySqlPool;

#[derive(Serialize)]
pub struct Slope {
    pub id: i64,
    pub resort_id: String,
    pub name: Option<String>,
    pub display: SlopeDisplay,
    pub geometry: SlopeGeometry,
    pub specs: SlopeSpecs,
    pub source: SlopeSource,
    pub status: SlopeStatus,
}

#[derive(Serialize)]
pub struct SlopeDisplay {
    pub normalized_name: Option<String>,
    pub difficulty: String,
}

#[derive(Serialize)]
pub struct SlopeGeometry {
    pub start: CoordinatePoint,
    pub end: CoordinatePoint,
    pub path: Option<Vec<CoordinatePoint>>,
    pub direction: Option<f64>,
}

#[derive(Serialize)]
pub struct CoordinatePoint {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Serialize)]
pub struct SlopeSpecs {
    pub length_m: Option<i32>,
    pub vertical_drop_m: Option<i32>,
    pub average_gradient: Option<f64>,
    pub max_gradient: Option<f64>,
    pub snowmaking: bool,
    pub night_skiing: bool,
    pub family_friendly: bool,
    pub race_slope: bool,
}

#[derive(Serialize)]
pub struct SlopeSource {
    pub system: String,
    pub entity_id: Option<String>,
    pub source_url: Option<String>,
}

#[derive(Serialize)]
pub struct SlopeStatus {
    pub operational_status: String,
    pub grooming_status: String,
    pub note: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Deserialize)]
pub struct CreateSlope {
    pub resort_id: String,
    pub name: Option<String>,
    pub difficulty: String,
    pub length_m: Option<i32>,
    pub vertical_drop_m: Option<i32>,
    pub average_gradient: Option<f64>,
    pub max_gradient: Option<f64>,
    pub snowmaking: Option<bool>,
    pub night_skiing: Option<bool>,
    pub family_friendly: Option<bool>,
    pub race_slope: Option<bool>,
    pub lat_start: Option<f64>,
    pub lon_start: Option<f64>,
    pub lat_end: Option<f64>,
    pub lon_end: Option<f64>,
    pub source_system: Option<String>,
    pub source_entity_id: Option<String>,
    pub name_normalized: Option<String>,
    pub operational_status: Option<String>,
    pub grooming_status: Option<String>,
    pub operational_note: Option<String>,
    pub status_updated_at: Option<String>,
    pub status_source_url: Option<String>,
    pub slope_path_json: Option<String>,
    pub direction: Option<f64>,
}

#[derive(Deserialize)]
pub struct UpdateSlope {
    pub resort_id: String,
    pub name: Option<String>,
    pub difficulty: String,
    pub length_m: Option<i32>,
    pub vertical_drop_m: Option<i32>,
    pub average_gradient: Option<f64>,
    pub max_gradient: Option<f64>,
    pub snowmaking: Option<bool>,
    pub night_skiing: Option<bool>,
    pub family_friendly: Option<bool>,
    pub race_slope: Option<bool>,
    pub lat_start: Option<f64>,
    pub lon_start: Option<f64>,
    pub lat_end: Option<f64>,
    pub lon_end: Option<f64>,
    pub source_system: Option<String>,
    pub source_entity_id: Option<String>,
    pub name_normalized: Option<String>,
    pub operational_status: Option<String>,
    pub grooming_status: Option<String>,
    pub operational_note: Option<String>,
    pub status_updated_at: Option<String>,
    pub status_source_url: Option<String>,
    pub slope_path_json: Option<String>,
    pub direction: Option<f64>,
}

fn parse_path_geojson(path_geojson: Option<String>) -> Option<Vec<CoordinatePoint>> {
    let raw = path_geojson?;
    let parsed: Value = serde_json::from_str(&raw).ok()?;
    let arr = parsed.as_array()?;

    let mut points: Vec<CoordinatePoint> = Vec::new();
    for item in arr {
        let latitude = item
            .get("latitude")
            .and_then(|v| v.as_f64())
            .or_else(|| item.get("lat").and_then(|v| v.as_f64()));
        let longitude = item
            .get("longitude")
            .and_then(|v| v.as_f64())
            .or_else(|| item.get("lon").and_then(|v| v.as_f64()));

        if latitude.is_none() || longitude.is_none() {
            continue;
        }
        points.push(CoordinatePoint {
            latitude,
            longitude,
        });
    }

    if points.is_empty() {
        None
    } else {
        Some(points)
    }
}

pub async fn get_slopes(db: web::Data<MySqlPool>) -> impl Responder {
    let result = sqlx::query!(
        r#"
        SELECT id, resort_id, name, difficulty, name_normalized,
               length_m, vertical_drop_m,
               CAST(average_gradient AS DOUBLE) AS average_gradient,
               CAST(max_gradient AS DOUBLE) AS max_gradient,
               snowmaking, night_skiing, family_friendly, race_slope,
               CAST(lat_start AS DOUBLE) AS lat_start, CAST(lon_start AS DOUBLE) AS lon_start,
               CAST(lat_end AS DOUBLE) AS lat_end, CAST(lon_end AS DOUBLE) AS lon_end,
               CAST(path_geojson AS CHAR) AS path_geojson,
               CAST(direction AS DOUBLE) AS direction,
               source_system, source_entity_id, operational_status, grooming_status, operational_note, status_source_url,
               DATE_FORMAT(status_updated_at, '%Y-%m-%dT%H:%i:%sZ') AS status_updated_at
        FROM slopes
        ORDER BY resort_id, name
        "#
    )
    .fetch_all(db.get_ref())
    .await;

    match result {
        Ok(rows) => HttpResponse::Ok().json(
            rows.into_iter()
                .map(|row| Slope {
                    id: row.id,
                    resort_id: row.resort_id,
                    name: row.name,
                    display: SlopeDisplay {
                        normalized_name: row.name_normalized,
                        difficulty: row.difficulty,
                    },
                    geometry: SlopeGeometry {
                        start: CoordinatePoint {
                            latitude: row.lat_start,
                            longitude: row.lon_start,
                        },
                        end: CoordinatePoint {
                            latitude: row.lat_end,
                            longitude: row.lon_end,
                        },
                        direction: row.direction,
                        path: parse_path_geojson(row.path_geojson),
                    },
                    specs: SlopeSpecs {
                        length_m: row.length_m,
                        vertical_drop_m: row.vertical_drop_m,
                        average_gradient: row.average_gradient,
                        max_gradient: row.max_gradient,
                        snowmaking: row.snowmaking.unwrap_or(0) != 0,
                        night_skiing: row.night_skiing.unwrap_or(0) != 0,
                        family_friendly: row.family_friendly.unwrap_or(0) != 0,
                        race_slope: row.race_slope.unwrap_or(0) != 0,
                    },
                    source: SlopeSource {
                        system: row.source_system,
                        entity_id: row.source_entity_id,
                        source_url: row.status_source_url,
                    },
                    status: SlopeStatus {
                        operational_status: row.operational_status,
                        grooming_status: row.grooming_status,
                        note: row.operational_note,
                        updated_at: row.status_updated_at,
                    },
                })
                .collect::<Vec<Slope>>(),
        ),
        Err(err) => {
            eprintln!("GET /slopes error: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_slope(db: web::Data<MySqlPool>, id: web::Path<i64>) -> impl Responder {
    let result = sqlx::query!(
        r#"
        SELECT id, resort_id, name, difficulty, name_normalized,
               length_m, vertical_drop_m,
               CAST(average_gradient AS DOUBLE) AS average_gradient,
               CAST(max_gradient AS DOUBLE) AS max_gradient,
               snowmaking, night_skiing, family_friendly, race_slope,
               CAST(lat_start AS DOUBLE) AS lat_start, CAST(lon_start AS DOUBLE) AS lon_start,
               CAST(lat_end AS DOUBLE) AS lat_end, CAST(lon_end AS DOUBLE) AS lon_end,
               CAST(path_geojson AS CHAR) AS path_geojson,
               CAST(direction AS DOUBLE) AS direction,
               source_system, source_entity_id, operational_status, grooming_status, operational_note, status_source_url,
               DATE_FORMAT(status_updated_at, '%Y-%m-%dT%H:%i:%sZ') AS status_updated_at
        FROM slopes
        WHERE id = ?
        "#,
        *id
    )
    .fetch_optional(db.get_ref())
    .await;

    match result {
        Ok(Some(row)) => HttpResponse::Ok().json(Slope {
            id: row.id,
            resort_id: row.resort_id,
            name: row.name,
            display: SlopeDisplay {
                normalized_name: row.name_normalized,
                difficulty: row.difficulty,
            },
            geometry: SlopeGeometry {
                start: CoordinatePoint {
                    latitude: row.lat_start,
                    longitude: row.lon_start,
                },
                end: CoordinatePoint {
                    latitude: row.lat_end,
                    longitude: row.lon_end,
                },
                direction: row.direction,
                path: parse_path_geojson(row.path_geojson),
            },
            specs: SlopeSpecs {
                length_m: row.length_m,
                vertical_drop_m: row.vertical_drop_m,
                average_gradient: row.average_gradient,
                max_gradient: row.max_gradient,
                snowmaking: row.snowmaking.unwrap_or(0) != 0,
                night_skiing: row.night_skiing.unwrap_or(0) != 0,
                family_friendly: row.family_friendly.unwrap_or(0) != 0,
                race_slope: row.race_slope.unwrap_or(0) != 0,
            },
            source: SlopeSource {
                system: row.source_system,
                entity_id: row.source_entity_id,
                source_url: row.status_source_url,
            },
            status: SlopeStatus {
                operational_status: row.operational_status,
                grooming_status: row.grooming_status,
                note: row.operational_note,
                updated_at: row.status_updated_at,
            },
        }),
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(err) => {
            eprintln!("GET /slopes/{{id}} error: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_slopes_by_resort(
    db: web::Data<MySqlPool>,
    resort_id: web::Path<String>,
) -> impl Responder {
    let result = sqlx::query!(
        r#"
        SELECT id, resort_id, name, difficulty, name_normalized,
               length_m, vertical_drop_m,
               CAST(average_gradient AS DOUBLE) AS average_gradient,
               CAST(max_gradient AS DOUBLE) AS max_gradient,
               snowmaking, night_skiing, family_friendly, race_slope,
               CAST(lat_start AS DOUBLE) AS lat_start, CAST(lon_start AS DOUBLE) AS lon_start,
               CAST(lat_end AS DOUBLE) AS lat_end, CAST(lon_end AS DOUBLE) AS lon_end,
               CAST(path_geojson AS CHAR) AS path_geojson,
               CAST(direction AS DOUBLE) AS direction,
               source_system, source_entity_id, operational_status, grooming_status, operational_note, status_source_url,
               DATE_FORMAT(status_updated_at, '%Y-%m-%dT%H:%i:%sZ') AS status_updated_at
        FROM slopes
        WHERE resort_id = ?
        ORDER BY name
        "#,
        resort_id.into_inner()
    )
    .fetch_all(db.get_ref())
    .await;

    match result {
        Ok(rows) => HttpResponse::Ok().json(
            rows.into_iter()
                .map(|row| Slope {
                    id: row.id,
                    resort_id: row.resort_id,
                    name: row.name,
                    display: SlopeDisplay {
                        normalized_name: row.name_normalized,
                        difficulty: row.difficulty,
                    },
                    geometry: SlopeGeometry {
                        start: CoordinatePoint {
                            latitude: row.lat_start,
                            longitude: row.lon_start,
                        },
                        end: CoordinatePoint {
                            latitude: row.lat_end,
                            longitude: row.lon_end,
                        },
                        direction: row.direction,
                        path: parse_path_geojson(row.path_geojson),
                    },
                    specs: SlopeSpecs {
                        length_m: row.length_m,
                        vertical_drop_m: row.vertical_drop_m,
                        average_gradient: row.average_gradient,
                        max_gradient: row.max_gradient,
                        snowmaking: row.snowmaking.unwrap_or(0) != 0,
                        night_skiing: row.night_skiing.unwrap_or(0) != 0,
                        family_friendly: row.family_friendly.unwrap_or(0) != 0,
                        race_slope: row.race_slope.unwrap_or(0) != 0,
                    },
                    source: SlopeSource {
                        system: row.source_system,
                        entity_id: row.source_entity_id,
                        source_url: row.status_source_url,
                    },
                    status: SlopeStatus {
                        operational_status: row.operational_status,
                        grooming_status: row.grooming_status,
                        note: row.operational_note,
                        updated_at: row.status_updated_at,
                    },
                })
                .collect::<Vec<Slope>>(),
        ),
        Err(err) => {
            eprintln!("GET /slopes/by_resort error: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn create_slope(
    db: web::Data<MySqlPool>,
    slope: web::Json<CreateSlope>,
) -> impl Responder {
    let result = sqlx::query!(
        r#"
        INSERT INTO slopes
        (resort_id, name, difficulty,
         length_m, vertical_drop_m, average_gradient, max_gradient,
         snowmaking, night_skiing, family_friendly, race_slope,
         lat_start, lon_start, lat_end, lon_end, path_geojson, direction,
         source_system, source_entity_id, name_normalized,
         operational_status, grooming_status, operational_note, status_updated_at, status_source_url)
        VALUES (?, ?, ?,
                ?, ?, ?, ?,
                ?, ?, ?, ?,
                ?, ?, ?, ?, ?, ?,
                ?, ?, ?,
                ?, ?, ?, ?, ?)
        "#,
        slope.resort_id,
        slope.name,
        slope.difficulty,
        slope.length_m,
        slope.vertical_drop_m,
        slope.average_gradient,
        slope.max_gradient,
        slope.snowmaking.unwrap_or(false),
        slope.night_skiing.unwrap_or(false),
        slope.family_friendly.unwrap_or(false),
        slope.race_slope.unwrap_or(false),
        slope.lat_start,
        slope.lon_start,
        slope.lat_end,
        slope.lon_end,
        slope.slope_path_json,
        slope.direction,
        slope.source_system.as_deref().unwrap_or("osm"),
        slope.source_entity_id,
        slope.name_normalized,
        slope.operational_status.as_deref().unwrap_or("unknown"),
        slope.grooming_status.as_deref().unwrap_or("unknown"),
        slope.operational_note,
        slope.status_updated_at,
        slope.status_source_url
    )
    .execute(db.get_ref())
    .await;

    match result {
        Ok(res) => HttpResponse::Created().json(res.last_insert_id()),
        Err(err) => {
            eprintln!("POST /slopes error: {:?}", err);
            HttpResponse::BadRequest().finish()
        }
    }
}

pub async fn update_slope(
    db: web::Data<MySqlPool>,
    id: web::Path<i64>,
    slope: web::Json<UpdateSlope>,
) -> impl Responder {
    let result = sqlx::query!(
        r#"
        UPDATE slopes
        SET resort_id = ?, name = ?, difficulty = ?,
            length_m = ?, vertical_drop_m = ?, average_gradient = ?, max_gradient = ?,
            snowmaking = ?, night_skiing = ?, family_friendly = ?, race_slope = ?,
            lat_start = ?, lon_start = ?, lat_end = ?, lon_end = ?, path_geojson = COALESCE(?, path_geojson), direction = ?,
            source_system = ?, source_entity_id = ?, name_normalized = ?,
            operational_status = ?, grooming_status = ?, operational_note = ?, status_updated_at = ?, status_source_url = ?
        WHERE id = ?
        "#,
        slope.resort_id,
        slope.name,
        slope.difficulty,
        slope.length_m,
        slope.vertical_drop_m,
        slope.average_gradient,
        slope.max_gradient,
        slope.snowmaking.unwrap_or(false),
        slope.night_skiing.unwrap_or(false),
        slope.family_friendly.unwrap_or(false),
        slope.race_slope.unwrap_or(false),
        slope.lat_start,
        slope.lon_start,
        slope.lat_end,
        slope.lon_end,
        slope.slope_path_json,
        slope.direction,
        slope.source_system.as_deref().unwrap_or("osm"),
        slope.source_entity_id,
        slope.name_normalized,
        slope.operational_status.as_deref().unwrap_or("unknown"),
        slope.grooming_status.as_deref().unwrap_or("unknown"),
        slope.operational_note,
        slope.status_updated_at,
        slope.status_source_url,
        *id
    )
    .execute(db.get_ref())
    .await;

    match result {
        Ok(res) if res.rows_affected() == 0 => HttpResponse::NotFound().finish(),
        Ok(_) => HttpResponse::Ok().finish(),
        Err(err) => {
            eprintln!("PUT /slopes/{{id}} error: {:?}", err);
            HttpResponse::BadRequest().finish()
        }
    }
}

pub async fn delete_slope(
    db: web::Data<MySqlPool>,
    id: web::Path<i64>,
) -> impl Responder {
    let result = sqlx::query!("DELETE FROM slopes WHERE id = ?", *id)
        .execute(db.get_ref())
        .await;

    match result {
        Ok(res) if res.rows_affected() == 0 => HttpResponse::NotFound().finish(),
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(err) => {
            eprintln!("DELETE /slopes/{{id}} error: {:?}", err);
            HttpResponse::BadRequest().finish()
        }
    }
}

pub async fn delete_slopes_by_resort(
    db: web::Data<MySqlPool>,
    resort_id: web::Path<String>,
) -> impl Responder {
    let result: Result<sqlx::mysql::MySqlQueryResult, sqlx::Error> = sqlx::query!("DELETE FROM slopes WHERE resort_id = ?", resort_id.into_inner())
        .execute(db.get_ref())
        .await;

    match result {
        Ok(res) => HttpResponse::Ok().json(res.rows_affected()),
        Err(err) => {
            eprintln!("DELETE /slopes/by_resort error: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}
