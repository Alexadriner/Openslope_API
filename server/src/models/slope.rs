//! Slope API response models for OpenSlope.
//!
//! This module defines the slope data structure used by the API.

use serde::Serialize;
use serde_json::Value;

/// Individual slope information for API responses
///
/// Represents a single slope within a ski resort, including its difficulty level
/// and length. This information helps skiers choose appropriate runs for their
/// skill level and plan their day on the mountain.
#[derive(Serialize)]
pub struct SlopeResponse {
    /// Unique identifier for the slope
    pub id: String,
    /// Lit status for the slope
    pub lit: Option<bool>,
    /// Reference identifier from the GeoJSON import
    pub ref_: Option<String>,
    /// Name of the slope
    pub name: Option<String>,
    /// Slope classification from the GeoJSON import
    pub r#type: Option<String>,
    /// Secondary uses for the slope (e.g. ski touring)
    pub uses: Option<Value>,
    /// Indicates whether the run is gladed
    pub gladed: Option<bool>,
    /// Indicates whether the run is one-way only
    pub oneway: Option<bool>,
    /// Place-level metadata for the slope
    pub places: Option<Value>,
    /// Current operational status of the slope
    pub status: Option<String>,
    /// Indicates whether the slope passes through a tunnel
    pub tunnel: Option<bool>,
    /// Source metadata for the slope
    pub sources: Option<Value>,
    /// Grooming information for the slope
    pub grooming: Option<String>,
    /// Associated ski areas
    pub ski_areas: Option<Value>,
    /// Website metadata for the slope
    pub websites: Option<Value>,
    /// Whether the slope is patrolled
    pub patrolled: Option<bool>,
    /// Difficulty level (e.g., "Green", "Blue", "Red", "Black")
    pub difficulty: Option<String>,
    /// Whether snowmaking is available
    pub snowmaking: Option<bool>,
    /// Wikidata identifier for the slope
    pub wikidata_id: Option<String>,
    /// Human-readable description of the slope
    pub description: Option<String>,
    /// Whether snow farming is used on the slope
    pub snowfarming: Option<bool>,
    /// Viewport hint for map rendering
    pub viewport_hint: Option<Value>,
    /// Elevation profile metadata for the slope
    pub elevation_profile: Option<Value>,
    /// Difficulty convention for the slope
    pub difficulty_convention: Option<String>,
    /// Geometry type for the slope path
    pub geometry_type: Option<String>,
    /// GeoJSON geometry payload for the slope
    pub geometry: Option<Value>,
    /// Additional GeoJSON properties for the slope
    pub properties: Option<Value>,
    /// Database creation timestamp
    pub created_at: Option<String>,
    /// Database update timestamp
    pub updated_at: Option<String>,
}
