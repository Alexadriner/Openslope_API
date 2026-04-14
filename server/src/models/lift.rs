//! Lift API response models for OpenSlope.
//!
//! This module defines the lift data structure used by the API.

use serde::Serialize;
use serde_json::Value;

/// Individual lift information for API responses
///
/// Represents a single lift within a ski resort, including its type and current status.
/// This information is useful for skiers planning their day on the mountain.
#[derive(Serialize)]
pub struct LiftResponse {
    /// Unique identifier for the lift
    pub id: String,
    /// Reference identifier from the GeoJSON import
    pub ref_: Option<String>,
    /// Name of the lift
    pub name: Option<String>,
    /// Lift classification from the GeoJSON import
    pub r#type: Option<String>,
    /// Access restrictions or access type
    pub access: Option<String>,
    /// Indicates whether this lift has a bubble cover
    pub bubble: Option<bool>,
    /// Indicates whether the lift is one-way only
    pub oneway: Option<bool>,
    /// Current operational status of the lift
    pub status: Option<String>,
    /// Indicates whether the lift passes through a tunnel
    pub tunnel: Option<bool>,
    /// Heating type or system for the lift
    pub heating: Option<String>,
    /// Rider capacity per cabin or vehicle
    pub capacity: Option<i32>,
    /// Estimated ride duration in minutes
    pub duration: Option<f64>,
    /// Specific lift type label (e.g. chairlift, gondola)
    pub lift_type: Option<String>,
    /// Occupancy metadata
    pub occupancy: Option<Value>,
    /// Whether the lift is detachable
    pub detachable: Option<bool>,
    /// Additional reference identifier
    pub ref_frcairn: Option<String>,
    /// Wikidata identifier for the lift
    pub wikidata_id: Option<String>,
    /// Human-readable description of the lift
    pub description: Option<String>,
    /// Viewport hint for map rendering
    pub viewport_hint: Option<Value>,
    /// Place-level metadata for the lift
    pub places: Option<Value>,
    /// Source metadata for the lift
    pub sources: Option<Value>,
    /// Associated ski areas
    pub ski_areas: Option<Value>,
    /// Station metadata for the lift
    pub stations: Option<Value>,
    /// Website metadata for the lift
    pub websites: Option<Value>,
    /// Geometry type for the lift path
    pub geometry_type: Option<String>,
    /// GeoJSON geometry payload for the lift
    pub geometry: Option<Value>,
    /// Additional GeoJSON properties for the lift
    pub properties: Option<Value>,
    /// Database creation timestamp
    pub created_at: Option<String>,
    /// Database update timestamp
    pub updated_at: Option<String>,
}
