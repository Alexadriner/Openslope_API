use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct Place {
    pub id: i64,
    pub country_code: Option<String>,
    pub region_code: Option<String>,
    pub country_name: Option<String>,
    pub region_name: Option<String>,
    pub locality: Option<String>,
}

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct ResortRow {
    pub id: String,
    pub name: String,
    #[sqlx(rename = "type")]
    pub r#type: Option<String>,
    pub status: Option<String>,
    pub activities: Option<Value>,
    pub geometry: Value,
    pub websites: Option<Value>,
    pub sources: Option<Value>,
    #[sqlx(default)]
    pub places: Vec<Place>,
}

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct LiftRow {
    pub id: String,
    pub name: Option<String>,
    #[sqlx(rename = "type")]
    pub r#type: Option<String>,
    pub status: Option<String>,
    pub capacity: Option<i32>,
    pub duration: Option<i32>,
    pub geometry: Value,
    pub websites: Option<Value>,
    pub sources: Option<Value>,
    #[sqlx(default)]
    pub places: Vec<Place>,
}

#[derive(sqlx::FromRow, Debug, Clone, Serialize, Deserialize)]
pub struct SlopeRow {
    pub id: String,
    pub name: Option<String>,
    pub difficulty: Option<String>,
    pub status: Option<String>,
    pub grooming: Option<String>,
    pub geometry: Value,
    pub websites: Option<Value>,
    pub sources: Option<Value>,
    #[sqlx(default)]
    pub places: Vec<Place>,
}

#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct ElevationProfileRow {
    pub id: String,
    pub entity_type: String,
    pub entity_id: String,
    pub heights: Option<Value>,
    pub resolution: Option<f64>,
    pub target_resolution: Option<f64>,
}
