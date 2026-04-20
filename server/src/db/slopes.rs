use crate::models::db::{Place, ResortRow, SlopeRow};
use serde_json::Value;
use sqlx::MySqlPool;
use std::collections::HashMap;

pub struct SlopeWithResorts {
    pub slope: SlopeRow,
    pub resorts: Vec<ResortRow>,
}

pub async fn get_all(pool: &MySqlPool) -> Result<Vec<SlopeWithResorts>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            s.id as slope_id,
            s.name as slope_name,
            s.difficulty as slope_difficulty,
            s.status as slope_status,
            s.grooming as slope_grooming,
            ST_AsGeoJSON(s.geometry) as slope_geometry,
            s.websites as slope_websites,
            s.sources as slope_sources,
            r.id as resort_id,
            r.name as resort_name,
            r.type as resort_type,
            r.status as resort_status,
            r.activities as resort_activities,
            ST_AsGeoJSON(r.geometry) as resort_geometry,
            r.websites as resort_websites,
            r.sources as resort_sources
        FROM slopes s
        LEFT JOIN slope_resorts sr ON s.id = sr.slope_id
        LEFT JOIN resorts r ON sr.resort_id = r.id
        ORDER BY s.name, r.name
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut slope_map: HashMap<String, SlopeWithResorts> = HashMap::new();

    for row in rows {
        let slope_id = row.slope_id.clone();
        let slope = slope_map
            .entry(slope_id.clone())
            .or_insert_with(|| SlopeWithResorts {
                slope: SlopeRow {
                    id: row.slope_id.clone(),
                    name: row.slope_name.clone(),
                    difficulty: row.slope_difficulty.clone(),
                    status: row.slope_status.clone(),
                    grooming: row.slope_grooming.clone(),
                    geometry: row.slope_geometry.clone().unwrap_or_default(),
                    websites: row.slope_websites.clone(),
                    sources: row.slope_sources.clone(),
                    places: Vec::new(),
                },
                resorts: Vec::new(),
            });

        if let Some(resort_id) = row.resort_id {
            slope.resorts.push(ResortRow {
                id: resort_id,
                name: row.resort_name.unwrap_or_default(),
                r#type: row.resort_type,
                status: row.resort_status,
                activities: row.resort_activities,
                geometry: row.resort_geometry.unwrap_or_default(),
                websites: row.resort_websites,
                sources: row.resort_sources,
                places: Vec::new(),
            });
        }
    }

    populate_slope_places(pool, &mut slope_map).await?;

    let mut result: Vec<SlopeWithResorts> = slope_map.into_iter().map(|(_, v)| v).collect();
    result.sort_by(|a, b| a.slope.name.cmp(&b.slope.name));
    Ok(result)
}

pub async fn get_by_id(
    pool: &MySqlPool,
    slope_id: &str,
) -> Result<Option<SlopeWithResorts>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            s.id as slope_id,
            s.name as slope_name,
            s.difficulty as slope_difficulty,
            s.status as slope_status,
            s.grooming as slope_grooming,
            ST_AsGeoJSON(s.geometry) as slope_geometry,
            s.websites as slope_websites,
            s.sources as slope_sources,
            r.id as resort_id,
            r.name as resort_name,
            r.type as resort_type,
            r.status as resort_status,
            r.activities as resort_activities,
            ST_AsGeoJSON(r.geometry) as resort_geometry,
            r.websites as resort_websites,
            r.sources as resort_sources
        FROM slopes s
        LEFT JOIN slope_resorts sr ON s.id = sr.slope_id
        LEFT JOIN resorts r ON sr.resort_id = r.id
        WHERE s.id = ?
        ORDER BY r.name
        "#,
        slope_id
    )
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let mut resorts = Vec::new();
    let slope_row = &rows[0];

    for row in &rows {
        if let Some(resort_id) = &row.resort_id {
            resorts.push(ResortRow {
                id: resort_id.clone(),
                name: row.resort_name.clone().unwrap_or_default(),
                r#type: row.resort_type.clone(),
                status: row.resort_status.clone(),
                activities: row.resort_activities.clone(),
                geometry: row.resort_geometry.clone().unwrap_or_default(),
                websites: row.resort_websites.clone(),
                sources: row.resort_sources.clone(),
                places: Vec::new(),
            });
        }
    }

    let slope = SlopeWithResorts {
        slope: SlopeRow {
            id: slope_row.slope_id.clone(),
            name: slope_row.slope_name.clone(),
            difficulty: slope_row.slope_difficulty.clone(),
            status: slope_row.slope_status.clone(),
            grooming: slope_row.slope_grooming.clone(),
            geometry: slope_row.slope_geometry.clone().unwrap_or_default(),
            websites: slope_row.slope_websites.clone(),
            sources: slope_row.slope_sources.clone(),
            places: Vec::new(),
        },
        resorts,
    };

    let mut slope_map = HashMap::from([(slope.slope.id.clone(), slope)]);
    populate_slope_places(pool, &mut slope_map).await?;

    Ok(slope_map.remove(slope_id))
}

pub async fn get_by_resort(
    pool: &MySqlPool,
    resort_id: &str,
) -> Result<Vec<SlopeWithResorts>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            s.id as slope_id,
            s.name as slope_name,
            s.difficulty as slope_difficulty,
            s.status as slope_status,
            s.grooming as slope_grooming,
            ST_AsGeoJSON(s.geometry) as slope_geometry,
            s.websites as slope_websites,
            s.sources as slope_sources,
            r.id as resort_id,
            r.name as resort_name,
            r.type as resort_type,
            r.status as resort_status,
            r.activities as resort_activities,
            ST_AsGeoJSON(r.geometry) as resort_geometry,
            r.websites as resort_websites,
            r.sources as resort_sources
        FROM slopes s
        INNER JOIN slope_resorts sr ON sr.slope_id = s.id
        INNER JOIN resorts r ON sr.resort_id = r.id
        WHERE sr.resort_id = ?
        ORDER BY s.name
        "#,
        resort_id
    )
    .fetch_all(pool)
    .await?;

    let mut slope_map: HashMap<String, SlopeWithResorts> = HashMap::new();

    for row in rows {
        let slope_id = row.slope_id.clone();
        let slope = slope_map
            .entry(slope_id.clone())
            .or_insert_with(|| SlopeWithResorts {
                slope: SlopeRow {
                    id: row.slope_id.clone(),
                    name: row.slope_name.clone(),
                    difficulty: row.slope_difficulty.clone(),
                    status: row.slope_status.clone(),
                    grooming: row.slope_grooming.clone(),
                    geometry: row.slope_geometry.clone().unwrap_or_default(),
                    websites: row.slope_websites.clone(),
                    sources: row.slope_sources.clone(),
                    places: Vec::new(),
                },
                resorts: Vec::new(),
            });

        slope.resorts.push(ResortRow {
            id: row.resort_id,
            name: row.resort_name.unwrap_or_default(),
            r#type: row.resort_type,
            status: row.resort_status,
            activities: row.resort_activities,
            geometry: row.resort_geometry.unwrap_or_default(),
            websites: row.resort_websites,
            sources: row.resort_sources,
            places: Vec::new(),
        });
    }

    populate_slope_places(pool, &mut slope_map).await?;

    let mut result: Vec<SlopeWithResorts> = slope_map.into_iter().map(|(_, v)| v).collect();
    result.sort_by(|a, b| a.slope.name.cmp(&b.slope.name));
    Ok(result)
}

pub async fn insert(
    pool: &MySqlPool,
    id: &str,
    name: Option<&str>,
    difficulty: Option<&str>,
    status: Option<&str>,
    grooming: Option<&str>,
    geometry_wkt: &str,
    websites: Option<&Value>,
    sources: Option<&Value>,
    resort_ids: &[String],
    places: &[Place],
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO slopes
            (id, name, difficulty, status, grooming, geometry, websites, sources)
        VALUES
            (?, ?, ?, ?, ?, ST_GeomFromText(?, 4326), ?, ?)
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(difficulty)
    .bind(status)
    .bind(grooming)
    .bind(geometry_wkt)
    .bind(websites)
    .bind(sources)
    .execute(pool)
    .await?;

    sync_slope_resorts(pool, id, resort_ids).await?;
    crate::db::places::sync_slope_places(pool, id, places).await?;
    Ok(())
}

pub async fn update(
    pool: &MySqlPool,
    id: &str,
    name: Option<&str>,
    difficulty: Option<&str>,
    status: Option<&str>,
    grooming: Option<&str>,
    geometry_wkt: &str,
    websites: Option<&Value>,
    sources: Option<&Value>,
    resort_ids: &[String],
    places: &[Place],
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE slopes SET
            name = ?,
            difficulty = ?,
            status = ?,
            grooming = ?,
            geometry = ST_GeomFromText(?, 4326),
            websites = ?,
            sources = ?
        WHERE id = ?
        "#,
    )
    .bind(name)
    .bind(difficulty)
    .bind(status)
    .bind(grooming)
    .bind(geometry_wkt)
    .bind(websites)
    .bind(sources)
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() > 0 {
        sync_slope_resorts(pool, id, resort_ids).await?;
        crate::db::places::sync_slope_places(pool, id, places).await?;
    }

    Ok(result.rows_affected())
}

pub async fn delete(pool: &MySqlPool, id: &str) -> Result<u64, sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM slope_resorts
        WHERE slope_id = ?
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    crate::db::places::delete_slope_places(pool, id).await?;

    let result = sqlx::query(
        r#"
        DELETE FROM slopes
        WHERE id = ?
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn delete_by_resort(pool: &MySqlPool, resort_id: &str) -> Result<u64, sqlx::Error> {
    let ids = sqlx::query_scalar!(
        r#"
        SELECT slope_id
        FROM slope_resorts
        WHERE resort_id = ?
        "#,
        resort_id
    )
    .fetch_all(pool)
    .await?;

    let mut deleted = 0;
    for id in ids {
        deleted += delete(pool, &id).await?;
    }
    Ok(deleted)
}

async fn sync_slope_resorts(
    pool: &MySqlPool,
    slope_id: &str,
    resort_ids: &[String],
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM slope_resorts
        WHERE slope_id = ?
        "#,
    )
    .bind(slope_id)
    .execute(pool)
    .await?;

    for resort_id in resort_ids {
        sqlx::query(
            r#"
            INSERT INTO slope_resorts (slope_id, resort_id)
            VALUES (?, ?)
            "#,
        )
        .bind(slope_id)
        .bind(resort_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn populate_slope_places(
    pool: &MySqlPool,
    slopes: &mut HashMap<String, SlopeWithResorts>,
) -> Result<(), sqlx::Error> {
    let slope_ids = slopes.keys().cloned().collect::<Vec<_>>();
    let slope_places = crate::db::places::get_for_slopes(pool, &slope_ids).await?;

    let resort_ids = slopes
        .values()
        .flat_map(|slope| slope.resorts.iter().map(|resort| resort.id.clone()))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let resort_places = crate::db::places::get_for_resorts(pool, &resort_ids).await?;

    for (slope_id, slope) in slopes.iter_mut() {
        slope.slope.places = slope_places.get(slope_id).cloned().unwrap_or_default();
        for resort in &mut slope.resorts {
            resort.places = resort_places.get(&resort.id).cloned().unwrap_or_default();
        }
    }

    Ok(())
}
