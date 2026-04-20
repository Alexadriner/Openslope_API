use crate::models::db::{LiftRow, Place, ResortRow};
use serde_json::Value;
use sqlx::MySqlPool;
use std::collections::HashMap;

pub struct LiftWithResorts {
    pub lift: LiftRow,
    pub resorts: Vec<ResortRow>,
}

pub async fn get_all(pool: &MySqlPool) -> Result<Vec<LiftWithResorts>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            l.id as lift_id,
            l.name as lift_name,
            l.type as lift_type,
            l.status as lift_status,
            l.capacity as lift_capacity,
            l.duration as lift_duration,
            ST_AsGeoJSON(l.geometry) as lift_geometry,
            l.websites as lift_websites,
            l.sources as lift_sources,
            r.id as resort_id,
            r.name as resort_name,
            r.type as resort_type,
            r.status as resort_status,
            r.activities as resort_activities,
            ST_AsGeoJSON(r.geometry) as resort_geometry,
            r.websites as resort_websites,
            r.sources as resort_sources
        FROM lifts l
        LEFT JOIN lift_resorts lr ON l.id = lr.lift_id
        LEFT JOIN resorts r ON lr.resort_id = r.id
        ORDER BY l.name, r.name
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut lift_map: HashMap<String, LiftWithResorts> = HashMap::new();

    for row in rows {
        let lift_id = row.lift_id.clone();
        let lift = lift_map
            .entry(lift_id.clone())
            .or_insert_with(|| LiftWithResorts {
                lift: LiftRow {
                    id: row.lift_id.clone(),
                    name: row.lift_name.clone(),
                    r#type: row.lift_type.clone(),
                    status: row.lift_status.clone(),
                    capacity: row.lift_capacity,
                    duration: row.lift_duration,
                    geometry: row.lift_geometry.clone().unwrap_or_default(),
                    websites: row.lift_websites.clone(),
                    sources: row.lift_sources.clone(),
                    places: Vec::new(),
                },
                resorts: Vec::new(),
            });

        if let Some(resort_id) = row.resort_id {
            lift.resorts.push(ResortRow {
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

    populate_lift_places(pool, &mut lift_map).await?;

    let mut result: Vec<LiftWithResorts> = lift_map.into_iter().map(|(_, v)| v).collect();
    result.sort_by(|a, b| a.lift.name.cmp(&b.lift.name));
    Ok(result)
}

pub async fn get_by_id(
    pool: &MySqlPool,
    lift_id: &str,
) -> Result<Option<LiftWithResorts>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            l.id as lift_id,
            l.name as lift_name,
            l.type as lift_type,
            l.status as lift_status,
            l.capacity as lift_capacity,
            l.duration as lift_duration,
            ST_AsGeoJSON(l.geometry) as lift_geometry,
            l.websites as lift_websites,
            l.sources as lift_sources,
            r.id as resort_id,
            r.name as resort_name,
            r.type as resort_type,
            r.status as resort_status,
            r.activities as resort_activities,
            ST_AsGeoJSON(r.geometry) as resort_geometry,
            r.websites as resort_websites,
            r.sources as resort_sources
        FROM lifts l
        LEFT JOIN lift_resorts lr ON l.id = lr.lift_id
        LEFT JOIN resorts r ON lr.resort_id = r.id
        WHERE l.id = ?
        ORDER BY r.name
        "#,
        lift_id
    )
    .fetch_all(pool)
    .await?;

    if rows.is_empty() {
        return Ok(None);
    }

    let mut resorts = Vec::new();
    let lift_row = &rows[0];

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

    let lift = LiftWithResorts {
        lift: LiftRow {
            id: lift_row.lift_id.clone(),
            name: lift_row.lift_name.clone(),
            r#type: lift_row.lift_type.clone(),
            status: lift_row.lift_status.clone(),
            capacity: lift_row.lift_capacity,
            duration: lift_row.lift_duration,
            geometry: lift_row.lift_geometry.clone().unwrap_or_default(),
            websites: lift_row.lift_websites.clone(),
            sources: lift_row.lift_sources.clone(),
            places: Vec::new(),
        },
        resorts,
    };

    let mut lift_map = HashMap::from([(lift.lift.id.clone(), lift)]);
    populate_lift_places(pool, &mut lift_map).await?;

    Ok(lift_map.remove(lift_id))
}

pub async fn get_by_resort(
    pool: &MySqlPool,
    resort_id: &str,
) -> Result<Vec<LiftWithResorts>, sqlx::Error> {
    let rows = sqlx::query!(
        r#"
        SELECT
            l.id as lift_id,
            l.name as lift_name,
            l.type as lift_type,
            l.status as lift_status,
            l.capacity as lift_capacity,
            l.duration as lift_duration,
            ST_AsGeoJSON(l.geometry) as lift_geometry,
            l.websites as lift_websites,
            l.sources as lift_sources,
            r.id as resort_id,
            r.name as resort_name,
            r.type as resort_type,
            r.status as resort_status,
            r.activities as resort_activities,
            ST_AsGeoJSON(r.geometry) as resort_geometry,
            r.websites as resort_websites,
            r.sources as resort_sources
        FROM lifts l
        INNER JOIN lift_resorts lr ON lr.lift_id = l.id
        INNER JOIN resorts r ON lr.resort_id = r.id
        WHERE lr.resort_id = ?
        ORDER BY l.name
        "#,
        resort_id
    )
    .fetch_all(pool)
    .await?;

    let mut lift_map: HashMap<String, LiftWithResorts> = HashMap::new();

    for row in rows {
        let lift_id = row.lift_id.clone();
        let lift = lift_map
            .entry(lift_id.clone())
            .or_insert_with(|| LiftWithResorts {
                lift: LiftRow {
                    id: row.lift_id.clone(),
                    name: row.lift_name.clone(),
                    r#type: row.lift_type.clone(),
                    status: row.lift_status.clone(),
                    capacity: row.lift_capacity,
                    duration: row.lift_duration,
                    geometry: row.lift_geometry.clone().unwrap_or_default(),
                    websites: row.lift_websites.clone(),
                    sources: row.lift_sources.clone(),
                    places: Vec::new(),
                },
                resorts: Vec::new(),
            });

        lift.resorts.push(ResortRow {
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

    populate_lift_places(pool, &mut lift_map).await?;

    let mut result: Vec<LiftWithResorts> = lift_map.into_iter().map(|(_, v)| v).collect();
    result.sort_by(|a, b| a.lift.name.cmp(&b.lift.name));
    Ok(result)
}

pub async fn insert(
    pool: &MySqlPool,
    id: &str,
    name: Option<&str>,
    lift_type: Option<&str>,
    status: Option<&str>,
    capacity: Option<i32>,
    duration: Option<i32>,
    geometry_wkt: &str,
    websites: Option<&Value>,
    sources: Option<&Value>,
    resort_ids: &[String],
    places: &[Place],
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO lifts
            (id, name, `type`, status, capacity, duration, geometry, websites, sources)
        VALUES
            (?, ?, ?, ?, ?, ?, ST_GeomFromText(?, 4326), ?, ?)
        "#,
    )
    .bind(id)
    .bind(name)
    .bind(lift_type)
    .bind(status)
    .bind(capacity)
    .bind(duration)
    .bind(geometry_wkt)
    .bind(websites)
    .bind(sources)
    .execute(pool)
    .await?;

    sync_lift_resorts(pool, id, resort_ids).await?;
    crate::db::places::sync_lift_places(pool, id, places).await?;
    Ok(())
}

pub async fn update(
    pool: &MySqlPool,
    id: &str,
    name: Option<&str>,
    lift_type: Option<&str>,
    status: Option<&str>,
    capacity: Option<i32>,
    duration: Option<i32>,
    geometry_wkt: &str,
    websites: Option<&Value>,
    sources: Option<&Value>,
    resort_ids: &[String],
    places: &[Place],
) -> Result<u64, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE lifts SET
            name = ?,
            `type` = ?,
            status = ?,
            capacity = ?,
            duration = ?,
            geometry = ST_GeomFromText(?, 4326),
            websites = ?,
            sources = ?
        WHERE id = ?
        "#,
    )
    .bind(name)
    .bind(lift_type)
    .bind(status)
    .bind(capacity)
    .bind(duration)
    .bind(geometry_wkt)
    .bind(websites)
    .bind(sources)
    .bind(id)
    .execute(pool)
    .await?;

    if result.rows_affected() > 0 {
        sync_lift_resorts(pool, id, resort_ids).await?;
        crate::db::places::sync_lift_places(pool, id, places).await?;
    }

    Ok(result.rows_affected())
}

pub async fn delete(pool: &MySqlPool, id: &str) -> Result<u64, sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM lift_resorts
        WHERE lift_id = ?
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    crate::db::places::delete_lift_places(pool, id).await?;

    let result = sqlx::query(
        r#"
        DELETE FROM lifts
        WHERE id = ?
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

pub async fn delete_by_resort(pool: &MySqlPool, resort_id: &str) -> Result<u64, sqlx::Error> {
    let mates = sqlx::query!(
        r#"
        SELECT lift_id
        FROM lift_resorts
        WHERE resort_id = ?
        "#,
        resort_id
    )
    .fetch_all(pool)
    .await?;

    let mut deleted = 0;
    for row in mates {
        deleted += delete(pool, &row.lift_id).await?;
    }
    Ok(deleted)
}

async fn sync_lift_resorts(
    pool: &MySqlPool,
    lift_id: &str,
    resort_ids: &[String],
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        DELETE FROM lift_resorts
        WHERE lift_id = ?
        "#,
    )
    .bind(lift_id)
    .execute(pool)
    .await?;

    for resort_id in resort_ids {
        sqlx::query(
            r#"
            INSERT INTO lift_resorts (lift_id, resort_id)
            VALUES (?, ?)
            "#,
        )
        .bind(lift_id)
        .bind(resort_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn populate_lift_places(
    pool: &MySqlPool,
    lifts: &mut HashMap<String, LiftWithResorts>,
) -> Result<(), sqlx::Error> {
    let lift_ids = lifts.keys().cloned().collect::<Vec<_>>();
    let lift_places = crate::db::places::get_for_lifts(pool, &lift_ids).await?;

    let resort_ids = lifts
        .values()
        .flat_map(|lift| lift.resorts.iter().map(|resort| resort.id.clone()))
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    let resort_places = crate::db::places::get_for_resorts(pool, &resort_ids).await?;

    for (lift_id, lift) in lifts.iter_mut() {
        lift.lift.places = lift_places.get(lift_id).cloned().unwrap_or_default();
        for resort in &mut lift.resorts {
            resort.places = resort_places.get(&resort.id).cloned().unwrap_or_default();
        }
    }

    Ok(())
}
