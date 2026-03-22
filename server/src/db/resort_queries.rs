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
//! The module follows a layered approach:
//! - **Single Query Functions**: Efficient retrieval of complete resort data
//! - **Type Safety**: Uses SQLx's query_as! macro for compile-time type checking
//! - **Error Handling**: Propagates SQLx errors for proper error handling
//!
//! # Performance Considerations
//!
//! - Uses prepared statements for better performance
//! - Fetches related data (lifts, slopes) in separate queries to avoid cartesian products
//! - Leverages SQLx's async capabilities for non-blocking database operations
//!
//! Author: OpenSlope Team
//! Version: 1.0.0

use crate::models::db::{ResortRow, LiftRow, SlopeRow};
use sqlx::MySqlPool;

/// Retrieve a complete resort with all associated lifts and slopes
///
/// This function performs three separate database queries to fetch:
/// 1. The resort details
/// 2. All lifts associated with the resort
/// 3. All slopes associated with the resort
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
/// - `ResortRow`: The resort's basic information
/// - `Vec<LiftRow>`: All lifts associated with the resort
/// - `Vec<SlopeRow>`: All slopes associated with the resort
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
/// let (resort, lifts, slopes) = get_resort_full(&pool, "resort_123").await?;
/// println!("Resort: {}", resort.name);
/// println!("Lifts: {}", lifts.len());
/// println!("Slopes: {}", slopes.len());
/// ```
///
/// # SQL Queries
///
/// The function executes these three queries:
///
/// 1. **Resort Query**: Retrieves basic resort information including
///    geographical coordinates, altitude data, and area details
///
/// 2. **Lifts Query**: Fetches all lifts associated with the resort,
///    including their names and types
///
/// 3. **Slopes Query**: Retrieves all slopes for the resort with their
///    names and difficulty levels
///
/// # Performance Notes
///
/// - Uses `fetch_one()` for the resort query since each resort ID is unique
/// - Uses `fetch_all()` for lifts and slopes as there can be multiple entries
/// - All queries are executed sequentially to maintain data consistency
/// - The function is async and non-blocking, allowing for concurrent operations
pub async fn get_resort_full(
    pool: &MySqlPool,
    resort_id: &str,
) -> Result<(ResortRow, Vec<LiftRow>, Vec<SlopeRow>), sqlx::Error> {

    // Query 1: Fetch resort details
    let resort = sqlx::query_as!(
        ResortRow,
        r#"
        SELECT
            id, name, country, region, continent,
            latitude, longitude,
            village_altitude_m, min_altitude_m, max_altitude_m,
            ski_area_name, ski_area_type
        FROM resorts
        WHERE id = ?
        "#,
        resort_id
    )
    .fetch_one(pool)
    .await?;

    // Query 2: Fetch associated lifts
    let lifts = sqlx::query_as!(
        LiftRow,
        r#"
        SELECT id, resort_id, name, lift_type
        FROM lifts
        WHERE resort_id = ?
        "#,
        resort_id
    )
    .fetch_all(pool)
    .await?;

    // Query 3: Fetch associated slopes
    let slopes = sqlx::query_as!(
        SlopeRow,
        r#"
        SELECT id, resort_id, name, difficulty
        FROM slopes
        WHERE resort_id = ?
        "#,
        resort_id
    )
    .fetch_all(pool)
    .await?;

    Ok((resort, lifts, slopes))
}