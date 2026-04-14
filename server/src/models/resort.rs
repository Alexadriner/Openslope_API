//! OpenSlope API Response Models
//!
//! This module defines the data structures used for API responses in the OpenSlope system.
//! These models are designed to provide comprehensive information about ski resorts,
//! including their geographical location, altitude data, ski area details, and associated
//! lifts and slopes.
//!
//! # Architecture Overview
//!
//! The response models follow a hierarchical structure where a `ResortResponse` contains
//! nested blocks of related information:
//!
//! - **LocationBlock**: Geographical and administrative information
//! - **AltitudeBlock**: Elevation data for the resort
//! - **SkiAreaBlock**: Information about the ski area itself
//! - **LiftResponse**: Individual lift information
//! - **SlopeResponse**: Individual slope information
//!
//! # Design Principles
//!
//! - **API-First Design**: Models are optimized for JSON serialization and API consumption
//! - **Comprehensive Data**: Each model provides all relevant information for its domain
//! - **Type Safety**: Strong typing with appropriate data types for each field
//! - **Optional Fields**: Non-critical statistics are optional to handle incomplete data
//!
//! # Usage Examples
//!
//! ```rust
//! use openslope_api::models::resort::{ResortResponse, LocationBlock, AltitudeBlock};
//!
//! // Create a resort response
//! let resort = ResortResponse {
//!     id: "resort_123".to_string(),
//!     name: "Example Ski Resort".to_string(),
//!     location: LocationBlock {
//!         country: "Austria".to_string(),
//!         region: "Tyrol".to_string(),
//!         continent: "Europe".to_string(),
//!         latitude: 47.2628,
//!         longitude: 11.3936,
//!     },
//!     altitude: AltitudeBlock {
//!         village_altitude_m: 1200,
//!         min_altitude_m: 1100,
//!         max_altitude_m: 2500,
//!     },
//!     // ... other fields
//!     lifts: vec![],
//!     slopes: vec![],
//! };
//! ```
//!
//! # Field Descriptions
//!
//! ## ResortResponse
//! - `id`: Unique identifier for the resort (required)
//! - `name`: Official name of the resort (required)
//! - `location`: Geographical and administrative location data
//! - `altitude`: Elevation information for the resort
//! - `ski_area`: Information about the ski area operations
//! - `lifts`: List of all lifts in the resort
//! - `slopes`: List of all slopes in the resort
//!
//! ## LocationBlock
//! - `country`: Country where the resort is located
//! - `region`: Administrative region within the country
//! - `continent`: Continent where the resort is located
//! - `latitude`: Geographic latitude coordinate (WGS84)
//! - `longitude`: Geographic longitude coordinate (WGS84)
//!
//! ## AltitudeBlock
//! - `village_altitude_m`: Altitude of the resort village/base area in meters
//! - `min_altitude_m`: Minimum skiable altitude in meters
//! - `max_altitude_m`: Maximum skiable altitude in meters
//!
//! ## SkiAreaBlock
//! - `name`: Name of the ski area
//! - `area_type`: Type of ski area (e.g., "Alpine", "Nordic", "Cross-country")
//! - `total_slope_km`: Total length of all slopes in kilometers (optional)
//! - `total_lifts`: Total number of lifts in the area (optional)
//! - `snowmaking_percent`: Percentage of slopes with snowmaking coverage (optional)
//! - `night_skiing`: Whether night skiing is available (optional)
//!
//! ## LiftResponse
//! - `id`: Unique identifier for the lift
//! - `name`: Name of the lift
//! - `lift_type`: Type of lift (e.g., "Chairlift", "Gondola", "T-bar", "Surface lift")
//! - `status`: Current operational status of the lift
//!
//! ## SlopeResponse
//! - `id`: Unique identifier for the slope
//! - `name`: Name of the slope
//! - `difficulty`: Difficulty level (e.g., "Green", "Blue", "Red", "Black")
//! - `length_km`: Length of the slope in kilometers
//!
//! Author: OpenSlope Team
//! Version: 1.0.0

use serde::Serialize;
use serde_json::Value;
use super::lift::LiftResponse;
use super::slope::SlopeResponse;

/// Complete ski resort information for API responses
///
/// This is the main response model that contains comprehensive information about
/// a ski resort, including its location, altitude data, ski area details, and
/// all associated lifts and slopes.
///
/// # Example
/// ```rust
/// let resort = ResortResponse {
///     id: "resort_123".to_string(),
///     name: "Example Resort".to_string(),
///     location: LocationBlock {
///         country: "Austria".to_string(),
///         region: "Tyrol".to_string(),
///         continent: "Europe".to_string(),
///         latitude: 47.2628,
///         longitude: 11.3936,
///     },
///     altitude: AltitudeBlock {
///         village_altitude_m: 1200,
///         min_altitude_m: 1100,
///         max_altitude_m: 2500,
///     },
///     ski_area: SkiAreaBlock {
///         name: "Example Ski Area".to_string(),
///         area_type: "Alpine".to_string(),
///         total_slope_km: Some(50.5),
///         total_lifts: Some(15),
///         snowmaking_percent: Some(80),
///         night_skiing: Some(false),
///     },
///     lifts: vec![],
///     slopes: vec![],
/// };
/// ```
#[derive(Serialize)]
pub struct ResortResponse {
    /// Unique identifier for the resort
    pub id: String,
    /// Official name of the resort
    pub name: String,
    /// Resort type or category (e.g. "alpine", "cross-country")
    pub r#type: Option<String>,
    /// Current status of the resort
    pub status: Option<String>,
    /// Wikidata identifier for the resort
    pub wikidata_id: Option<String>,
    /// Geographical and administrative location information
    pub location: LocationBlock,
    /// Elevation data for the resort
    pub altitude: AltitudeBlock,
    /// Information about the ski area operations
    pub ski_area: SkiAreaBlock,
    /// General place-level metadata from the geojson import
    pub places: Option<Value>,
    /// Source metadata for the resort record
    pub sources: Option<Value>,
    /// Website links associated with the resort
    pub websites: Option<Value>,
    /// Activity flags and additional operations metadata
    pub activities: Option<Value>,
    /// Statistical information for the resort
    pub statistics: Option<Value>,
    /// Viewport hint for map rendering
    pub viewport_hint: Option<Value>,
    /// Run convention metadata for the resort
    pub run_convention: Option<String>,
    /// Geometry type of the resort location
    pub geometry_type: Option<String>,
    /// GeoJSON geometry payload for the resort
    pub geometry: Option<Value>,
    /// Additional GeoJSON properties for the resort
    pub properties: Option<Value>,
    /// Database creation timestamp
    pub created_at: Option<String>,
    /// Database update timestamp
    pub updated_at: Option<String>,
    /// List of all lifts in the resort
    pub lifts: Vec<LiftResponse>,
    /// List of all slopes in the resort
    pub slopes: Vec<SlopeResponse>,
}

/// Geographical and administrative location information
///
/// Contains the country, region, continent, and precise coordinates of a ski resort.
///
/// # Coordinate System
/// - Latitude and longitude use the WGS84 coordinate system
/// - Latitude ranges from -90 to +90 degrees
/// - Longitude ranges from -180 to +180 degrees
///
/// # Example
/// ```rust
/// let location = LocationBlock {
///     country: "Austria".to_string(),
///     region: "Tyrol".to_string(),
///     continent: "Europe".to_string(),
///     latitude: 47.2628,
///     longitude: 11.3936,
/// };
/// ```
#[derive(Serialize)]
pub struct LocationBlock {
    /// Country where the resort is located
    pub country: Option<String>,
    /// Administrative region within the country
    pub region: Option<String>,
    /// Continent where the resort is located
    pub continent: Option<String>,
    /// Geographic latitude coordinate (WGS84)
    pub latitude: Option<f64>,
    /// Geographic longitude coordinate (WGS84)
    pub longitude: Option<f64>,
}

/// Elevation information for a ski resort
///
/// Contains altitude data for different parts of the resort, which is important
/// for skiers to understand the vertical range and difficulty of the terrain.
///
/// # Altitude Measurements
/// - All measurements are in meters above sea level
/// - Village altitude represents the base area or main village elevation
/// - Min/max altitudes represent the skiable elevation range
///
/// # Example
/// ```rust
/// let altitude = AltitudeBlock {
///     village_altitude_m: 1200,
///     min_altitude_m: 1100,
///     max_altitude_m: 2500,
/// };
/// // This resort has a vertical drop of 1400 meters
/// ```
#[derive(Serialize)]
pub struct AltitudeBlock {
    /// Altitude of the resort village/base area in meters
    pub village_altitude_m: Option<i32>,
    /// Minimum skiable altitude in meters
    pub min_altitude_m: Option<i32>,
    /// Maximum skiable altitude in meters
    pub max_altitude_m: Option<i32>,
}

/// Ski area operational information
///
/// Contains details about the ski area including its name, type, and various
/// operational statistics. This information helps skiers understand the
/// capabilities and features of the ski area.
///
/// # Optional Fields
/// - Statistics like total slope length and number of lifts may not be available
///   for all resorts, hence they are optional
/// - Snowmaking percentage indicates the proportion of slopes with artificial snow
/// - Night skiing indicates whether slopes are lit for evening skiing
///
/// # Example
/// ```rust
/// let ski_area = SkiAreaBlock {
///     name: "Example Ski Area".to_string(),
///     area_type: "Alpine".to_string(),
///     total_slope_km: Some(50.5),
///     total_lifts: Some(15),
///     snowmaking_percent: Some(80),
///     night_skiing: Some(false),
/// };
/// ```
#[derive(Serialize)]
pub struct SkiAreaBlock {
    /// Name of the ski area
    pub name: Option<String>,
    /// Type of ski area (e.g., "Alpine", "Nordic", "Cross-country")
    pub area_type: Option<String>,
    /// Total length of all slopes in kilometers (optional)
    pub total_slope_km: Option<f64>,
    /// Total number of lifts in the area (optional)
    pub total_lifts: Option<i32>,
    /// Percentage of slopes with snowmaking coverage (optional)
    pub snowmaking_percent: Option<i32>,
    /// Whether night skiing is available (optional)
    pub night_skiing: Option<bool>,
    /// Current status of the ski area
    pub status: Option<String>,
    /// Wikidata identifier for the ski area
    pub wikidata_id: Option<String>,
    /// General place-level metadata
    pub places: Option<Value>,
    /// Source metadata for the ski area
    pub sources: Option<Value>,
    /// Website metadata for the ski area
    pub websites: Option<Value>,
    /// Activity metadata for the ski area
    pub activities: Option<Value>,
    /// Statistical metadata for the ski area
    pub statistics: Option<Value>,
    /// Viewport hint for the ski area
    pub viewport_hint: Option<Value>,
    /// Run convention metadata for the ski area
    pub run_convention: Option<String>,
    /// Geometry type for the ski area
    pub geometry_type: Option<String>,
    /// GeoJSON geometry payload for the ski area
    pub geometry: Option<Value>,
    /// Additional GeoJSON properties for the ski area
    pub properties: Option<Value>,
}

/// Individual lift information for API responses
///
/// Represents a single lift within a ski resort, including its type and current status.
/// This information is useful for skiers planning their day on the mountain.
///
/// # Lift Types
/// Common lift types include:
/// - "Chairlift": Detachable or fixed-grip chairlifts
/// - "Gondola": Enclosed cabin lifts
/// - "T-bar": Surface lift with T-shaped bars
/// - "Surface lift": Magic carpet or similar surface lifts
///
/// # Status Values
/// Status typically indicates whether the lift is operational:
/// - "Open": Lift is currently operating
/// - "Closed": Lift is not operating
/// - "Maintenance": Lift is under maintenance
///

/// Individual slope information for API responses
///
/// Represents a single slope within a ski resort, including its difficulty level
/// and length. This information helps skiers choose appropriate runs for their
/// skill level and plan their day on the mountain.
///
/// # Difficulty Levels
/// Standard difficulty classifications:
/// - "Green": Beginner slopes, gentle gradients
/// - "Blue": Intermediate slopes, moderate gradients
/// - "Red": Advanced slopes, steep gradients
/// - "Black": Expert slopes, very steep and challenging
///
/// # Length Measurement
/// - Length is measured in kilometers
/// - Represents the total distance from top to bottom of the slope
/// - May include variations for different route options
///
