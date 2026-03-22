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
    /// Geographical and administrative location information
    pub location: LocationBlock,
    /// Elevation data for the resort
    pub altitude: AltitudeBlock,
    /// Information about the ski area operations
    pub ski_area: SkiAreaBlock,
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
    pub country: String,
    /// Administrative region within the country
    pub region: String,
    /// Continent where the resort is located
    pub continent: String,
    /// Geographic latitude coordinate (WGS84)
    pub latitude: f64,
    /// Geographic longitude coordinate (WGS84)
    pub longitude: f64,
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
    pub village_altitude_m: i32,
    /// Minimum skiable altitude in meters
    pub min_altitude_m: i32,
    /// Maximum skiable altitude in meters
    pub max_altitude_m: i32,
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
    pub name: String,
    /// Type of ski area (e.g., "Alpine", "Nordic", "Cross-country")
    pub area_type: String,
    /// Total length of all slopes in kilometers (optional)
    pub total_slope_km: Option<f64>,
    /// Total number of lifts in the area (optional)
    pub total_lifts: Option<i32>,
    /// Percentage of slopes with snowmaking coverage (optional)
    pub snowmaking_percent: Option<i32>,
    /// Whether night skiing is available (optional)
    pub night_skiing: Option<bool>,
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
/// # Example
/// ```rust
/// let lift = LiftResponse {
///     id: "lift_001".to_string(),
///     name: "Main Chair".to_string(),
///     lift_type: "Chairlift".to_string(),
///     status: "Open".to_string(),
/// };
/// ```
#[derive(Serialize)]
pub struct LiftResponse {
    /// Unique identifier for the lift
    pub id: String,
    /// Name of the lift
    pub name: String,
    /// Type of lift (e.g., "Chairlift", "Gondola", "T-bar")
    pub lift_type: String,
    /// Current operational status of the lift
    pub status: String,
}

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
/// # Example
/// ```rust
/// let slope = SlopeResponse {
///     id: "slope_001".to_string(),
///     name: "Black Diamond Run".to_string(),
///     difficulty: "Black".to_string(),
///     length_km: 2.5,
/// };
/// ```
#[derive(Serialize)]
pub struct SlopeResponse {
    /// Unique identifier for the slope
    pub id: String,
    /// Name of the slope
    pub name: String,
    /// Difficulty level (e.g., "Green", "Blue", "Red", "Black")
    pub difficulty: String,
    /// Length of the slope in kilometers
    pub length_km: f64,
}
