//! Slope DTOs for API responses

use serde::Serialize;
use serde_json::Value;

use crate::models::db::Place;

use super::resorts::ResortSummary;

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
    pub resorts: Vec<ResortSummary>,
    pub elevation_profile: Option<ElevationProfileResponse>,
}

#[derive(Serialize)]
pub struct ElevationProfileResponse {
    pub id: i32,
    pub heights: Value,
    pub resolution: f64,
    pub target_resolution: f64,
}
