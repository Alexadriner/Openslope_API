use actix_web::{web, HttpResponse};
use sqlx::MySqlPool;

use crate::db::resort_queries::get_resort_full;
use crate::models::resort::*;

pub async fn get_resort(
    pool: web::Data<MySqlPool>,
    path: web::Path<String>,
) -> Result<HttpResponse, actix_web::Error> {

    let resort_id = path.into_inner();

    let (resort, lifts, slopes) =
        get_resort_full(&pool, &resort_id)
            .await
            .map_err(|_| actix_web::error::ErrorNotFound("Resort not found"))?;

    let response = serde_json::json!({
        "resort": {
            "id": resort.id,
            "name": resort.name,
            "location": {
                "country": resort.country,
                "region": resort.region,
                "continent": resort.continent,
                "latitude": resort.latitude,
                "longitude": resort.longitude
            },
            "altitude": {
                "village_m": resort.village_altitude_m,
                "min_m": resort.min_altitude_m,
                "max_m": resort.max_altitude_m
            },
            "ski_area": {
                "name": resort.ski_area_name,
                "type": resort.ski_area_type
            }
        },
        "lifts": lifts,
        "slopes": slopes
    });

    Ok(HttpResponse::Ok().json(response))
}