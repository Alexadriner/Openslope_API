//! OpenSlope API Resorts Routes
//!
//! This module handles all HTTP requests related to ski resort management. It
//! exposes CRUD operations for resorts and includes related lift and slope
//! summaries when request data includes the full resort record.

use actix_web::{HttpResponse, Responder, web};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::db::status::ResortStatusSnapshotRow;
use crate::{
    db,
    error::AppError,
    models::db::Place,
    utils::{parse_feature_id, parse_geometry_to_wkt, parse_places},
};

#[derive(Deserialize)]
pub struct GeoJsonFeature {
    #[serde(rename = "type")]
    pub feature_type: String,
    pub geometry: Value,
    pub properties: Value,
}

#[derive(Serialize)]
pub struct ResortSummary {
    pub id: String,
    pub name: String,
    pub places: Vec<Place>,
}

#[derive(Clone, Serialize)]
pub struct LiftSummary {
    pub id: String,
    pub name: Option<String>,
    pub lift_type: Option<String>,
    pub status: Option<String>,
    pub capacity: Option<i32>,
    pub duration: Option<i32>,
    pub geometry: Option<Value>,
    pub places: Vec<Place>,
}

#[derive(Clone, Serialize)]
pub struct SlopeSummary {
    pub id: String,
    pub name: Option<String>,
    pub difficulty: Option<String>,
    pub status: Option<String>,
    pub grooming: Option<String>,
    pub geometry: Option<Value>,
    pub places: Vec<Place>,
}

#[derive(Clone, Serialize)]
pub struct ResortSnapshotSummary {
    pub snapshot_time: Option<String>,
    pub lifts_open_count: Option<i32>,
    pub lifts_total_count: Option<i32>,
    pub slopes_open_count: Option<i32>,
    pub slopes_total_count: Option<i32>,
    pub snow_depth_valley_cm: Option<i16>,
    pub snow_depth_mountain_cm: Option<i16>,
    pub new_snow_24h_cm: Option<i16>,
    pub temperature_valley_c: Option<f64>,
    pub temperature_mountain_c: Option<f64>,
}

#[derive(Clone, Serialize)]
pub struct ResortStats {
    pub lift_count: usize,
    pub slope_count: usize,
    pub open_lift_count: Option<i32>,
    pub open_slope_count: Option<i32>,
}

#[derive(Serialize)]
pub struct ResortResponse {
    pub id: String,
    pub name: String,
    pub r#type: Option<String>,
    pub status: Option<String>,
    pub activities: Option<Value>,
    pub geometry: Value,
    pub websites: Option<Value>,
    pub sources: Option<Value>,
    pub places: Vec<Place>,
    pub stats: ResortStats,
    pub latest_snapshot: Option<ResortSnapshotSummary>,
    pub lifts: Vec<LiftSummary>,
    pub slopes: Vec<SlopeSummary>,
}

struct ResortPayload {
    pub id: String,
    pub name: String,
    pub r#type: Option<String>,
    pub status: Option<String>,
    pub activities: Option<Value>,
    pub geometry: String,
    pub websites: Option<Value>,
    pub sources: Option<Value>,
    pub places: Vec<Place>,
}

impl ResortPayload {
    fn from_feature(feature: &GeoJsonFeature) -> Result<Self, AppError> {
        let id = parse_feature_id(&serde_json::json!({
            "id": feature.properties.get("id").cloned().unwrap_or(Value::Null),
        }))
        .or_else(|| parse_feature_id(&serde_json::json!({ "id": feature.feature_type.clone() })))
        .ok_or_else(|| AppError::BadRequest("Missing feature id".into()))?;

        let name = extract_string(feature.properties.get("name"))
            .or_else(|| extract_string(feature.properties.get("title")))
            .ok_or_else(|| AppError::BadRequest("Missing resort name".into()))?;

        let geometry = parse_geometry_to_wkt(&feature.geometry)
            .ok_or_else(|| AppError::BadRequest("Invalid GeoJSON geometry".into()))?;

        Ok(ResortPayload {
            id,
            name,
            r#type: extract_string(feature.properties.get("type")),
            status: extract_string(feature.properties.get("status")),
            activities: feature.properties.get("activities").cloned(),
            geometry,
            websites: feature.properties.get("websites").cloned(),
            sources: feature.properties.get("sources").cloned(),
            places: parse_places(&feature.properties),
        })
    }
}

fn extract_string(value: Option<&Value>) -> Option<String> {
    value.and_then(|value| match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    })
}

fn lift_summaries_by_resort(
    lifts: Vec<crate::db::lifts::LiftWithResorts>,
) -> std::collections::HashMap<String, Vec<LiftSummary>> {
    let mut map: std::collections::HashMap<String, Vec<LiftSummary>> =
        std::collections::HashMap::new();
    for lift in lifts {
        let summary = LiftSummary {
            id: lift.lift.id.clone(),
            name: lift.lift.name.clone(),
            lift_type: lift.lift.r#type.clone(),
            status: lift.lift.status.clone(),
            capacity: lift.lift.capacity,
            duration: lift.lift.duration,
            geometry: Some(lift.lift.geometry.clone()),
            places: lift.lift.places.clone(),
        };
        for resort in &lift.resorts {
            map.entry(resort.id.clone())
                .or_default()
                .push(summary.clone());
        }
    }
    map
}

fn slope_summaries_by_resort(
    slopes: Vec<crate::db::slopes::SlopeWithResorts>,
) -> std::collections::HashMap<String, Vec<SlopeSummary>> {
    let mut map: std::collections::HashMap<String, Vec<SlopeSummary>> =
        std::collections::HashMap::new();
    for slope in slopes {
        let summary = SlopeSummary {
            id: slope.slope.id.clone(),
            name: slope.slope.name.clone(),
            difficulty: slope.slope.difficulty.clone(),
            status: slope.slope.status.clone(),
            grooming: slope.slope.grooming.clone(),
            geometry: Some(slope.slope.geometry.clone()),
            places: slope.slope.places.clone(),
        };
        for resort in &slope.resorts {
            map.entry(resort.id.clone())
                .or_default()
                .push(summary.clone());
        }
    }
    map
}

fn map_snapshot(snapshot: &ResortStatusSnapshotRow) -> ResortSnapshotSummary {
    ResortSnapshotSummary {
        snapshot_time: snapshot.snapshot_time.clone(),
        lifts_open_count: snapshot.lifts_open_count,
        lifts_total_count: snapshot.lifts_total_count,
        slopes_open_count: snapshot.slopes_open_count,
        slopes_total_count: snapshot.slopes_total_count,
        snow_depth_valley_cm: snapshot.snow_depth_valley_cm,
        snow_depth_mountain_cm: snapshot.snow_depth_mountain_cm,
        new_snow_24h_cm: snapshot.new_snow_24h_cm,
        temperature_valley_c: snapshot.temperature_valley_c,
        temperature_mountain_c: snapshot.temperature_mountain_c,
    }
}

fn build_resort_stats(
    lifts: &[LiftSummary],
    slopes: &[SlopeSummary],
    snapshot: Option<&ResortStatusSnapshotRow>,
) -> ResortStats {
    ResortStats {
        lift_count: lifts.len(),
        slope_count: slopes.len(),
        open_lift_count: snapshot
            .and_then(|entry| entry.lifts_open_count)
            .or_else(|| {
                Some(
                    lifts
                        .iter()
                        .filter(|lift| matches!(lift.status.as_deref(), Some("open")))
                        .count() as i32,
                )
            }),
        open_slope_count: snapshot
            .and_then(|entry| entry.slopes_open_count)
            .or_else(|| {
                Some(
                    slopes
                        .iter()
                        .filter(|slope| matches!(slope.status.as_deref(), Some("open")))
                        .count() as i32,
                )
            }),
    }
}

pub async fn get_resorts(
    db: web::Data<sqlx::MySqlPool>,
    query: web::Query<std::collections::HashMap<String, String>>,
) -> Result<impl Responder, AppError> {
    if query
        .get("summary")
        .map(|value| {
            matches!(
                value.to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
    {
        let resorts = db::resorts::get_all(db.get_ref()).await?;
        let summaries: Vec<ResortSummary> = resorts
            .into_iter()
            .map(|resort| ResortSummary {
                id: resort.id,
                name: resort.name,
                places: resort.places,
            })
            .collect();
        return Ok(HttpResponse::Ok().json(summaries));
    }

    let resorts = db::resorts::get_all(db.get_ref()).await?;
    let lifts = db::lifts::get_all(db.get_ref()).await?;
    let slopes = db::slopes::get_all(db.get_ref()).await?;
    let resort_ids = resorts
        .iter()
        .map(|resort| resort.id.clone())
        .collect::<Vec<_>>();
    let latest_snapshots = db::status::get_latest_for_resorts(db.get_ref(), &resort_ids).await?;

    let lifts_by_resort = lift_summaries_by_resort(lifts);
    let slopes_by_resort = slope_summaries_by_resort(slopes);

    let response: Vec<ResortResponse> = resorts
        .into_iter()
        .map(|resort| {
            let lifts = lifts_by_resort.get(&resort.id).cloned().unwrap_or_default();
            let slopes = slopes_by_resort
                .get(&resort.id)
                .cloned()
                .unwrap_or_default();
            let latest_snapshot = latest_snapshots.get(&resort.id);

            ResortResponse {
                id: resort.id.clone(),
                name: resort.name,
                r#type: resort.r#type,
                status: resort.status,
                activities: resort.activities,
                geometry: resort.geometry,
                websites: resort.websites,
                sources: resort.sources,
                places: resort.places,
                stats: build_resort_stats(&lifts, &slopes, latest_snapshot),
                latest_snapshot: latest_snapshot.map(map_snapshot),
                lifts,
                slopes,
            }
        })
        .collect();

    Ok(HttpResponse::Ok().json(response))
}

pub async fn get_resort(
    db: web::Data<sqlx::MySqlPool>,
    id: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let resort = db::resorts::get_by_id(db.get_ref(), &id)
        .await?
        .ok_or_else(|| AppError::NotFound("Resort not found".into()))?;

    let lifts = db::lifts::get_by_resort(db.get_ref(), &resort.id)
        .await?
        .into_iter()
        .map(|lift| LiftSummary {
            id: lift.lift.id,
            name: lift.lift.name,
            lift_type: lift.lift.r#type,
            status: lift.lift.status,
            capacity: lift.lift.capacity,
            duration: lift.lift.duration,
            geometry: Some(lift.lift.geometry),
            places: lift.lift.places,
        })
        .collect::<Vec<_>>();
    let slopes = db::slopes::get_by_resort(db.get_ref(), &resort.id)
        .await?
        .into_iter()
        .map(|slope| SlopeSummary {
            id: slope.slope.id,
            name: slope.slope.name,
            difficulty: slope.slope.difficulty,
            status: slope.slope.status,
            grooming: slope.slope.grooming,
            geometry: Some(slope.slope.geometry),
            places: slope.slope.places,
        })
        .collect::<Vec<_>>();
    let latest_snapshot = db::status::get_latest_for_resorts(db.get_ref(), &[resort.id.clone()])
        .await?
        .remove(&resort.id);

    let response = ResortResponse {
        id: resort.id.clone(),
        name: resort.name,
        r#type: resort.r#type,
        status: resort.status,
        activities: resort.activities,
        geometry: resort.geometry,
        websites: resort.websites,
        sources: resort.sources,
        places: resort.places,
        stats: build_resort_stats(&lifts, &slopes, latest_snapshot.as_ref()),
        latest_snapshot: latest_snapshot.as_ref().map(map_snapshot),
        lifts,
        slopes,
    };

    Ok(HttpResponse::Ok().json(response))
}

pub async fn create_resort(
    db: web::Data<sqlx::MySqlPool>,
    feature: web::Json<GeoJsonFeature>,
) -> Result<HttpResponse, AppError> {
    let payload = ResortPayload::from_feature(&feature)?;
    if db::resorts::get_by_id(db.get_ref(), &payload.id)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("Resort already exists".into()));
    }

    db::resorts::insert(
        db.get_ref(),
        &payload.id,
        &payload.name,
        payload.r#type.as_deref(),
        payload.status.as_deref(),
        payload.activities.as_ref(),
        &payload.geometry,
        payload.websites.as_ref(),
        payload.sources.as_ref(),
        &payload.places,
    )
    .await?;

    Ok(HttpResponse::Created().json(serde_json::json!({"id": payload.id})))
}

pub async fn update_resort(
    db: web::Data<sqlx::MySqlPool>,
    id: web::Path<String>,
    feature: web::Json<GeoJsonFeature>,
) -> Result<impl Responder, AppError> {
    let payload = ResortPayload::from_feature(&feature)?;
    let rows = db::resorts::update(
        db.get_ref(),
        &id,
        &payload.name,
        payload.r#type.as_deref(),
        payload.status.as_deref(),
        payload.activities.as_ref(),
        &payload.geometry,
        payload.websites.as_ref(),
        payload.sources.as_ref(),
        &payload.places,
    )
    .await?;

    if rows == 0 {
        return Err(AppError::NotFound("Resort not found".into()));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({"id": id.into_inner()})))
}

pub async fn delete_resort(
    db: web::Data<sqlx::MySqlPool>,
    id: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let deleted = db::resorts::delete(db.get_ref(), &id).await?;
    if deleted == 0 {
        Err(AppError::NotFound("Resort not found".into()))
    } else {
        Ok(HttpResponse::NoContent().finish())
    }
}
