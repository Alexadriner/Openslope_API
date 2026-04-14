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

use actix_web::{HttpResponse, Responder, web};
use serde::Serialize;
use serde_json::Value;
use sqlx::MySqlPool;

#[derive(Serialize)]
pub struct Slope {
    pub id: String,
    pub resort_id: Option<String>,
    pub name: Option<String>,
    pub display: SlopeDisplay,
    pub geometry: SlopeGeometry,
    pub specs: SlopeSpecs,
    pub source: Option<SlopeSource>,
    pub description: Option<String>,
    pub status: SlopeStatus,
}

#[derive(Serialize)]
pub struct SlopeDisplay {
    pub normalized_name: Option<String>,
    pub difficulty: Option<String>,
}

#[derive(Serialize)]
pub struct SlopeGeometry {
    pub geometry_type: Option<String>,
    pub start: CoordinatePoint,
    pub end: CoordinatePoint,
    pub path: Option<Vec<CoordinatePoint>>,
}

#[derive(Clone, Serialize)]
pub struct CoordinatePoint {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Serialize)]
pub struct SlopeSpecs {
    pub snowmaking: bool,
    pub lit: bool,
    pub patrolled: bool,
    pub difficulty_convention: Option<String>,
    pub grooming: Option<String>,
}

#[derive(Serialize)]
pub struct SlopeSource {
    pub system: Option<String>,
    pub entity_id: Option<String>,
}

#[derive(Serialize)]
pub struct SlopeStatus {
    pub status: Option<String>,
    pub grooming_status: Option<String>,
    pub note: Option<String>,
    pub updated_at: Option<String>,
}

fn parse_geojson_geometry(raw: Option<String>) -> (Option<String>, Option<Vec<CoordinatePoint>>) {
    match raw.and_then(|value| serde_json::from_str::<Value>(&value).ok()) {
        Some(parsed) => {
            let geometry_type = parsed
                .get("type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let path = parsed.get("coordinates").and_then(flatten_coordinates);
            (geometry_type, path)
        }
        None => (None, None),
    }
}

fn flatten_coordinates(value: &Value) -> Option<Vec<CoordinatePoint>> {
    if let Value::Array(arr) = value {
        if arr.is_empty() {
            return None;
        }

        if arr.iter().all(|item| item.is_number()) {
            return coordinate_point_from_array(arr);
        }

        let mut points = Vec::new();
        for item in arr {
            if let Some(mut nested) = flatten_coordinates(item) {
                points.append(&mut nested);
            }
        }

        if points.is_empty() {
            None
        } else {
            Some(points)
        }
    } else {
        None
    }
}

fn coordinate_point_from_array(arr: &[Value]) -> Option<Vec<CoordinatePoint>> {
    if arr.len() >= 2 {
        let lon = arr[0].as_f64();
        let lat = arr[1].as_f64();
        if latitude_and_longitude_valid(lat, lon) {
            return Some(vec![CoordinatePoint {
                latitude: lat,
                longitude: lon,
            }]);
        }
    }
    None
}

fn latitude_and_longitude_valid(lat: Option<f64>, lon: Option<f64>) -> bool {
    lat.is_some() && lon.is_some()
}

fn parse_first_ski_area(raw: Option<String>) -> Option<String> {
    let raw = raw?;
    let parsed: Value = serde_json::from_str(&raw).ok()?;

    if let Some(array) = parsed.as_array() {
        for item in array {
            if let Some(value) = item.as_str() {
                return Some(value.to_string());
            }
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                return Some(id.to_string());
            }
        }
    }

    parsed.as_str().map(|s| s.to_string())
}

fn parse_source_info(raw: Option<String>) -> Option<SlopeSource> {
    let raw = raw?;
    let parsed: Value = serde_json::from_str(&raw).ok()?;
    let source = if parsed.is_array() {
        parsed.get(0)?
    } else {
        &parsed
    };

    if !source.is_object() {
        return None;
    }

    let system = source
        .get("system")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let entity_id = source
        .get("entity_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    if system.is_none() && entity_id.is_none() {
        return None;
    }

    Some(SlopeSource { system, entity_id })
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
        SELECT id, name, difficulty, status, grooming, snowmaking, lit, patrolled,
               difficulty_convention, description,
               CAST(geometry AS CHAR) AS geometry_json,
               CAST(sources AS CHAR) AS sources_json,
               CAST(ski_areas AS CHAR) AS ski_areas_json
        FROM geojson_runs
        ORDER BY name
        "#
    )
    .fetch_all(db.get_ref())
    .await;

    match result {
        Ok(rows) => HttpResponse::Ok().json(
            rows.into_iter()
                .map(|row| {
                    let (geometry_type, path) = parse_geojson_geometry(row.geometry_json);
                    let start = path
                        .as_ref()
                        .and_then(|points| points.first().cloned())
                        .unwrap_or(CoordinatePoint {
                            latitude: None,
                            longitude: None,
                        });
                    let end = path
                        .as_ref()
                        .and_then(|points| points.last().cloned())
                        .unwrap_or(CoordinatePoint {
                            latitude: None,
                            longitude: None,
                        });

                    Slope {
                        id: row.id,
                        resort_id: parse_first_ski_area(row.ski_areas_json),
                        name: row.name,
                        display: SlopeDisplay {
                            normalized_name: None,
                            difficulty: row.difficulty,
                        },
                        geometry: SlopeGeometry {
                            geometry_type,
                            start,
                            end,
                            path,
                        },
                        specs: SlopeSpecs {
                            snowmaking: row.snowmaking.unwrap_or(false),
                            lit: row.lit.unwrap_or(false),
                            patrolled: row.patrolled.unwrap_or(false),
                            difficulty_convention: row.difficulty_convention,
                            grooming: row.grooming,
                        },
                        source: parse_source_info(row.sources_json),
                        description: row.description,
                        status: SlopeStatus {
                            status: row.status,
                            grooming_status: row.grooming,
                            note: None,
                            updated_at: None,
                        },
                    }
                })
                .collect::<Vec<Slope>>(),
        ),
        Err(err) => {
            eprintln!("GET /slopes error: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_slope(db: web::Data<MySqlPool>, id: web::Path<String>) -> impl Responder {
    let result = sqlx::query!(
        r#"
        SELECT id, name, difficulty, status, grooming, snowmaking, lit, patrolled,
               difficulty_convention, description,
               CAST(geometry AS CHAR) AS geometry_json,
               CAST(sources AS CHAR) AS sources_json,
               CAST(ski_areas AS CHAR) AS ski_areas_json
        FROM geojson_runs
        WHERE id = ?
        "#,
        id.into_inner()
    )
    .fetch_optional(db.get_ref())
    .await;

    match result {
        Ok(Some(row)) => {
            let (geometry_type, path) = parse_geojson_geometry(row.geometry_json);
            let start = path
                .as_ref()
                .and_then(|points| points.first().cloned())
                .unwrap_or(CoordinatePoint {
                    latitude: None,
                    longitude: None,
                });
            let end = path
                .as_ref()
                .and_then(|points| points.last().cloned())
                .unwrap_or(CoordinatePoint {
                    latitude: None,
                    longitude: None,
                });

            HttpResponse::Ok().json(Slope {
                id: row.id,
                resort_id: parse_first_ski_area(row.ski_areas_json),
                name: row.name,
                display: SlopeDisplay {
                    normalized_name: None,
                    difficulty: row.difficulty,
                },
                geometry: SlopeGeometry {
                    geometry_type,
                    start,
                    end,
                    path,
                },
                specs: SlopeSpecs {
                    snowmaking: row.snowmaking.unwrap_or(false),
                    lit: row.lit.unwrap_or(false),
                    patrolled: row.patrolled.unwrap_or(false),
                    difficulty_convention: row.difficulty_convention,
                    grooming: row.grooming,
                },
                source: parse_source_info(row.sources_json),
                description: row.description,
                status: SlopeStatus {
                    status: row.status,
                    grooming_status: row.grooming,
                    note: None,
                    updated_at: None,
                },
            })
        }
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
        SELECT id, name, difficulty, status, grooming, snowmaking, lit, patrolled,
               difficulty_convention, description,
               CAST(geometry AS CHAR) AS geometry_json,
               CAST(sources AS CHAR) AS sources_json,
               CAST(ski_areas AS CHAR) AS ski_areas_json
        FROM geojson_runs
        WHERE JSON_CONTAINS(ski_areas, JSON_QUOTE(?))
        ORDER BY name
        "#,
        resort_id.into_inner()
    )
    .fetch_all(db.get_ref())
    .await;

    match result {
        Ok(rows) => HttpResponse::Ok().json(
            rows.into_iter()
                .map(|row| {
                    let (geometry_type, path) = parse_geojson_geometry(row.geometry_json);
                    let start = path
                        .as_ref()
                        .and_then(|points| points.first().cloned())
                        .unwrap_or(CoordinatePoint {
                            latitude: None,
                            longitude: None,
                        });
                    let end = path
                        .as_ref()
                        .and_then(|points| points.last().cloned())
                        .unwrap_or(CoordinatePoint {
                            latitude: None,
                            longitude: None,
                        });

                    Slope {
                        id: row.id,
                        resort_id: parse_first_ski_area(row.ski_areas_json),
                        name: row.name,
                        display: SlopeDisplay {
                            normalized_name: None,
                            difficulty: row.difficulty,
                        },
                        geometry: SlopeGeometry {
                            geometry_type,
                            start,
                            end,
                            path,
                        },
                        specs: SlopeSpecs {
                            snowmaking: row.snowmaking.unwrap_or(false),
                            lit: row.lit.unwrap_or(false),
                            patrolled: row.patrolled.unwrap_or(false),
                            difficulty_convention: row.difficulty_convention,
                            grooming: row.grooming,
                        },
                        source: parse_source_info(row.sources_json),
                        description: row.description,
                        status: SlopeStatus {
                            status: row.status,
                            grooming_status: row.grooming,
                            note: None,
                            updated_at: None,
                        },
                    }
                })
                .collect::<Vec<Slope>>(),
        ),
        Err(err) => {
            eprintln!("GET /slopes/by_resort error: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn create_slope(_db: web::Data<MySqlPool>) -> impl Responder {
    HttpResponse::MethodNotAllowed()
        .body("Create operations are not supported for geojson import slope data")
}

pub async fn update_slope(_db: web::Data<MySqlPool>, _id: web::Path<String>) -> impl Responder {
    HttpResponse::MethodNotAllowed()
        .body("Update operations are not supported for geojson import slope data")
}

pub async fn delete_slope(_db: web::Data<MySqlPool>, _id: web::Path<String>) -> impl Responder {
    HttpResponse::MethodNotAllowed()
        .body("Delete operations are not supported for geojson import slope data")
}

pub async fn delete_slopes_by_resort(
    _db: web::Data<MySqlPool>,
    _resort_id: web::Path<String>,
) -> impl Responder {
    HttpResponse::MethodNotAllowed()
        .body("Delete operations are not supported for geojson import slope data")
}
