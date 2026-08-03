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
    pub geometry: Value, // GeoJSON
    pub websites: Option<Value>,
    pub sources: Option<Value>,
    pub places: Vec<Place>,
    pub stats: ResortStats,
    pub latest_snapshot: Option<ResortSnapshotSummary>,
    pub lifts: Vec<LiftSummary>,
    pub slopes: Vec<SlopeSummary>,
}
