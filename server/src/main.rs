//! OpenSlope API Server
//!
//! This is the main entry point for the OpenSlope API server.
//! The server provides RESTful endpoints for managing ski resort data,
//! including resorts, slopes, lifts, and user authentication.
//!
//! The server is built using Actix Web and uses MySQL as the database backend.
//! It includes features like CORS support, API key authentication, and
//! comprehensive route handling for all entities.
//!
//! # Architecture
//!
//! The server follows a modular architecture with the following key components:
//! - **Routes**: HTTP endpoint handlers organized by entity type
//! - **Models**: Database schema definitions and data structures
//! - **Security**: Authentication and authorization middleware
//! - **Database**: MySQL connection pool and query utilities
//!
//! # Usage
//!
//! ```bash
//! # Start the server
//! cargo run
//!
//! # Server will be available at http://127.0.0.1:8080
//! ```
//!
//! # Environment Variables
//!
//! - `DATABASE_URL`: MySQL connection string (default: mysql://username:password@Central.local:3306/openslope_db)
//!
//! # Endpoints
//!
//! ## Public Endpoints (No Authentication Required)
//! - `POST /signup` - User registration
//! - `POST /signin` - User login
//!
//! ## Protected Endpoints (API Key Required)
//! - `GET /me` - Get current user info
//! - `GET /resorts` - Get all resorts
//! - `GET /resorts/{id}` - Get specific resort
//! - `POST /resorts` - Create new resort
//! - `PUT /resorts/{id}` - Update resort
//! - `DELETE /resorts/{id}` - Delete resort
//! - `GET /slopes` - Get all slopes
//! - `GET /slopes/{id}` - Get specific slope
//! - `POST /slopes` - Create new slope
//! - `PUT /slopes/{id}` - Update slope
//! - `DELETE /slopes/{id}` - Delete slope
//! - `GET /lifts` - Get all lifts
//! - `GET /lifts/{id}` - Get specific lift
//! - `POST /lifts` - Create new lift
//! - `PUT /lifts/{id}` - Update lift
//! - `DELETE /lifts/{id}` - Delete lift
//!
//! # Dependencies
//!
//! - `actix-web`: Web framework
//! - `sqlx`: Async SQL toolkit
//! - `dotenvy`: Environment variable loading
//! - `actix-cors`: CORS middleware
//!
//! Author: OpenSlope Team
//! Version: 1.0.0

use actix_web::{web, App, HttpServer};
use sqlx::MySqlPool;
use dotenvy::dotenv;
use std::env;

mod auth;
mod routes;
mod security;
mod user_service;

use auth::ApiKeyAuth;
use routes::resorts::*;
use routes::slopes::*;
use routes::lifts::*;
use routes::status::*;

use routes::auth::{signup, signin, me};

use actix_cors::Cors;
use actix_web::web::JsonConfig;

/// Main application entry point
///
/// This function:
/// 1. Loads environment variables from .env file
/// 2. Establishes database connection
/// 3. Configures and starts the HTTP server
///
/// # Panics
///
/// This function will panic if:
/// - Database connection fails
/// - Server binding fails
///
/// # Example
///
/// ```rust
/// // This is the main function, called automatically when the binary runs
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     // Server setup and startup logic
/// }
/// ```
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Load environment variables from .env file
    dotenv().ok();

    // Get database URL from environment or use default
    let database_url = env::var("DATABASE_URL")
        .unwrap_or_else(|_| "mysql://username:password@Central.local:3306/openslope_db".to_string());

    // Establish database connection pool
    let pool = MySqlPool::connect(&database_url)
        .await
        .expect("DB connection failed");

    // Log server startup
    println!("Server läuft auf Port 8080");

    // Configure and start HTTP server
    HttpServer::new(move || {
        App::new()
            // Configure CORS middleware for cross-origin requests
            .wrap(
                Cors::default()
                    .allow_any_origin()   // erlaubt Zugriff von jedem Frontend (für Entwicklung)
                    .allow_any_method()   // GET, POST, PUT, DELETE etc.
                    .allow_any_header()   // alle Header erlaubt
                    .supports_credentials() // erlaubt Cookies und Credentials
            )
            // Share database pool across all handlers
            .app_data(web::Data::new(pool.clone()))

            // Public routes - no authentication required
            .route("/signup", web::post().to(signup))
            .route("/signin", web::post().to(signin))

            // Protected routes - require API key authentication
            .service(
                web::scope("")
                    .wrap(ApiKeyAuth { pool: pool.clone() })

                    // Resorts endpoints
                    .route("/resorts", web::get().to(get_resorts))
                    .route("/resorts/{id}", web::get().to(get_resort))
                    .route("/resorts", web::post().to(create_resort))
                    .route("/resorts/{id}", web::put().to(update_resort))
                    .route("/resorts/{id}", web::delete().to(delete_resort))

                    // Slopes endpoints
                    .route("/slopes", web::get().to(get_slopes))
                    .route("/slopes/{id}", web::get().to(get_slope))
                    .route("/slopes", web::post().to(create_slope))
                    .route("/slopes/{id}", web::put().to(update_slope))
                    .route("/slopes/{id}", web::delete().to(delete_slope))
                    .route("/slopes/by_resort/{resort_id}", web::get().to(get_slopes_by_resort))
                    .route("/slopes/by_resort/{resort_id}", web::delete().to(delete_slopes_by_resort))

                    // Lifts endpoints
                    .route("/lifts", web::get().to(get_lifts))
                    .route("/lifts/{id}", web::get().to(get_lift))
                    .route("/lifts", web::post().to(create_lift))
                    .route("/lifts/{id}", web::put().to(update_lift))
                    .route("/lifts/{id}", web::delete().to(delete_lift))
                    .route("/lifts/by_resort/{resort_id}", web::get().to(get_lifts_by_resort))
                    .route("/lifts/by_resort/{resort_id}", web::delete().to(delete_lifts_by_resort))

                    // Status and scraping endpoints
                    .route("/scrape-runs", web::get().to(get_scrape_runs))
                    .route("/scrape-runs/{id}", web::get().to(get_scrape_run))
                    .route("/status-snapshots", web::get().to(get_status_snapshots))
                    .route("/resorts/{resort_id}/status-snapshots", web::get().to(get_status_snapshots_by_resort))

                    // User endpoints
                    .route("/me", web::get().to(me))
            )
    })
    // Bind server to localhost on port 8080
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}