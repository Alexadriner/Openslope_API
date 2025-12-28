use serde::Serialize;

#[derive(Serialize)]
pub struct ResortResponse {
    pub id: String,
    pub name: String,
    pub location: LocationBlock,
    pub altitude: AltitudeBlock,
    pub ski_area: SkiAreaBlock,
    pub lifts: Vec<LiftResponse>,
    pub slopes: Vec<SlopeResponse>,
}

#[derive(Serialize)]
pub struct LocationBlock {
    pub country: String,
    pub region: String,
    pub continent: String,
    pub latitude: f64,
    pub longitude: f64,
}

#[derive(Serialize)]
pub struct AltitudeBlock {
    pub village_altitude_m: i32,
    pub min_altitude_m: i32,
    pub max_altitude_m: i32,
}

#[derive(Serialize)]
pub struct SkiAreaBlock {
    pub name: String,
    pub area_type: String,
    pub total_slope_km: Option<f64>,
    pub total_lifts: Option<i32>,
    pub snowmaking_percent: Option<i32>,
    pub night_skiing: Option<bool>,
}

#[derive(Serialize)]
pub struct LiftResponse {
    pub id: String,
    pub name: String,
    pub lift_type: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct SlopeResponse {
    pub id: String,
    pub name: String,
    pub difficulty: String,
    pub length_km: f64,
}