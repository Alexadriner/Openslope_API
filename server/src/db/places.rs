use crate::models::db::Place;
use sqlx::{FromRow, MySql, MySqlPool, QueryBuilder};
use std::collections::HashMap;

#[derive(FromRow)]
struct EntityPlaceRow {
    entity_id: String,
    id: i64,
    country_code: Option<String>,
    region_code: Option<String>,
    country_name: Option<String>,
    region_name: Option<String>,
    locality: Option<String>,
}

pub async fn get_for_resorts(
    pool: &MySqlPool,
    resort_ids: &[String],
) -> Result<HashMap<String, Vec<Place>>, sqlx::Error> {
    get_places_for_entities(pool, resort_ids, "resort_places", "resort_id").await
}

pub async fn get_for_lifts(
    pool: &MySqlPool,
    lift_ids: &[String],
) -> Result<HashMap<String, Vec<Place>>, sqlx::Error> {
    get_places_for_entities(pool, lift_ids, "lift_places", "lift_id").await
}

pub async fn get_for_slopes(
    pool: &MySqlPool,
    slope_ids: &[String],
) -> Result<HashMap<String, Vec<Place>>, sqlx::Error> {
    get_places_for_entities(pool, slope_ids, "slope_places", "slope_id").await
}

pub async fn sync_resort_places(
    pool: &MySqlPool,
    resort_id: &str,
    places: &[Place],
) -> Result<(), sqlx::Error> {
    sync_places_for_entity(pool, resort_id, places, "resort_places", "resort_id").await
}

pub async fn sync_lift_places(
    pool: &MySqlPool,
    lift_id: &str,
    places: &[Place],
) -> Result<(), sqlx::Error> {
    sync_places_for_entity(pool, lift_id, places, "lift_places", "lift_id").await
}

pub async fn sync_slope_places(
    pool: &MySqlPool,
    slope_id: &str,
    places: &[Place],
) -> Result<(), sqlx::Error> {
    sync_places_for_entity(pool, slope_id, places, "slope_places", "slope_id").await
}

pub async fn delete_resort_places(pool: &MySqlPool, resort_id: &str) -> Result<(), sqlx::Error> {
    delete_places_for_entity(pool, resort_id, "resort_places", "resort_id").await
}

pub async fn delete_lift_places(pool: &MySqlPool, lift_id: &str) -> Result<(), sqlx::Error> {
    delete_places_for_entity(pool, lift_id, "lift_places", "lift_id").await
}

pub async fn delete_slope_places(pool: &MySqlPool, slope_id: &str) -> Result<(), sqlx::Error> {
    delete_places_for_entity(pool, slope_id, "slope_places", "slope_id").await
}

async fn get_places_for_entities(
    pool: &MySqlPool,
    entity_ids: &[String],
    relation_table: &str,
    entity_column: &str,
) -> Result<HashMap<String, Vec<Place>>, sqlx::Error> {
    if entity_ids.is_empty() {
        return Ok(HashMap::new());
    }

    let mut builder = QueryBuilder::<MySql>::new(format!(
        "
        SELECT
            rel.{entity_column} as entity_id,
            p.id,
            p.country_code,
            p.region_code,
            p.country_name,
            p.region_name,
            p.locality
        FROM {relation_table} rel
        INNER JOIN places p ON p.id = rel.place_id
        WHERE rel.{entity_column} IN (
        "
    ));

    let mut separated = builder.separated(", ");
    for entity_id in entity_ids {
        separated.push_bind(entity_id);
    }
    separated.push_unseparated(
        "
        )
        ORDER BY p.country_name, p.region_name, p.locality, p.id
        ",
    );

    let rows = builder
        .build_query_as::<EntityPlaceRow>()
        .fetch_all(pool)
        .await?;

    let mut places_by_entity = HashMap::<String, Vec<Place>>::new();
    for row in rows {
        places_by_entity
            .entry(row.entity_id)
            .or_default()
            .push(Place {
                id: row.id,
                country_code: row.country_code,
                region_code: row.region_code,
                country_name: row.country_name,
                region_name: row.region_name,
                locality: row.locality,
            });
    }

    Ok(places_by_entity)
}

async fn sync_places_for_entity(
    pool: &MySqlPool,
    entity_id: &str,
    places: &[Place],
    relation_table: &str,
    entity_column: &str,
) -> Result<(), sqlx::Error> {
    delete_places_for_entity(pool, entity_id, relation_table, entity_column).await?;

    for place in places {
        let place_id = resolve_place_id(pool, place).await?;

        sqlx::query(&format!(
            "
            INSERT INTO {relation_table} ({entity_column}, place_id)
            VALUES (?, ?)
            "
        ))
        .bind(entity_id)
        .bind(place_id)
        .execute(pool)
        .await?;
    }

    Ok(())
}

async fn delete_places_for_entity(
    pool: &MySqlPool,
    entity_id: &str,
    relation_table: &str,
    entity_column: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(&format!(
        "
        DELETE FROM {relation_table}
        WHERE {entity_column} = ?
        "
    ))
    .bind(entity_id)
    .execute(pool)
    .await?;

    Ok(())
}

async fn resolve_place_id(pool: &MySqlPool, place: &Place) -> Result<i64, sqlx::Error> {
    if let Some(existing) = sqlx::query_as::<_, Place>(
        r#"
        SELECT
            id,
            country_code,
            region_code,
            country_name,
            region_name,
            locality
        FROM places
        WHERE country_code <=> ?
          AND region_code <=> ?
          AND locality <=> ?
        LIMIT 1
        "#,
    )
    .bind(&place.country_code)
    .bind(&place.region_code)
    .bind(&place.locality)
    .fetch_optional(pool)
    .await?
    {
        return Ok(existing.id);
    }

    let result = sqlx::query(
        r#"
        INSERT INTO places
            (country_code, region_code, country_name, region_name, locality)
        VALUES
            (?, ?, ?, ?, ?)
        "#,
    )
    .bind(&place.country_code)
    .bind(&place.region_code)
    .bind(&place.country_name)
    .bind(&place.region_name)
    .bind(&place.locality)
    .execute(pool)
    .await?;

    Ok(result.last_insert_id() as i64)
}
