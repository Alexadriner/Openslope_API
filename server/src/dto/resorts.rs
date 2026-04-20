//! Resort DTOs for API responses

use serde::Serialize;
use serde_json::Value;

use crate::models::db::Place;

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
    pub geometry: Option<Value>,
    pub places: Vec<Place>,
}

#[derive(Clone, Serialize)]
pub struct SlopeSummary {
    pub id: String,
    pub name: Option<String>,
    pub difficulty: Option<String>,
    pub geometry: Option<Value>,
    pub places: Vec<Place>,
}

#[derive(Serialize)]
pub struct ResortResponse {
    pub id: String,
    pub name: String,
    pub r#type: Option<String>,
    pub status: Option<String>,
    pub activities: Option<Value>,
    pub geometry: Value, // GeoJSON
    pub websites: Option<Value>,
    pub sources: Option<Value>,
    pub places: Vec<Place>,
    pub lifts: Vec<LiftSummary>,
    pub slopes: Vec<SlopeSummary>,
}
