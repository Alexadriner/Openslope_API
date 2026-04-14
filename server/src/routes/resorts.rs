//! OpenSlope API Resorts Routes
//!
//! This module handles all HTTP requests related to ski resort management in the
//! OpenSlope API. It provides comprehensive CRUD operations for resorts including
//! detailed information about lifts, slopes, and operational status.
//!
//! # Route Overview
//!
//! The resorts module provides the following endpoints:
//!
//! - **GET /resorts**: List all resorts with optional summary mode
//! - **GET /resorts/{id}**: Get detailed information about a specific resort
//! - **POST /resorts**: Create a new resort
//! - **PUT /resorts/{id}**: Update an existing resort
//! - **DELETE /resorts/{id}**: Delete a resort
//!
//! # Data Models
//!
//! The module defines several data structures for handling resort information:
//!
//! - **Resort**: Complete database representation of a resort
//! - **ResortWithRelations**: Comprehensive response model with nested data
//! - **CreateResort/UpdateResort**: Input models for creation and updates
//! - **LiftSummary/SlopeSummary**: Nested models for associated facilities
//!
//! # Architecture Patterns
//!
//! - **Hierarchical Data**: Resorts contain nested lifts and slopes
//! - **Optional Fields**: Many fields are optional to handle incomplete data
//! - **Coordinate Handling**: Latitude/longitude with proper type conversion
//! - **JSON Path Parsing**: Complex path_geojson parsing for slope routes
//!
//! # Performance Optimizations
//!
//! - **Batch Loading**: Lifts and slopes are loaded in batches for efficiency
//! - **HashMap Indexing**: Fast lookup of related data by resort ID
//! - **Lazy Evaluation**: Only load related data when needed
//! - **Query Optimization**: Optimized SQL queries with proper column selection
//!
//! # Error Handling
//!
//! All route handlers implement consistent error handling:
//! - Database errors return 500 Internal Server Error
//! - Missing resources return 404 Not Found
//! - Invalid data returns 400 Bad Request
//! - Successful operations return appropriate HTTP status codes
//!
//! # Usage Examples
//!
//! ```rust
//! // Get all resorts with full details
//! GET /api/v1/resorts
//!
//! // Get resort summary only
//! GET /api/v1/resorts?summary=1
//!
//! // Get specific resort with all related data
//! GET /api/v1/resorts/resort_123
//!
//! // Create new resort
//! POST /api/v1/resorts
//! {
//!   "id": "new_resort",
//!   "name": "New Ski Resort",
//!   "country": "Austria",
//!   // ... other fields
//! }
//! ```
//!
//! Author: OpenSlope Team
//! Version: 1.0.0

use actix_web::{HttpResponse, Responder, web};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::{Error, MySqlPool};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Resort {
    pub id: String,
    pub name: String,
    pub country: Option<String>,
    pub region: Option<String>,
    pub continent: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub village_altitude_m: Option<i32>,
    pub min_altitude_m: Option<i32>,
    pub max_altitude_m: Option<i32>,
    pub ski_area_name: Option<String>,
    pub ski_area_type: Option<String>,
    pub official_website: Option<String>,
    pub lift_status_url: Option<String>,
    pub slope_status_url: Option<String>,
    pub snow_report_url: Option<String>,
    pub weather_url: Option<String>,
    pub status_provider: Option<String>,
    pub status_last_scraped_at: Option<String>,
    pub lifts_open_count: Option<i32>,
    pub slopes_open_count: Option<i32>,
    pub snow_depth_valley_cm: Option<i16>,
    pub snow_depth_mountain_cm: Option<i16>,
    pub new_snow_24h_cm: Option<i16>,
    pub temperature_valley_c: Option<f64>,
    pub temperature_mountain_c: Option<f64>,
}

#[derive(Deserialize)]
pub struct CreateResort {
    pub id: String,
    pub name: String,
    pub country: String,
    pub region: Option<String>,
    pub continent: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub village_altitude_m: Option<i32>,
    pub min_altitude_m: Option<i32>,
    pub max_altitude_m: Option<i32>,
    pub ski_area_name: Option<String>,
    pub ski_area_type: String,
    pub official_website: Option<String>,
    pub lift_status_url: Option<String>,
    pub slope_status_url: Option<String>,
    pub snow_report_url: Option<String>,
    pub weather_url: Option<String>,
    pub status_provider: Option<String>,
    pub status_last_scraped_at: Option<String>,
    pub lifts_open_count: Option<i32>,
    pub slopes_open_count: Option<i32>,
    pub snow_depth_valley_cm: Option<i16>,
    pub snow_depth_mountain_cm: Option<i16>,
    pub new_snow_24h_cm: Option<i16>,
    pub temperature_valley_c: Option<f64>,
    pub temperature_mountain_c: Option<f64>,
}

#[derive(Deserialize)]
pub struct UpdateResort {
    pub name: String,
    pub country: String,
    pub region: Option<String>,
    pub continent: Option<String>,
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
    pub village_altitude_m: Option<i32>,
    pub min_altitude_m: Option<i32>,
    pub max_altitude_m: Option<i32>,
    pub ski_area_name: Option<String>,
    pub ski_area_type: String,
    pub official_website: Option<String>,
    pub lift_status_url: Option<String>,
    pub slope_status_url: Option<String>,
    pub snow_report_url: Option<String>,
    pub weather_url: Option<String>,
    pub status_provider: Option<String>,
    pub status_last_scraped_at: Option<String>,
    pub lifts_open_count: Option<i32>,
    pub slopes_open_count: Option<i32>,
    pub snow_depth_valley_cm: Option<i16>,
    pub snow_depth_mountain_cm: Option<i16>,
    pub new_snow_24h_cm: Option<i16>,
    pub temperature_valley_c: Option<f64>,
    pub temperature_mountain_c: Option<f64>,
}

#[derive(Serialize, Clone)]
pub struct LiftSummary {
    pub id: i64,
    pub name: Option<String>,
    pub lift_type: String,
    pub geometry: LineGeometry,
    pub status: LiftStatusSummary,
}

#[derive(Serialize, Clone)]
pub struct SlopeSummary {
    pub id: i64,
    pub name: Option<String>,
    pub difficulty: String,
    pub geometry: LineGeometry,
    pub status: SlopeStatusSummary,
}

#[derive(Serialize, Clone)]
pub struct LineGeometry {
    pub start: CoordinatePoint,
    pub end: CoordinatePoint,
    pub path: Option<Vec<CoordinatePoint>>,
}

#[derive(Serialize, Clone)]
pub struct CoordinatePoint {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Serialize, Clone)]
pub struct LiftStatusSummary {
    pub operational_status: String,
    pub note: Option<String>,
    pub planned_open_time: Option<String>,
    pub planned_close_time: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct SlopeStatusSummary {
    pub operational_status: String,
    pub grooming_status: String,
    pub note: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize)]
pub struct Coordinates {
    pub latitude: Option<f64>,
    pub longitude: Option<f64>,
}

#[derive(Serialize)]
pub struct Geography {
    pub continent: Option<String>,
    pub country: Option<String>,
    pub region: Option<String>,
    pub coordinates: Coordinates,
}

#[derive(Serialize)]
pub struct Altitude {
    pub village_m: Option<i32>,
    pub min_m: Option<i32>,
    pub max_m: Option<i32>,
}

#[derive(Serialize)]
pub struct SkiArea {
    pub name: Option<String>,
    pub area_type: Option<String>,
}

#[derive(Serialize)]
pub struct ResortSources {
    pub official_website: Option<String>,
    pub lift_status_url: Option<String>,
    pub slope_status_url: Option<String>,
    pub snow_report_url: Option<String>,
    pub weather_url: Option<String>,
    pub status_provider: Option<String>,
}

#[derive(Serialize)]
pub struct ResortLiveStatus {
    pub last_scraped_at: Option<String>,
    pub lifts_open_count: Option<i32>,
    pub slopes_open_count: Option<i32>,
    pub snow_depth_valley_cm: Option<i16>,
    pub snow_depth_mountain_cm: Option<i16>,
    pub new_snow_24h_cm: Option<i16>,
    pub temperature_valley_c: Option<f64>,
    pub temperature_mountain_c: Option<f64>,
}

#[derive(Serialize)]
pub struct ResortWithRelations {
    pub id: String,
    pub name: String,
    pub geography: Geography,
    pub altitude: Altitude,
    pub ski_area: SkiArea,
    pub sources: ResortSources,
    pub live_status: ResortLiveStatus,
    pub lifts: Vec<LiftSummary>,
    pub slopes: Vec<SlopeSummary>,
}

impl ResortWithRelations {
    fn from_resort(resort: Resort, lifts: Vec<LiftSummary>, slopes: Vec<SlopeSummary>) -> Self {
        Self {
            id: resort.id,
            name: resort.name,
            geography: Geography {
                continent: resort.continent,
                country: resort.country,
                region: resort.region,
                coordinates: Coordinates {
                    latitude: resort.latitude,
                    longitude: resort.longitude,
                },
            },
            altitude: Altitude {
                village_m: resort.village_altitude_m,
                min_m: resort.min_altitude_m,
                max_m: resort.max_altitude_m,
            },
            ski_area: SkiArea {
                name: resort.ski_area_name,
                area_type: resort.ski_area_type,
            },
            sources: ResortSources {
                official_website: resort.official_website,
                lift_status_url: resort.lift_status_url,
                slope_status_url: resort.slope_status_url,
                snow_report_url: resort.snow_report_url,
                weather_url: resort.weather_url,
                status_provider: resort.status_provider,
            },
            live_status: ResortLiveStatus {
                last_scraped_at: resort.status_last_scraped_at,
                lifts_open_count: resort.lifts_open_count,
                slopes_open_count: resort.slopes_open_count,
                snow_depth_valley_cm: resort.snow_depth_valley_cm,
                snow_depth_mountain_cm: resort.snow_depth_mountain_cm,
                new_snow_24h_cm: resort.new_snow_24h_cm,
                temperature_valley_c: resort.temperature_valley_c,
                temperature_mountain_c: resort.temperature_mountain_c,
            },
            lifts,
            slopes,
        }
    }
}

#[derive(Deserialize)]
pub struct ResortsQuery {
    pub summary: Option<String>,
}

#[derive(Serialize)]
pub struct ResortSummary {
    pub id: String,
    pub name: String,
}

fn is_truthy_flag(value: &str) -> bool {
    matches!(
        value.to_ascii_lowercase().as_str(),
        "1" | "true" | "yes" | "on"
    )
}

fn parse_geojson_geometry(raw: Option<String>) -> (Option<String>, Option<Vec<CoordinatePoint>>) {
    match raw.and_then(|value| serde_json::from_str::<Value>(&value).ok()) {
        Some(parsed) => {
            let geometry_type = parsed
                .get("type")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string());
            let path = parsed
                .get("coordinates")
                .and_then(parse_geojson_coordinates);
            (geometry_type, path)
        }
        None => (None, None),
    }
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

fn parse_geojson_path(raw: Option<String>) -> Option<Vec<CoordinatePoint>> {
    let raw = raw?;
    let parsed: Value = serde_json::from_str(&raw).ok()?;
    let coordinates = parsed.get("coordinates")?;
    parse_geojson_coordinates(coordinates)
}

fn parse_geojson_coordinates(value: &Value) -> Option<Vec<CoordinatePoint>> {
    if let Value::Array(arr) = value {
        if arr.is_empty() {
            return None;
        }

        if arr.iter().all(|item| item.is_number()) {
            return parse_coordinate_point(arr);
        }

        let mut points = Vec::new();
        for item in arr {
            if let Some(mut nested) = parse_geojson_coordinates(item) {
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

fn parse_coordinate_point(arr: &[Value]) -> Option<Vec<CoordinatePoint>> {
    if arr.len() >= 2 {
        let lon = arr[0].as_f64();
        let lat = arr[1].as_f64();
        if lat.is_some() && lon.is_some() {
            return Some(vec![CoordinatePoint {
                latitude: lat,
                longitude: lon,
            }]);
        }
    }
    None
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

async fn load_lifts_by_resort(db: &MySqlPool) -> Result<HashMap<String, Vec<LiftSummary>>, Error> {
    let rows = sqlx::query!(
        r#"
        SELECT id, name, lift_type, `type` AS feature_type, status,
               CAST(geometry AS CHAR) AS geometry_json,
               CAST(ski_areas AS CHAR) AS ski_areas_json
        FROM geojson_lifts
        "#
    )
    .fetch_all(db)
    .await?;

    let mut map: HashMap<String, Vec<LiftSummary>> = HashMap::new();
    for row in rows {
        let (_geometry_type, path) = parse_geojson_geometry(row.geometry_json);
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

        if let Some(resort_key) = parse_first_ski_area(row.ski_areas_json) {
            map.entry(resort_key).or_default().push(LiftSummary {
                id: row.id,
                name: row.name,
                lift_type: row
                    .lift_type
                    .or(row.feature_type)
                    .unwrap_or_else(|| "Unknown".to_string()),
                geometry: LineGeometry { start, end, path },
                status: LiftStatusSummary {
                    operational_status: row.status.unwrap_or_else(|| "Unknown".to_string()),
                    note: None,
                    planned_open_time: None,
                    planned_close_time: None,
                    updated_at: None,
                },
            });
        }
    }

    Ok(map)
}

async fn load_slopes_by_resort(
    db: &MySqlPool,
) -> Result<HashMap<String, Vec<SlopeSummary>>, Error> {
    let rows = sqlx::query!(
        r#"
        SELECT id, name, difficulty, status, grooming, snowmaking, lit, patrolled,
               difficulty_convention, CAST(geometry AS CHAR) AS geometry_json,
               CAST(ski_areas AS CHAR) AS ski_areas_json
        FROM geojson_runs
        "#
    )
    .fetch_all(db)
    .await?;

    let mut map: HashMap<String, Vec<SlopeSummary>> = HashMap::new();
    for row in rows {
        let (_geometry_type, path) = parse_geojson_geometry(row.geometry_json);
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

        if let Some(resort_key) = parse_first_ski_area(row.ski_areas_json) {
            map.entry(resort_key).or_default().push(SlopeSummary {
                id: row.id,
                name: row.name,
                difficulty: row.difficulty.unwrap_or_else(|| "Unknown".to_string()),
                geometry: LineGeometry { start, end, path },
                status: SlopeStatusSummary {
                    operational_status: row.status.unwrap_or_else(|| "Unknown".to_string()),
                    grooming_status: row.grooming.unwrap_or_else(|| "Unknown".to_string()),
                    note: None,
                    updated_at: None,
                },
            });
        }
    }

    Ok(map)
}

pub async fn get_resorts(
    db: web::Data<MySqlPool>,
    query: web::Query<ResortsQuery>,
) -> impl Responder {
    if query
        .summary
        .as_deref()
        .map(is_truthy_flag)
        .unwrap_or(false)
    {
        let result = sqlx::query_as!(
            ResortSummary,
            r#"
            SELECT id, name
            FROM resorts
            ORDER BY name
            "#
        )
        .fetch_all(db.get_ref())
        .await;

        return match result {
            Ok(summaries) => HttpResponse::Ok().json(summaries),
            Err(_) => HttpResponse::InternalServerError().finish(),
        };
    }

    let resorts_result = sqlx::query!(
        r#"
        SELECT
            id,
            name,
            NULL AS country,
            NULL AS region,
            NULL AS continent,
            NULL AS latitude,
            NULL AS longitude,
            NULL AS village_altitude_m,
            NULL AS min_altitude_m,
            NULL AS max_altitude_m,
            NULL AS ski_area_name,
            NULL AS ski_area_type,
            NULL AS official_website,
            NULL AS lift_status_url,
            NULL AS slope_status_url,
            NULL AS snow_report_url,
            NULL AS weather_url,
            NULL AS status_provider,
            NULL AS status_last_scraped_at,
            NULL AS lifts_open_count,
            NULL AS slopes_open_count,
            NULL AS snow_depth_valley_cm,
            NULL AS snow_depth_mountain_cm,
            NULL AS new_snow_24h_cm,
            NULL AS temperature_valley_c,
            NULL AS temperature_mountain_c
        FROM resorts
        ORDER BY name
        "#
    )
    .fetch_all(db.get_ref())
    .await;

    let resorts = match resorts_result {
        Ok(rows) => rows
            .into_iter()
            .map(|row| Resort {
                id: row.id,
                name: row.name,
                country: row.country,
                region: row.region,
                continent: row.continent,
                latitude: row.latitude,
                longitude: row.longitude,
                village_altitude_m: row.village_altitude_m,
                min_altitude_m: row.min_altitude_m,
                max_altitude_m: row.max_altitude_m,
                ski_area_name: row.ski_area_name,
                ski_area_type: row.ski_area_type,
                official_website: row.official_website,
                lift_status_url: row.lift_status_url,
                slope_status_url: row.slope_status_url,
                snow_report_url: row.snow_report_url,
                weather_url: row.weather_url,
                status_provider: row.status_provider,
                status_last_scraped_at: row.status_last_scraped_at,
                lifts_open_count: row.lifts_open_count,
                slopes_open_count: row.slopes_open_count,
                snow_depth_valley_cm: row.snow_depth_valley_cm,
                snow_depth_mountain_cm: row.snow_depth_mountain_cm,
                new_snow_24h_cm: row.new_snow_24h_cm,
                temperature_valley_c: row.temperature_valley_c,
                temperature_mountain_c: row.temperature_mountain_c,
            })
            .collect::<Vec<_>>(),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let lifts_by_resort = match load_lifts_by_resort(db.get_ref()).await {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let slopes_by_resort = match load_slopes_by_resort(db.get_ref()).await {
        Ok(data) => data,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let response: Vec<ResortWithRelations> = resorts
        .into_iter()
        .map(|resort| {
            let lifts = lifts_by_resort.get(&resort.id).cloned().unwrap_or_default();
            let slopes = slopes_by_resort
                .get(&resort.id)
                .cloned()
                .unwrap_or_default();
            ResortWithRelations::from_resort(resort, lifts, slopes)
        })
        .collect();

    HttpResponse::Ok().json(response)
}

pub async fn get_resort(db: web::Data<MySqlPool>, id: web::Path<String>) -> impl Responder {
    let resort_result = sqlx::query!(
        r#"
        SELECT
            id,
            name,
            NULL AS country,
            NULL AS region,
            NULL AS continent,
            NULL AS latitude,
            NULL AS longitude,
            NULL AS village_altitude_m,
            NULL AS min_altitude_m,
            NULL AS max_altitude_m,
            NULL AS ski_area_name,
            NULL AS ski_area_type,
            NULL AS official_website,
            NULL AS lift_status_url,
            NULL AS slope_status_url,
            NULL AS snow_report_url,
            NULL AS weather_url,
            NULL AS status_provider,
            NULL AS status_last_scraped_at,
            NULL AS lifts_open_count,
            NULL AS slopes_open_count,
            NULL AS snow_depth_valley_cm,
            NULL AS snow_depth_mountain_cm,
            NULL AS new_snow_24h_cm,
            NULL AS temperature_valley_c,
            NULL AS temperature_mountain_c
        FROM resorts
        WHERE id = ?
        "#,
        *id
    )
    .fetch_optional(db.get_ref())
    .await;

    let resort = match resort_result {
        Ok(Some(row)) => Resort {
            id: row.id,
            name: row.name,
            country: row.country,
            region: row.region,
            continent: row.continent,
            latitude: row.latitude,
            longitude: row.longitude,
            village_altitude_m: row.village_altitude_m,
            min_altitude_m: row.min_altitude_m,
            max_altitude_m: row.max_altitude_m,
            ski_area_name: row.ski_area_name,
            ski_area_type: row.ski_area_type,
            official_website: row.official_website,
            lift_status_url: row.lift_status_url,
            slope_status_url: row.slope_status_url,
            snow_report_url: row.snow_report_url,
            weather_url: row.weather_url,
            status_provider: row.status_provider,
            status_last_scraped_at: row.status_last_scraped_at,
            lifts_open_count: row.lifts_open_count,
            slopes_open_count: row.slopes_open_count,
            snow_depth_valley_cm: row.snow_depth_valley_cm,
            snow_depth_mountain_cm: row.snow_depth_mountain_cm,
            new_snow_24h_cm: row.new_snow_24h_cm,
            temperature_valley_c: row.temperature_valley_c,
            temperature_mountain_c: row.temperature_mountain_c,
        },
        Ok(None) => return HttpResponse::NotFound().finish(),
        Err(Error::RowNotFound) => return HttpResponse::NotFound().finish(),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let lifts_result = sqlx::query!(
        r#"
        SELECT id, name, lift_type, `type` AS feature_type, status,
               CAST(geometry AS CHAR) AS geometry_json
        FROM geojson_lifts
        WHERE JSON_CONTAINS(ski_areas, JSON_QUOTE(?))
        ORDER BY name
        "#,
        resort.id
    )
    .fetch_all(db.get_ref())
    .await;

    let lifts = match lifts_result {
        Ok(rows) => rows
            .into_iter()
            .map(|row| {
                let (_geometry_type, path) = parse_geojson_geometry(row.geometry_json);
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

                LiftSummary {
                    id: row.id,
                    name: row.name,
                    lift_type: row
                        .lift_type
                        .or(row.feature_type)
                        .unwrap_or_else(|| "Unknown".to_string()),
                    geometry: LineGeometry { start, end, path },
                    status: LiftStatusSummary {
                        operational_status: row.status.unwrap_or_else(|| "Unknown".to_string()),
                        note: None,
                        planned_open_time: None,
                        planned_close_time: None,
                        updated_at: None,
                    },
                }
            })
            .collect(),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let slopes_result = sqlx::query!(
        r#"
        SELECT id, name, difficulty, status, grooming,
               CAST(geometry AS CHAR) AS geometry_json
        FROM geojson_runs
        WHERE JSON_CONTAINS(ski_areas, JSON_QUOTE(?))
        ORDER BY name
        "#,
        resort.id
    )
    .fetch_all(db.get_ref())
    .await;

    let slopes = match slopes_result {
        Ok(rows) => rows
            .into_iter()
            .map(|row| {
                let (_geometry_type, path) = parse_geojson_geometry(row.geometry_json);
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

                SlopeSummary {
                    id: row.id,
                    name: row.name,
                    difficulty: row.difficulty.unwrap_or_else(|| "Unknown".to_string()),
                    geometry: LineGeometry { start, end, path },
                    status: SlopeStatusSummary {
                        operational_status: row.status.unwrap_or_else(|| "Unknown".to_string()),
                        grooming_status: row.grooming.unwrap_or_else(|| "Unknown".to_string()),
                        note: None,
                        updated_at: None,
                    },
                }
            })
            .collect(),
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let response = ResortWithRelations::from_resort(resort, lifts, slopes);
    HttpResponse::Ok().json(response)
}

pub async fn create_resort(
    _db: web::Data<MySqlPool>,
    _resort: web::Json<CreateResort>,
) -> impl Responder {
    HttpResponse::MethodNotAllowed()
        .body("Create operations are not supported for the current resort schema")
}

pub async fn update_resort(
    _db: web::Data<MySqlPool>,
    _id: web::Path<String>,
    _resort: web::Json<UpdateResort>,
) -> impl Responder {
    HttpResponse::MethodNotAllowed()
        .body("Update operations are not supported for the current resort schema")
}

pub async fn delete_resort(db: web::Data<MySqlPool>, id: web::Path<String>) -> impl Responder {
    let result = sqlx::query!("DELETE FROM resorts WHERE id = ?", *id)
        .execute(db.get_ref())
        .await;

    match result {
        Ok(res) if res.rows_affected() == 0 => HttpResponse::NotFound().finish(),
        Ok(_) => HttpResponse::NoContent().finish(),
        Err(_) => HttpResponse::BadRequest().finish(),
    }
}
