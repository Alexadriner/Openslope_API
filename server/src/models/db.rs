//! Database Models
//!
//! This module defines the database row structures used throughout the OpenSlope API.
//! These models represent the data structures that are directly mapped from the
//! MySQL database tables using SQLx's FromRow derive macro.
//!
//! The models are designed to provide type-safe database interactions while
//! maintaining compatibility with the API's serialization requirements.
//!
//! # Architecture
//!
//! The module contains three main data models:
//! - **ResortRow**: Complete resort information including geographical and operational data
//! - **LiftRow**: Individual lift information associated with resorts
//! - **SlopeRow**: Individual slope information associated with resorts
//!
//! # Design Principles
//!
//! - **Type Safety**: All fields use appropriate Rust types with proper null handling
//! - **Serialization**: All models implement Serialize for JSON API responses
//! - **Database Mapping**: Uses sqlx::FromRow for automatic database row mapping
//! - **Optional Fields**: All non-critical fields are optional to handle incomplete data
//!
//! Author: OpenSlope Team
//! Version: 1.0.0

use serde::{Deserialize, Serialize};

/// Database representation of a ski resort
///
/// This struct maps directly to the `resorts` table in the database and contains
/// comprehensive information about a ski resort including geographical location,
/// altitude data, and operational details.
///
/// # Fields
///
/// - `id`: Unique identifier for the resort (required)
/// - `name`: Official name of the resort (required)
/// - `country`: Country where the resort is located (optional)
/// - `region`: Administrative region within the country (optional)
/// - `continent`: Continent where the resort is located (optional)
/// - `latitude`: Geographic latitude coordinate (optional)
/// - `longitude`: Geographic longitude coordinate (optional)
/// - `village_altitude_m`: Altitude of the resort village in meters (optional)
/// - `min_altitude_m`: Minimum skiable altitude in meters (optional)
/// - `max_altitude_m`: Maximum skiable altitude in meters (optional)
/// - `ski_area_name`: Name of the ski area (optional)
/// - `ski_area_type`: Type of ski area (e.g., "Alpine", "Nordic") (optional)
///
/// # Database Mapping
///
/// This struct is automatically mapped from database rows using SQLx's `FromRow` derive macro.
/// The field names correspond directly to the column names in the `resorts` table.
///
/// # Usage
///
/// ```rust
/// let resort: ResortRow = sqlx::query_as!(ResortRow, "SELECT * FROM resorts WHERE id = ?", resort_id)
///     .fetch_one(&pool)
///     .await?;
/// ```
#[derive(sqlx::FromRow, Debug, serde::Serialize)]
pub struct ResortRow {
    pub id: String,
    pub name: String,

    pub country: Option<String>,
    pub region: Option<String>,
    pub continent: Option<String>,

    pub latitude: Option<f64>,
    pub longitude: Option<f64>,

    pub village_altitude_m: Option<i32>,
    pub min_altitude_m: Option<i32>,
    pub max_altitude_m: Option<i32>,

    pub ski_area_name: Option<String>,
    pub ski_area_type: Option<String>,
}

/// Database representation of a ski lift
///
/// This struct maps directly to the `lifts` table in the database and contains
/// information about individual lifts within a ski resort.
///
/// # Fields
///
/// - `id`: Unique database identifier for the lift (required)
/// - `resort_id`: Foreign key linking to the parent resort (required)
/// - `name`: Name of the lift (optional)
/// - `lift_type`: Type of lift (e.g., "Chairlift", "Gondola", "T-bar") (optional)
///
/// # Important Notes
///
/// - The `name` and `lift_type` fields are marked as important in the original code
/// - These fields may be empty or null in the database, hence they are optional
/// - The struct implements Serialize for API responses
///
/// # Database Mapping
///
/// This struct is automatically mapped from database rows using SQLx's `FromRow` derive macro.
/// The field names correspond directly to the column names in the `lifts` table.
///
/// # Usage
///
/// ```rust
/// let lifts: Vec<LiftRow> = sqlx::query_as!(LiftRow, "SELECT * FROM lifts WHERE resort_id = ?", resort_id)
///     .fetch_all(&pool)
///     .await?;
/// ```
#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct LiftRow {
    pub id: i64,
    pub resort_id: String,
    pub name: Option<String>,      // ← wichtig
    pub lift_type: Option<String>, // ← wichtig
}

/// Database representation of a ski slope
///
/// This struct maps directly to the `slopes` table in the database and contains
/// information about individual slopes within a ski resort.
///
/// # Fields
///
/// - `id`: Unique database identifier for the slope (required)
/// - `resort_id`: Foreign key linking to the parent resort (required)
/// - `name`: Name of the slope (optional)
/// - `difficulty`: Difficulty level of the slope (e.g., "Green", "Blue", "Red", "Black") (optional)
///
/// # Important Notes
///
/// - The `name` and `difficulty` fields are marked as important in the original code
/// - These fields may be empty or null in the database, hence they are optional
/// - The struct implements Serialize for API responses
///
/// # Database Mapping
///
/// This struct is automatically mapped from database rows using SQLx's `FromRow` derive macro.
/// The field names correspond directly to the column names in the `slopes` table.
///
/// # Usage
///
/// ```rust
/// let slopes: Vec<SlopeRow> = sqlx::query_as!(SlopeRow, "SELECT * FROM slopes WHERE resort_id = ?", resort_id)
///     .fetch_all(&pool)
///     .await?;
/// ```
#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct SlopeRow {
    pub id: i64,
    pub resort_id: String,
    pub name: Option<String>,        // ← wichtig
    pub difficulty: Option<String>,  // ← wichtig
}