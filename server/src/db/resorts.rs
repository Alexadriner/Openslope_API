use crate::models::db::{Place, ResortRow};
use serde_json::Value;
use sqlx::MySqlPool;

#[derive(sqlx::FromRow)]
struct RawResortRow {
    id: String,
    name: String,
    #[sqlx(rename = "type")]
    r#type: Option<String>,
    status: Option<String>,
    activities: Option<Value>,
    geometry: Value,
    websites: Option<Value>,
    sources: Option<Value>,
}

pub async fn get_all(pool: &MySqlPool) -> Result<Vec<ResortRow>, sqlx::Error> {
    let raw_resorts = sqlx::query_as::<_, RawResortRow>(
        r#"
        SELECT
            id,
            name,
            `type`,
            status,
            activities,
            ST_AsGeoJSON(geometry) AS geometry,
            websites,
            sources
        FROM resorts
        ORDER BY name
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut resorts = raw_resorts
        .into_iter()
        .map(map_resort_row)
        .collect::<Vec<_>>();
    populate_resort_places(pool, &mut resorts).await?;
    Ok(resorts)
}

pub async fn get_by_id(
    pool: &MySqlPool,
    resort_id: &str,
) -> Result<Option<ResortRow>, sqlx::Error> {
    let mut resort = sqlx::query_as::<_, RawResortRow>(
        r#"
        SELECT
            id,
            name,
            `type`,
            status,
            activities,
            ST_AsGeoJSON(geometry) AS geometry,
            websites,
            sources
        FROM resorts
        WHERE id = ?
        "#,
    )
    .bind(resort_id)
    .fetch_optional(pool)
    .await?
    .map(map_resort_row);

    if let Some(resort) = resort.as_mut() {
        let places_by_resort =
            crate::db::places::get_for_resorts(pool, &[resort.id.clone()]).await?;
        resort.places = places_by_resort
            .get(&resort.id)
            .cloned()
            .unwrap_or_default();
    }

    Ok(resort)
}

pub async fn insert(
    pool: &MySqlPool,
    id: &str,
    name: &str,
    resort_type: Option<&str>,
    status: Option<&str>,
    activities: Option<&Value>,
    geometry_wkt: &str,
    websites: Option<&Value>,
    sources: Option<&Value>,
    places: &[Place],
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO resorts
            (id, name, `type`, status, activities, geometry, websites, sources)
        VALUES
            (?, ?, ?, ?, ?, ST_GeomFromText(?, 4326, 'axis-order=long-lat'), ?, ?)
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(resort_type)
    .bind(status)
    .bind(activities)
    .bind(geometry_wkt)
    .bind(websites)
    .bind(sources)
    .execute(pool)
    .await?;

    crate::db::places::sync_resort_places(pool, id, places).await?;
    Ok(())
}

pub async fn update(
    pool: &MySqlPool,
    id: &str,
    name: &str,
    resort_type: Option<&str>,
    status: Option<&str>,
    activities: Option<&Value>,
    geometry_wkt: &str,
    websites: Option<&Value>,
    sources: Option<&Value>,
    places: &[Place],
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE resorts SET
            name = ?,
            `type` = ?,
            status = ?,
            activities = ?,
            geometry = ST_GeomFromText(?, 4326, 'axis-order=long-lat'),
            websites = ?,
            sources = ?
        WHERE id = ?
        "#,
    )
    .bind(name)
    .bind(resort_type)
    .bind(status)
    .bind(activities)
    .bind(geometry_wkt)
    .bind(websites)
    .bind(sources)
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() > 0 {
        crate::db::places::sync_resort_places(pool, id, places).await?;
    }

    Ok(result.rows_affected())
}

pub async fn delete(pool: &MySqlPool, id: &str) -> Result<u64, sqlx::Error> {
    crate::db::places::delete_resort_places(pool, id).await?;

    let result = sqlx::query("DELETE FROM resorts WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected())
}

fn map_resort_row(row: RawResortRow) -> ResortRow {
    ResortRow {
        id: row.id,
        name: row.name,
        r#type: row.r#type,
        status: row.status,
        activities: row.activities,
        geometry: row.geometry,
        websites: row.websites,
        sources: row.sources,
        places: Vec::new(),
    }
}

async fn populate_resort_places(
    pool: &MySqlPool,
    resorts: &mut [ResortRow],
) -> Result<(), sqlx::Error> {
    let resort_ids = resorts
        .iter()
        .map(|resort| resort.id.clone())
        .collect::<Vec<_>>();
    let places_by_resort = crate::db::places::get_for_resorts(pool, &resort_ids).await?;

    for resort in resorts {
        resort.places = places_by_resort
            .get(&resort.id)
            .cloned()
            .unwrap_or_default();
    }

    Ok(())
}
