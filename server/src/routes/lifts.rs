//! OpenSlope API Lifts Routes
//!
//! This module handles all HTTP requests related to ski lift management in the
//! OpenSlope API. It provides comprehensive CRUD operations for individual lifts
//! including detailed information about their specifications, geometry, and status.
//!
//! # Route Overview
//!
//! The lifts module provides the following endpoints:
//!
//! - **GET /lifts**: List all lifts with detailed information
//! - **GET /lifts/{id}**: Get detailed information about a specific lift
//! - **GET /lifts/by_resort/{resort_id}**: Get all lifts for a specific resort
//! - **POST /lifts**: Create a new lift
//! - **PUT /lifts/{id}**: Update an existing lift
//! - **DELETE /lifts/{id}**: Delete a lift
//! - **DELETE /lifts/by_resort/{resort_id}**: Delete all lifts for a resort
//!
//! # Data Models
//!
//! The module defines several data structures for handling lift information:
//!
//! - **Lift**: Complete response model with nested data structures
//! - **LiftDisplay**: Display-related information (name, type)
//! - **LiftGeometry**: Geographical coordinates (start/end points)
//! - **LiftSpecs**: Technical specifications (capacity, seats, year built)
//! - **LiftSource**: Source system information and entity references
//! - **LiftStatus**: Operational status and timing information
//! - **CreateLift/UpdateLift**: Input models for creation and updates
//!
//! # Key Features
//!
//! - **Geographical Data**: Precise start/end coordinates for lift mapping
//! - **Technical Specifications**: Capacity, seats, year built, altitude data
//! - **Status Management**: Operational status, planned times, notes
//! - **Source Tracking**: Integration with external data sources (OSM, etc.)
//! - **Resort Association**: All lifts are linked to specific resorts
//!
//! # Coordinate System
//!
//! - Latitude and longitude use WGS84 coordinate system
//! - Altitude measurements are in meters above sea level
//! - Coordinates are cast to DOUBLE precision for accuracy
//!
//! # Lift Types
//!
//! Common lift types supported:
//! - **Chairlift**: Detachable or fixed-grip chairlifts
//! - **Gondola**: Enclosed cabin lifts
//! - **T-bar**: Surface lift with T-shaped bars
//! - **Surface lift**: Magic carpet or similar surface lifts
//! - **Funitel**: Cable transport system with two fixed ropes
//!
//! # Status Values
//!
//! Operational status values:
//! - **"Open"**: Lift is currently operating
//! - **"Closed"**: Lift is not operating
//! - **"Maintenance"**: Lift is under maintenance
//! - **"Unknown"**: Status is not available
//!
//! # Performance Considerations
//!
//! - **Efficient Queries**: Optimized SQL with proper column selection
//! - **Error Handling**: Comprehensive error logging and user-friendly responses
//! - **Data Validation**: Input validation for all create/update operations
//! - **Batch Operations**: Support for bulk operations on resort lifts
//!
//! # Usage Examples
//!
//! ```rust
//! // Get all lifts
//! GET /api/v1/lifts
//!
//! // Get specific lift
//! GET /api/v1/lifts/123
//!
//! // Get lifts for a resort
//! GET /api/v1/lifts/by_resort/resort_abc
//!
//! // Create new lift
//! POST /api/v1/lifts
//! {
//!   "resort_id": "resort_abc",
//!   "name": "Main Chair",
//!   "lift_type": "Chairlift",
//!   "capacity_per_hour": 2000,
//!   "seats": 4,
//!   "lat_start": 47.1234,
//!   "lon_start": 11.5678,
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
pub struct Lift {
    pub id: String,
    pub resort_id: Option<String>,
    pub name: Option<String>,
    pub display: LiftDisplay,
    pub geometry: LiftGeometry,
    pub specs: LiftSpecs,
    pub source: Option<LiftSource>,
    pub description: Option<String>,
    pub status: LiftStatus,
}

#[derive(Serialize)]
pub struct LiftDisplay {
    pub normalized_name: Option<String>,
    pub lift_type: Option<String>,
}

#[derive(Serialize)]
pub struct LiftGeometry {
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
pub struct LiftSpecs {
    pub capacity: Option<i32>,
    pub duration_minutes: Option<f64>,
    pub detachable: bool,
    pub heating: bool,
    pub bubble: bool,
}

#[derive(Serialize)]
pub struct LiftSource {
    pub system: Option<String>,
    pub entity_id: Option<String>,
}

#[derive(Serialize)]
pub struct LiftStatus {
    pub status: Option<String>,
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

fn parse_source_info(raw: Option<String>) -> Option<LiftSource> {
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

    Some(LiftSource { system, entity_id })
}

fn parse_heating(value: Option<String>) -> bool {
    value
        .map(|s| matches!(s.to_ascii_lowercase().as_str(), "heated" | "true" | "yes"))
        .unwrap_or(false)
}

pub async fn get_lifts(db: web::Data<MySqlPool>) -> impl Responder {
    let result = sqlx::query!(
        r#"
        SELECT id, name, lift_type, `type` AS feature_type,
               capacity, CAST(duration AS DOUBLE) AS duration, detachable, heating, bubble,
               description, status,
               CAST(geometry AS CHAR) AS geometry_json,
               CAST(sources AS CHAR) AS sources_json,
               CAST(ski_areas AS CHAR) AS ski_areas_json
        FROM geojson_lifts
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

                    Lift {
                        id: row.id,
                        resort_id: parse_first_ski_area(row.ski_areas_json),
                        name: row.name,
                        display: LiftDisplay {
                            normalized_name: None,
                            lift_type: row.lift_type.or(row.feature_type),
                        },
                        geometry: LiftGeometry {
                            geometry_type,
                            start,
                            end,
                            path,
                        },
                        specs: LiftSpecs {
                            capacity: row.capacity,
                            duration_minutes: row.duration,
                            detachable: row.detachable.map(|v| v != 0).unwrap_or(false),
                            heating: parse_heating(row.heating),
                            bubble: row.bubble.map(|v| v != 0).unwrap_or(false),
                        },
                        source: parse_source_info(row.sources_json),
                        description: row.description,
                        status: LiftStatus {
                            status: row.status,
                            note: None,
                            updated_at: None,
                        },
                    }
                })
                .collect::<Vec<Lift>>(),
        ),
        Err(err) => {
            eprintln!("GET /lifts error: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_lift(db: web::Data<MySqlPool>, id: web::Path<String>) -> impl Responder {
    let result = sqlx::query!(
        r#"
        SELECT id, name, lift_type, `type` AS feature_type,
               capacity, CAST(duration AS DOUBLE) AS duration, detachable, heating, bubble,
               description, status,
               CAST(geometry AS CHAR) AS geometry_json,
               CAST(sources AS CHAR) AS sources_json,
               CAST(ski_areas AS CHAR) AS ski_areas_json
        FROM geojson_lifts
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

            HttpResponse::Ok().json(Lift {
                id: row.id,
                resort_id: parse_first_ski_area(row.ski_areas_json),
                name: row.name,
                display: LiftDisplay {
                    normalized_name: None,
                    lift_type: row.lift_type.or(row.feature_type),
                },
                geometry: LiftGeometry {
                    geometry_type,
                    start,
                    end,
                    path,
                },
                specs: LiftSpecs {
                    capacity: row.capacity,
                    duration_minutes: row.duration,
                    detachable: row.detachable.map(|v| v != 0).unwrap_or(false),
                    heating: parse_heating(row.heating),
                    bubble: row.bubble.map(|v| v != 0).unwrap_or(false),
                },
                source: parse_source_info(row.sources_json),
                description: row.description,
                status: LiftStatus {
                    status: row.status,
                    note: None,
                    updated_at: None,
                },
            })
        }
        Ok(None) => HttpResponse::NotFound().finish(),
        Err(err) => {
            eprintln!("GET /lifts/{{id}} error: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_lifts_by_resort(
    db: web::Data<MySqlPool>,
    resort_id: web::Path<String>,
) -> impl Responder {
    let result = sqlx::query!(
        r#"
        SELECT id, name, lift_type, `type` AS feature_type,
               capacity, CAST(duration AS DOUBLE) AS duration, detachable, heating, bubble,
               description, status,
               CAST(geometry AS CHAR) AS geometry_json,
               CAST(sources AS CHAR) AS sources_json,
               CAST(ski_areas AS CHAR) AS ski_areas_json
        FROM geojson_lifts
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

                    Lift {
                        id: row.id,
                        resort_id: parse_first_ski_area(row.ski_areas_json),
                        name: row.name,
                        display: LiftDisplay {
                            normalized_name: None,
                            lift_type: row.lift_type.or(row.feature_type),
                        },
                        geometry: LiftGeometry {
                            geometry_type,
                            start,
                            end,
                            path,
                        },
                        specs: LiftSpecs {
                            capacity: row.capacity,
                            duration_minutes: row.duration,
                            detachable: row.detachable.map(|v| v != 0).unwrap_or(false),
                            heating: parse_heating(row.heating),
                            bubble: row.bubble.map(|v| v != 0).unwrap_or(false),
                        },
                        source: parse_source_info(row.sources_json),
                        description: row.description,
                        status: LiftStatus {
                            status: row.status,
                            note: None,
                            updated_at: None,
                        },
                    }
                })
                .collect::<Vec<Lift>>(),
        ),
        Err(err) => {
            eprintln!("GET /lifts/by_resort error: {:?}", err);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn create_lift(_db: web::Data<MySqlPool>) -> impl Responder {
    HttpResponse::MethodNotAllowed()
        .body("Create operations are not supported for geojson import lift data")
}

pub async fn update_lift(_db: web::Data<MySqlPool>, _id: web::Path<String>) -> impl Responder {
    HttpResponse::MethodNotAllowed()
        .body("Update operations are not supported for geojson import lift data")
}

pub async fn delete_lift(_db: web::Data<MySqlPool>, _id: web::Path<String>) -> impl Responder {
    HttpResponse::MethodNotAllowed()
        .body("Delete operations are not supported for geojson import lift data")
}

pub async fn delete_lifts_by_resort(
    _db: web::Data<MySqlPool>,
    _resort_id: web::Path<String>,
) -> impl Responder {
    HttpResponse::MethodNotAllowed()
        .body("Delete operations are not supported for geojson import lift data")
}
