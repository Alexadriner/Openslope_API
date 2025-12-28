use serde::{Deserialize, Serialize};

#[derive(sqlx::FromRow, Debug, serde::Serialize)]
pub struct ResortRow {
    pub id: String,
    pub name: String,

    pub country: Option<String>,
    pub region: Option<String>,
    pub continent: Option<String>,

    pub latitude: Option<f64>,
    pub longitude: Option<f64>,

    pub village_altitude_m: Option<i32>,
    pub min_altitude_m: Option<i32>,
    pub max_altitude_m: Option<i32>,

    pub ski_area_name: Option<String>,
    pub ski_area_type: Option<String>,
}

#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct LiftRow {
    pub id: i64,
    pub resort_id: String,
    pub name: Option<String>,      // ← wichtig
    pub lift_type: Option<String>, // ← wichtig
}

#[derive(sqlx::FromRow, Debug, Serialize)]
pub struct SlopeRow {
    pub id: i64,
    pub resort_id: String,
    pub name: Option<String>,        // ← wichtig
    pub difficulty: Option<String>,  // ← wichtig
}