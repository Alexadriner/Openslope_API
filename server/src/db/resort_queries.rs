use crate::models::db::{ResortRow, LiftRow, SlopeRow};
use sqlx::MySqlPool;

pub async fn get_resort_full(
    pool: &MySqlPool,
    resort_id: &str,
) -> Result<(ResortRow, Vec<LiftRow>, Vec<SlopeRow>), sqlx::Error> {

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