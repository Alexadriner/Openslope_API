//! OpenSlope API Slopes Routes
//!
//! This module provides CRUD operations for slopes and the ability to query slopes
//! by associated resort IDs.

use actix_web::{HttpResponse, Responder, web};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    db, dto,
    error::AppError,
    models::db::Place,
    utils::{parse_feature_id, parse_geometry_to_linestring_wkt, parse_places, parse_related_ids},
};

#[derive(Deserialize)]
pub struct GeoJsonFeature {
    #[serde(rename = "type")]
    pub feature_type: String,
    pub geometry: Value,
    pub properties: Value,
}

#[derive(Serialize)]
pub struct SlopeResponse {
    pub id: String,
    pub name: Option<String>,
    pub difficulty: Option<String>,
    pub status: Option<String>,
    pub grooming: Option<String>,
    pub geometry: Value, // GeoJSON
    pub websites: Option<Value>,
    pub sources: Option<Value>,
    pub places: Vec<Place>,
    pub resorts: Vec<dto::resorts::ResortSummary>,
    pub elevation_profile: Option<dto::slopes::ElevationProfileResponse>,
}

struct SlopePayload {
    id: String,
    name: Option<String>,
    difficulty: Option<String>,
    status: Option<String>,
    grooming: Option<String>,
    geometry: String,
    websites: Option<Value>,
    sources: Option<Value>,
    resort_ids: Vec<String>,
    places: Vec<Place>,
}

impl SlopePayload {
    fn from_feature(feature: &GeoJsonFeature) -> Result<Self, AppError> {
        let id = parse_feature_id(&serde_json::json!({
            "id": feature.properties.get("id").cloned().unwrap_or(Value::Null),
        }))
        .or_else(|| parse_feature_id(&serde_json::json!({ "id": feature.feature_type.clone() })))
        .ok_or_else(|| AppError::BadRequest("Missing feature id".into()))?;

        let geometry = parse_geometry_to_linestring_wkt(&feature.geometry)
            .ok_or_else(|| AppError::BadRequest("Invalid GeoJSON geometry".into()))?;

        Ok(SlopePayload {
            id,
            name: extract_optional_string(feature.properties.get("name")),
            difficulty: extract_optional_string(feature.properties.get("difficulty")),
            status: extract_optional_string(feature.properties.get("status")),
            grooming: extract_optional_string(feature.properties.get("grooming")),
            geometry,
            websites: feature.properties.get("websites").cloned(),
            sources: feature.properties.get("sources").cloned(),
            resort_ids: parse_related_ids(&feature.properties),
            places: parse_places(&feature.properties),
        })
    }
}

fn extract_optional_string(value: Option<&Value>) -> Option<String> {
    value.and_then(|value| match value {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    })
}

fn map_slope_row(slope: db::slopes::SlopeWithResorts) -> SlopeResponse {
    let resorts = slope
        .resorts
        .into_iter()
        .map(|r| dto::resorts::ResortSummary {
            id: r.id,
            name: r.name,
            places: r.places,
        })
        .collect();

    SlopeResponse {
        id: slope.slope.id,
        name: slope.slope.name,
        difficulty: slope.slope.difficulty,
        status: slope.slope.status,
        grooming: slope.slope.grooming,
        geometry: slope.slope.geometry,
        websites: slope.slope.websites,
        sources: slope.slope.sources,
        places: slope.slope.places,
        resorts,
        elevation_profile: None, // TODO: Load elevation profile
    }
}

pub async fn get_slopes(db: web::Data<sqlx::MySqlPool>) -> Result<impl Responder, AppError> {
    let slopes = db::slopes::get_all(db.get_ref()).await?;
    Ok(HttpResponse::Ok().json(slopes.into_iter().map(map_slope_row).collect::<Vec<_>>()))
}

pub async fn get_slope_count(db: web::Data<sqlx::MySqlPool>) -> Result<impl Responder, AppError> {
    let count = db::slopes::count(db.get_ref()).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({ "count": count })))
}

pub async fn get_slope(
    db: web::Data<sqlx::MySqlPool>,
    id: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let slope = db::slopes::get_by_id(db.get_ref(), &id).await?;
    match slope {
        Some(slope) => Ok(HttpResponse::Ok().json(map_slope_row(slope))),
        None => Err(AppError::NotFound("Slope not found".into())),
    }
}

pub async fn get_slopes_by_resort(
    db: web::Data<sqlx::MySqlPool>,
    resort_id: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let slopes = db::slopes::get_by_resort(db.get_ref(), &resort_id).await?;
    Ok(HttpResponse::Ok().json(slopes.into_iter().map(map_slope_row).collect::<Vec<_>>()))
}

pub async fn create_slope(
    db: web::Data<sqlx::MySqlPool>,
    feature: web::Json<GeoJsonFeature>,
) -> Result<HttpResponse, AppError> {
    let payload = SlopePayload::from_feature(&feature)?;
    if db::slopes::get_by_id(db.get_ref(), &payload.id)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("Slope already exists".into()));
    }

    db::slopes::insert(
        db.get_ref(),
        &payload.id,
        payload.name.as_deref(),
        payload.difficulty.as_deref(),
        payload.status.as_deref(),
        payload.grooming.as_deref(),
        &payload.geometry,
        payload.websites.as_ref(),
        payload.sources.as_ref(),
        &payload.resort_ids,
        &payload.places,
    )
    .await?;

    Ok(HttpResponse::Created().json(serde_json::json!({"id": payload.id})))
}

pub async fn update_slope(
    db: web::Data<sqlx::MySqlPool>,
    id: web::Path<String>,
    feature: web::Json<GeoJsonFeature>,
) -> Result<impl Responder, AppError> {
    let payload = SlopePayload::from_feature(&feature)?;
    let rows_affected = db::slopes::update(
        db.get_ref(),
        &id,
        payload.name.as_deref(),
        payload.difficulty.as_deref(),
        payload.status.as_deref(),
        payload.grooming.as_deref(),
        &payload.geometry,
        payload.websites.as_ref(),
        payload.sources.as_ref(),
        &payload.resort_ids,
        &payload.places,
    )
    .await?;

    if rows_affected == 0 {
        return Err(AppError::NotFound("Slope not found".into()));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({"id": id.into_inner()})))
}

pub async fn delete_slope(
    db: web::Data<sqlx::MySqlPool>,
    id: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let deleted = db::slopes::delete(db.get_ref(), &id).await?;
    if deleted == 0 {
        Err(AppError::NotFound("Slope not found".into()))
    } else {
        Ok(HttpResponse::NoContent().finish())
    }
}

pub async fn delete_slopes_by_resort(
    db: web::Data<sqlx::MySqlPool>,
    resort_id: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let deleted = db::slopes::delete_by_resort(db.get_ref(), &resort_id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": deleted})))
}
