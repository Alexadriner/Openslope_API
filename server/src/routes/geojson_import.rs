use actix_web::{web, HttpResponse, Responder};
use serde_json::Value;
use sqlx::MySqlPool;

fn extract_string(value: Option<&Value>) -> Option<String> {
    value.and_then(|inner| match inner {
        Value::String(s) => Some(s.clone()),
        Value::Number(n) => Some(n.to_string()),
        Value::Bool(b) => Some(b.to_string()),
        _ => None,
    })
}

fn json_field(properties: &Value, key: &str) -> Option<Value> {
    properties.get(key).cloned()
}

fn feature_id(feature: &Value) -> Option<String> {
    if let Some(id) = extract_string(feature.get("id")) {
        return Some(id);
    }
    feature
        .get("properties")
        .and_then(|props| extract_string(props.get("id")))
}

fn geometry_type(feature: &Value) -> Option<String> {
    feature
        .get("geometry")
        .and_then(|geom| geom.get("type"))
        .and_then(|value| extract_string(Some(value)))
}

/// Extract the resort reference ID from a feature's properties.
/// Looks for the first ski area ID in the skiAreas array.
fn extract_resort_ref(properties: &Value) -> Option<String> {
    properties
        .get("skiAreas")
        .and_then(|ski_areas| ski_areas.as_array())
        .and_then(|arr| arr.first())
        .and_then(|first| first.get("properties"))
        .and_then(|props| extract_string(props.get("id")))
}

async fn insert_geojson_lift(db: &MySqlPool, feature: &Value) -> Result<(), sqlx::Error> {
    let properties = feature.get("properties").cloned().unwrap_or(Value::Null);
    let geometry = feature.get("geometry").cloned().unwrap_or(Value::Null);

    // Extract resort reference from skiAreas
    let resort_ref = extract_resort_ref(&properties);

    sqlx::query(
        "INSERT INTO lifts
         (id, ref, name, `type`, status, places, sources, ski_areas, stations, websites, geometry_type, geometry, properties)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON DUPLICATE KEY UPDATE
         ref = VALUES(ref),
         name = VALUES(name),
         `type` = VALUES(`type`),
         status = VALUES(status),
         places = VALUES(places),
         sources = VALUES(sources),
         ski_areas = VALUES(ski_areas),
         stations = VALUES(stations),
         websites = VALUES(websites),
         geometry_type = VALUES(geometry_type),
         geometry = VALUES(geometry),
         properties = VALUES(properties)"
    )
    .bind(feature_id(feature).unwrap())
    .bind(resort_ref)
    .bind(extract_string(properties.get("name")))
    .bind(extract_string(properties.get("type")))
    .bind(extract_string(properties.get("status")))
    .bind(json_field(&properties, "places"))
    .bind(json_field(&properties, "sources"))
    .bind(json_field(&properties, "skiAreas"))
    .bind(json_field(&properties, "stations"))
    .bind(json_field(&properties, "websites"))
    .bind(geometry_type(feature))
    .bind(geometry)
    .bind(json_field(&properties, "properties"))
    .execute(db)
    .await
    .map(|_| ())
}

async fn insert_geojson_slope(db: &MySqlPool, feature: &Value) -> Result<(), sqlx::Error> {
    let properties = feature.get("properties").cloned().unwrap_or(Value::Null);
    let geometry = feature.get("geometry").cloned().unwrap_or(Value::Null);

    // Extract resort reference from skiAreas
    let resort_ref = extract_resort_ref(&properties);

    sqlx::query(
        "INSERT INTO slopes
         (id, ref, name, `type`, status, places, sources, ski_areas, websites, difficulty, grooming, difficulty_convention, geometry_type, geometry, properties)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON DUPLICATE KEY UPDATE
         ref = VALUES(ref),
         name = VALUES(name),
         `type` = VALUES(`type`),
         status = VALUES(status),
         places = VALUES(places),
         sources = VALUES(sources),
         ski_areas = VALUES(ski_areas),
         websites = VALUES(websites),
         difficulty = VALUES(difficulty),
         grooming = VALUES(grooming),
         difficulty_convention = VALUES(difficulty_convention),
         geometry_type = VALUES(geometry_type),
         geometry = VALUES(geometry),
         properties = VALUES(properties)"
    )
    .bind(feature_id(feature).unwrap())
    .bind(resort_ref)
    .bind(extract_string(properties.get("name")))
    .bind(extract_string(properties.get("type")))
    .bind(extract_string(properties.get("status")))
    .bind(json_field(&properties, "places"))
    .bind(json_field(&properties, "sources"))
    .bind(json_field(&properties, "skiAreas"))
    .bind(json_field(&properties, "websites"))
    .bind(extract_string(properties.get("difficulty")))
    .bind(extract_string(properties.get("grooming")))
    .bind(extract_string(properties.get("difficultyConvention")))
    .bind(geometry_type(feature))
    .bind(geometry)
    .bind(properties)
    .execute(db)
    .await
    .map(|_| ())
}

async fn insert_geojson_resort(db: &MySqlPool, feature: &Value) -> Result<(), sqlx::Error> {
    let properties = feature.get("properties").cloned().unwrap_or(Value::Null);
    let geometry = feature.get("geometry").cloned().unwrap_or(Value::Null);

    sqlx::query(
        "INSERT INTO resorts
         (id, name, `type`, status, places, sources, websites, activities, statistics, wikidata_id, viewport_hint, run_convention, geometry_type, geometry, properties)
         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
         ON DUPLICATE KEY UPDATE
         name = VALUES(name),
         `type` = VALUES(`type`),
         status = VALUES(status),
         places = VALUES(places),
         sources = VALUES(sources),
         websites = VALUES(websites),
         activities = VALUES(activities),
         statistics = VALUES(statistics),
         wikidata_id = VALUES(wikidata_id),
         viewport_hint = VALUES(viewport_hint),
         run_convention = VALUES(run_convention),
         geometry_type = VALUES(geometry_type),
         geometry = VALUES(geometry),
         properties = VALUES(properties)"
    )
    .bind(feature_id(feature).unwrap())
    .bind(extract_string(properties.get("name")))
    .bind(extract_string(properties.get("type")))
    .bind(extract_string(properties.get("status")))
    .bind(json_field(&properties, "places"))
    .bind(json_field(&properties, "sources"))
    .bind(json_field(&properties, "websites"))
    .bind(json_field(&properties, "activities"))
    .bind(json_field(&properties, "statistics"))
    .bind(extract_string(properties.get("wikidata_id")))
    .bind(json_field(&properties, "viewport_hint"))
    .bind(extract_string(properties.get("run_convention")))
    .bind(geometry_type(feature))
    .bind(geometry)
    .bind(properties)
    .execute(db)
    .await
    .map(|_| ())
}

pub async fn import_geojson(
    db: web::Data<MySqlPool>,
    resource: web::Path<String>,
    feature: web::Json<Value>,
) -> impl Responder {
    let resource = resource.into_inner();
    let feature = feature.into_inner();

    let id = match feature_id(&feature) {
        Some(id) => id,
        None => {
            return HttpResponse::BadRequest().body("Missing feature id in geojson payload");
        }
    };

    let result = match resource.as_str() {
        "lifts" => insert_geojson_lift(db.get_ref(), &feature).await,
        "slopes" => insert_geojson_slope(db.get_ref(), &feature).await,
        "resorts" => insert_geojson_resort(db.get_ref(), &feature).await,
        _ => return HttpResponse::BadRequest().body("Unknown geojson import resource"),
    };

    match result {
        Ok(_) => HttpResponse::Created().json(serde_json::json!({"id": id, "resource": resource})),
        Err(err) => {
            eprintln!("GeoJSON import failure for {}: {:?}", resource, err);
            HttpResponse::InternalServerError().body("Failed to import geojson feature")
        }
    }
}
