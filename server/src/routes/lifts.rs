//! OpenSlope API Lifts Routes
//!
//! This module provides CRUD operations for lifts and the ability to query lifts
//! by associated resort IDs.

use actix_web::{HttpResponse, Responder, web};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    db, dto,
    error::AppError,
    models::db::Place,
    utils::{parse_feature_id, parse_geometry_to_wkt, parse_places, parse_related_ids},
};

#[derive(Deserialize)]
pub struct GeoJsonFeature {
    #[serde(rename = "type")]
    pub feature_type: String,
    pub geometry: Value,
    pub properties: Value,
}

#[derive(Serialize)]
pub struct LiftResponse {
    pub id: String,
    pub name: Option<String>,
    pub lift_type: Option<String>,
    pub status: Option<String>,
    pub capacity: Option<i32>,
    pub duration: Option<i32>,
    pub geometry: Value, // GeoJSON
    pub websites: Option<Value>,
    pub sources: Option<Value>,
    pub places: Vec<Place>,
    pub resorts: Vec<dto::resorts::ResortSummary>,
    pub elevation_profile: Option<dto::lifts::ElevationProfileResponse>,
}

struct LiftPayload {
    id: String,
    name: Option<String>,
    lift_type: Option<String>,
    status: Option<String>,
    capacity: Option<i32>,
    duration: Option<i32>,
    geometry: String,
    websites: Option<Value>,
    sources: Option<Value>,
    resort_ids: Vec<String>,
    places: Vec<Place>,
}

impl LiftPayload {
    fn from_feature(feature: &GeoJsonFeature) -> Result<Self, AppError> {
        let id = parse_feature_id(&serde_json::json!({
            "id": feature.properties.get("id").cloned().unwrap_or(Value::Null),
        }))
        .or_else(|| parse_feature_id(&serde_json::json!({ "id": feature.feature_type.clone() })))
        .ok_or_else(|| AppError::BadRequest("Missing feature id".into()))?;

        let geometry = parse_geometry_to_wkt(&feature.geometry)
            .ok_or_else(|| AppError::BadRequest("Invalid GeoJSON geometry".into()))?;

        Ok(LiftPayload {
            id,
            name: extract_optional_string(feature.properties.get("name")),
            lift_type: extract_optional_string(feature.properties.get("liftType"))
                .or_else(|| extract_optional_string(feature.properties.get("type"))),
            status: extract_optional_string(feature.properties.get("status")),
            capacity: extract_optional_i32(feature.properties.get("capacity")),
            duration: extract_optional_i32(feature.properties.get("duration")),
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

fn extract_optional_i32(value: Option<&Value>) -> Option<i32> {
    match value {
        Some(Value::Number(num)) => num.as_i64().and_then(|value| Some(value as i32)),
        Some(Value::String(text)) => text.parse::<i32>().ok(),
        _ => None,
    }
}

fn map_lift_row(lift: db::lifts::LiftWithResorts) -> LiftResponse {
    let resorts = lift
        .resorts
        .into_iter()
        .map(|r| dto::resorts::ResortSummary {
            id: r.id,
            name: r.name,
            places: r.places,
        })
        .collect();

    LiftResponse {
        id: lift.lift.id,
        name: lift.lift.name,
        lift_type: lift.lift.r#type,
        status: lift.lift.status,
        capacity: lift.lift.capacity,
        duration: lift.lift.duration,
        geometry: lift.lift.geometry,
        websites: lift.lift.websites,
        sources: lift.lift.sources,
        places: lift.lift.places,
        resorts,
        elevation_profile: None, // TODO: Load elevation profile
    }
}

pub async fn get_lifts(db: web::Data<sqlx::MySqlPool>) -> Result<impl Responder, AppError> {
    let lifts = db::lifts::get_all(db.get_ref()).await?;
    Ok(HttpResponse::Ok().json(lifts.into_iter().map(map_lift_row).collect::<Vec<_>>()))
}

pub async fn get_lift(
    db: web::Data<sqlx::MySqlPool>,
    id: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let lift = db::lifts::get_by_id(db.get_ref(), &id).await?;
    match lift {
        Some(lift) => Ok(HttpResponse::Ok().json(map_lift_row(lift))),
        None => Err(AppError::NotFound("Lift not found".into())),
    }
}

pub async fn get_lifts_by_resort(
    db: web::Data<sqlx::MySqlPool>,
    resort_id: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let lifts = db::lifts::get_by_resort(db.get_ref(), &resort_id).await?;
    Ok(HttpResponse::Ok().json(lifts.into_iter().map(map_lift_row).collect::<Vec<_>>()))
}

pub async fn create_lift(
    db: web::Data<sqlx::MySqlPool>,
    feature: web::Json<GeoJsonFeature>,
) -> Result<HttpResponse, AppError> {
    let payload = LiftPayload::from_feature(&feature)?;
    if db::lifts::get_by_id(db.get_ref(), &payload.id)
        .await?
        .is_some()
    {
        return Err(AppError::Conflict("Lift already exists".into()));
    }

    db::lifts::insert(
        db.get_ref(),
        &payload.id,
        payload.name.as_deref(),
        payload.lift_type.as_deref(),
        payload.status.as_deref(),
        payload.capacity,
        payload.duration,
        &payload.geometry,
        payload.websites.as_ref(),
        payload.sources.as_ref(),
        &payload.resort_ids,
        &payload.places,
    )
    .await?;

    Ok(HttpResponse::Created().json(serde_json::json!({"id": payload.id})))
}

pub async fn update_lift(
    db: web::Data<sqlx::MySqlPool>,
    id: web::Path<String>,
    feature: web::Json<GeoJsonFeature>,
) -> Result<impl Responder, AppError> {
    let payload = LiftPayload::from_feature(&feature)?;
    let rows_affected = db::lifts::update(
        db.get_ref(),
        &id,
        payload.name.as_deref(),
        payload.lift_type.as_deref(),
        payload.status.as_deref(),
        payload.capacity,
        payload.duration,
        &payload.geometry,
        payload.websites.as_ref(),
        payload.sources.as_ref(),
        &payload.resort_ids,
        &payload.places,
    )
    .await?;

    if rows_affected == 0 {
        return Err(AppError::NotFound("Lift not found".into()));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({"id": id.into_inner()})))
}

pub async fn delete_lift(
    db: web::Data<sqlx::MySqlPool>,
    id: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let deleted = db::lifts::delete(db.get_ref(), &id).await?;
    if deleted == 0 {
        Err(AppError::NotFound("Lift not found".into()))
    } else {
        Ok(HttpResponse::NoContent().finish())
    }
}

pub async fn delete_lifts_by_resort(
    db: web::Data<sqlx::MySqlPool>,
    resort_id: web::Path<String>,
) -> Result<impl Responder, AppError> {
    let deleted = db::lifts::delete_by_resort(db.get_ref(), &resort_id).await?;
    Ok(HttpResponse::Ok().json(serde_json::json!({"deleted": deleted})))
}
