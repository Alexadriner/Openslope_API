use crate::models::db::ElevationProfileRow;
use serde_json::Value;
use sqlx::MySqlPool;

pub async fn get_all(pool: &MySqlPool) -> Result<Vec<ElevationProfileRow>, sqlx::Error> {
    sqlx::query_as::<_, ElevationProfileRow>(
        r#"
        SELECT
            id,
            entity_type,
            entity_id,
            heights,
            resolution,
            target_resolution
        FROM elevation_profiles
        ORDER BY id
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn get_by_id(
    pool: &MySqlPool,
    profile_id: &str,
) -> Result<Option<ElevationProfileRow>, sqlx::Error> {
    sqlx::query_as::<_, ElevationProfileRow>(
        r#"
        SELECT
            id,
            entity_type,
            entity_id,
            heights,
            resolution,
            target_resolution
        FROM elevation_profiles
        WHERE id = ?
        "#,
    )
    .bind(profile_id)
    .fetch_optional(pool)
    .await
}

pub async fn insert(
    pool: &MySqlPool,
    id: &str,
    entity_type: &str,
    entity_id: &str,
    heights: Option<&Value>,
    resolution: Option<f64>,
    target_resolution: Option<f64>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO elevation_profiles
            (id, entity_type, entity_id, heights, resolution, target_resolution)
        VALUES
            (?, ?, ?, ?, ?, ?)
        "#,
    )
    .bind(id)
    .bind(entity_type)
    .bind(entity_id)
    .bind(heights)
    .bind(resolution)
    .bind(target_resolution)
    .execute(pool)
    .await
    .map(|_| ())
}

pub async fn update(
    pool: &MySqlPool,
    id: &str,
    entity_type: &str,
    entity_id: &str,
    heights: Option<&Value>,
    resolution: Option<f64>,
    target_resolution: Option<f64>,
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE elevation_profiles SET
            entity_type = ?,
            entity_id = ?,
            heights = ?,
            resolution = ?,
            target_resolution = ?
        WHERE id = ?
        "#,
    )
    .bind(entity_type)
    .bind(entity_id)
    .bind(heights)
    .bind(resolution)
    .bind(target_resolution)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(result.rows_affected())
}
