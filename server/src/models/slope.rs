//! Slope API response models for OpenSlope.
//!
//! This module defines the slope data structure used by the API.

use serde::Serialize;
use serde_json::Value;

/// Individual slope information for API responses.
#[derive(Serialize)]
pub struct SlopeResponse {
    pub id: String,
    pub name: Option<String>,
    pub difficulty: Option<String>,
    pub status: Option<String>,
    pub grooming: Option<String>,
    pub geometry: Value,
    pub websites: Option<Value>,
    pub sources: Option<Value>,
    pub places: Vec<super::db::Place>,
    pub resorts: Vec<super::resort::ResortSummary>,
    pub elevation_profile: Option<ElevationProfileResponse>,
}

#[derive(Serialize)]
pub struct ElevationProfileResponse {
    pub id: i32,
    pub heights: Value,
    pub resolution: f64,
    pub target_resolution: f64,
}
