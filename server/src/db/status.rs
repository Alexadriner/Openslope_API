use sqlx::{FromRow, MySql, MySqlPool, QueryBuilder};
use std::collections::HashMap;

#[derive(Debug, Clone, FromRow)]
pub struct ResortStatusSnapshotRow {
    pub resort_id: String,
    pub snapshot_time: Option<String>,
    pub lifts_open_count: Option<i32>,
    pub lifts_total_count: Option<i32>,
    pub slopes_open_count: Option<i32>,
    pub slopes_total_count: Option<i32>,
    pub snow_depth_valley_cm: Option<i16>,
    pub snow_depth_mountain_cm: Option<i16>,
    pub new_snow_24h_cm: Option<i16>,
    pub temperature_valley_c: Option<f64>,
    pub temperature_mountain_c: Option<f64>,
}

pub async fn get_latest_for_resorts(
    pool: &MySqlPool,
    resort_ids: &[String],
) -> Result<HashMap<String, ResortStatusSnapshotRow>, sqlx::Error> {
    if resort_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut builder = QueryBuilder::<MySql>::new(
        r#"
        SELECT
            rss.resort_id,
            DATE_FORMAT(rss.snapshot_time, '%Y-%m-%dT%H:%i:%sZ') AS snapshot_time,
            rss.lifts_open_count,
            rss.lifts_total_count,
            rss.slopes_open_count,
            rss.slopes_total_count,
            rss.snow_depth_valley_cm,
            rss.snow_depth_mountain_cm,
            rss.new_snow_24h_cm,
            CAST(rss.temperature_valley_c AS DOUBLE) AS temperature_valley_c,
            CAST(rss.temperature_mountain_c AS DOUBLE) AS temperature_mountain_c
        FROM resort_status_snapshots rss
        INNER JOIN (
            SELECT resort_id, MAX(snapshot_time) AS max_snapshot_time
            FROM resort_status_snapshots
            WHERE resort_id IN (
        "#,
    );

    let mut separated = builder.separated(", ");
    for resort_id in resort_ids {
        separated.push_bind(resort_id);
    }
    separated.push_unseparated(
        r#"
            )
            GROUP BY resort_id
        ) latest
            ON latest.resort_id = rss.resort_id
           AND latest.max_snapshot_time = rss.snapshot_time
        ORDER BY rss.snapshot_time DESC
        "#,
    );

    let rows = builder
        .build_query_as::<ResortStatusSnapshotRow>()
        .fetch_all(pool)
        .await?;

    let mut snapshots = HashMap::new();
    for row in rows {
        snapshots.entry(row.resort_id.clone()).or_insert(row);
    }

    Ok(snapshots)
}
