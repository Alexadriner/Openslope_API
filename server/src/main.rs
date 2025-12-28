use actix_web::{web, App, HttpServer};
use sqlx::MySqlPool;
use dotenvy::dotenv;
use std::env;

mod auth;
mod routes;
mod security;
mod user_service;
use user_service::create_user;
mod models;
mod db;

use auth::ApiKeyAuth;
use routes::resorts::*;

use routes::auth::{signup, signin};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let database_url =
        env::var("DATABASE_URL").expect("DATABASE_URL not set");

    let pool = MySqlPool::connect(&database_url)
        .await
        .expect("DB connection failed");

    println!("Server läuft auf Port 8080");

    HttpServer::new(move || {
    App::new()
        .app_data(web::Data::new(pool.clone()))

        // Öffentliche Routen – garantiert ohne Auth
        .route("/signup", web::post().to(signup))
        .route("/signin", web::post().to(signin))

        // Geschützte Routen – explizit mit Auth
        .service(
            web::scope("/resorts")
                .wrap(ApiKeyAuth { pool: pool.clone() })
                .route("/{id}", web::get().to(get_resort))
        )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}