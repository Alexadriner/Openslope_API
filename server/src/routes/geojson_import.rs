use actix_web::{web, HttpResponse};
use serde_json::Value;

use crate::{error::AppError, routes::{lifts, resorts, slopes}};

pub async fn import_geojson(
    db: web::Data<sqlx::MySqlPool>,
    resource: web::Path<String>,
    feature: web::Json<Value>,
) -> Result<HttpResponse, AppError> {
    let resource = resource.into_inner();
    let feature = feature.into_inner();

    let handler_response: Result<HttpResponse, AppError> = match resource.as_str() {
        "lifts" => lifts::create_lift(db, web::Json(serde_json::from_value(feature)?)).await,
        "slopes" => slopes::create_slope(db, web::Json(serde_json::from_value(feature)?)).await,
        "resorts" => resorts::create_resort(db, web::Json(serde_json::from_value(feature)?)).await,
        _ => return Err(AppError::BadRequest("Unknown geojson resource".into())),
    };

    handler_response
}
