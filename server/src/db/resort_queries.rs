//! Resort Database Queries
//!
//! This module contains database query functions specifically for resort-related
//! operations. It provides efficient querying capabilities for retrieving
//! resort data along with their associated lifts and slopes.
//!
//! The queries are optimized for the OpenSlope API's data access patterns and
//! use SQLx for type-safe database interactions.
//!
//! # Architecture
//!
//! - `ResortRow`: The resort's complete database row using the current schema
//! - `GeojsonLiftRow`: Lift rows imported from GeoJSON into `geojson_lifts`
//! - `GeojsonRunRow`: Run rows imported from GeoJSON into `geojson_runs`
//!
//! # Performance Considerations
//!
//! - Uses prepared statements for better performance
//! - Fetches related data separately to avoid large cartesian products
//! - Leverages SQLx's async capabilities for non-blocking database operations
//!
//! Author: OpenSlope Team
//! Version: 1.0.0

use crate::models::db::{GeojsonLiftRow, GeojsonRunRow, ResortRow};
use sqlx::MySqlPool;

/// Retrieve a complete resort with all associated lifts and slopes
///
/// This function performs three separate database queries to fetch:
/// 1. The resort details
/// 2. All lifts associated with the resort
/// 3. All runs associated with the resort
///
/// The separation prevents cartesian products that would occur with JOINs,
/// ensuring accurate data retrieval and better performance.
///
/// # Arguments
///
/// * `pool` - A reference to the MySQL connection pool
/// * `resort_id` - The unique identifier of the resort to retrieve
///
/// # Returns
///
/// Returns a `Result` containing a tuple of:
/// - `ResortRow`: The resort's full database row
/// - `Vec<GeojsonLiftRow>`: All lifts associated with the resort
/// - `Vec<GeojsonRunRow>`: All runs associated with the resort
///
/// # Errors
///
/// Returns `sqlx::Error` if:
/// - The database connection fails
/// - The resort ID doesn't exist
/// - Any of the queries fail to execute
///
/// # Example
///
/// ```rust
/// let (resort, lifts, runs) = get_resort_full(&pool, "resort_123").await?;
/// println!("Resort: {:?}", resort.name);
/// ```
pub async fn get_resort_full(
    pool: &MySqlPool,
    resort_id: &str,
) -> Result<(ResortRow, Vec<GeojsonLiftRow>, Vec<GeojsonRunRow>), sqlx::Error> {
    let resort = sqlx::query_as!(
        ResortRow,
        r#"
        SELECT
            id,
            name,
            `type`,
            status,
            places,
            sources,
            websites,
            activities,
            statistics,
            wikidata_id,
            viewport_hint,
            run_convention,
            geometry_type,
            geometry,
            properties,
            created_at,
            updated_at
        FROM resorts
        WHERE id = ?
        "#,
        resort_id
    )
    .fetch_one(pool)
    .await?;

    let lifts = sqlx::query_as!(
        GeojsonLiftRow,
        r#"
        SELECT
            id,
            `ref`,
            name,
            `type`,
            access,
            bubble,
            oneway,
            status,
            tunnel,
            heating,
            capacity,
            duration,
            lift_type,
            occupancy,
            detachable,
            ref_frcairn,
            wikidata_id,
            description,
            viewport_hint,
            places,
            sources,
            ski_areas,
            stations,
            websites,
            geometry_type,
            geometry,
            properties,
            created_at,
            updated_at
        FROM geojson_lifts
        WHERE JSON_CONTAINS(ski_areas, JSON_QUOTE(?))
        ORDER BY name
        "#,
        resort_id
    )
    .fetch_all(pool)
    .await?;

    let runs = sqlx::query_as!(
        GeojsonRunRow,
        r#"
        SELECT
            id,
            lit,
            `ref`,
            name,
            `type`,
            uses,
            gladed,
            oneway,
            places,
            status,
            tunnel,
            sources,
            grooming,
            ski_areas,
            websites,
            patrolled,
            difficulty,
            snowmaking,
            wikidata_id,
            description,
            snowfarming,
            viewport_hint,
            elevation_profile,
            difficulty_convention,
            geometry_type,
            geometry,
            properties,
            created_at,
            updated_at
        FROM geojson_runs
        WHERE JSON_CONTAINS(ski_areas, JSON_QUOTE(?))
        ORDER BY name
        "#,
        resort_id
    )
    .fetch_all(pool)
    .await?;

    Ok((resort, lifts, runs))
}
