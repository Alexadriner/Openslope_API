use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct ResortRow {
    pub id: String,
    pub name: Option<String>,
    #[sqlx(rename = "type")]
    pub r#type: Option<String>,
    pub status: Option<String>,
    pub places: Option<Value>,
    pub sources: Option<Value>,
    pub websites: Option<Value>,
    pub activities: Option<Value>,
    pub statistics: Option<Value>,
    pub wikidata_id: Option<String>,
    pub viewport_hint: Option<Value>,
    pub run_convention: Option<String>,
    pub geometry_type: Option<String>,
    pub geometry: Option<Value>,
    pub properties: Option<Value>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct GeojsonLiftRow {
    pub id: String,
    #[sqlx(rename = "ref")]
    pub ref_: Option<String>,
    pub name: Option<String>,
    #[sqlx(rename = "type")]
    pub r#type: Option<String>,
    pub access: Option<String>,
    pub bubble: Option<bool>,
    pub oneway: Option<bool>,
    pub status: Option<String>,
    pub tunnel: Option<bool>,
    pub heating: Option<String>,
    pub capacity: Option<i32>,
    pub duration: Option<f64>,
    pub lift_type: Option<String>,
    pub occupancy: Option<Value>,
    pub detachable: Option<bool>,
    pub ref_frcairn: Option<String>,
    pub wikidata_id: Option<String>,
    pub description: Option<String>,
    pub viewport_hint: Option<Value>,
    pub places: Option<Value>,
    pub sources: Option<Value>,
    pub ski_areas: Option<Value>,
    pub stations: Option<Value>,
    pub websites: Option<Value>,
    pub geometry_type: Option<String>,
    pub geometry: Option<Value>,
    pub properties: Option<Value>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct GeojsonRunRow {
    pub id: String,
    pub lit: Option<bool>,
    #[sqlx(rename = "ref")]
    pub ref_: Option<String>,
    pub name: Option<String>,
    #[sqlx(rename = "type")]
    pub r#type: Option<String>,
    pub uses: Option<Value>,
    pub gladed: Option<bool>,
    pub oneway: Option<bool>,
    pub places: Option<Value>,
    pub status: Option<String>,
    pub tunnel: Option<bool>,
    pub sources: Option<Value>,
    pub grooming: Option<String>,
    pub ski_areas: Option<Value>,
    pub websites: Option<Value>,
    pub patrolled: Option<bool>,
    pub difficulty: Option<String>,
    pub snowmaking: Option<bool>,
    pub wikidata_id: Option<String>,
    pub description: Option<String>,
    pub snowfarming: Option<bool>,
    pub viewport_hint: Option<Value>,
    pub elevation_profile: Option<Value>,
    pub difficulty_convention: Option<String>,
    pub geometry_type: Option<String>,
    pub geometry: Option<Value>,
    pub properties: Option<Value>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct GeojsonSkiAreaRow {
    pub id: String,
    pub name: Option<String>,
    #[sqlx(rename = "type")]
    pub r#type: Option<String>,
    pub status: Option<String>,
    pub places: Option<Value>,
    pub sources: Option<Value>,
    pub websites: Option<Value>,
    pub activities: Option<Value>,
    pub statistics: Option<Value>,
    pub wikidata_id: Option<String>,
    pub viewport_hint: Option<Value>,
    pub run_convention: Option<String>,
    pub geometry_type: Option<String>,
    pub geometry: Option<Value>,
    pub properties: Option<Value>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
