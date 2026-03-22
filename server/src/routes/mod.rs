//! OpenSlope API Routes Module
//!
//! This module serves as the central registry for all HTTP route handlers in the
//! OpenSlope API. It provides a clean interface to access the different route
//! handlers that handle requests for resorts, lifts, slopes, status information,
//! and authentication.
//!
//! # Route Organization
//!
//! The routes module is organized into several submodules, each handling a specific
//! domain of the API:
//!
//! - **resorts**: Complete ski resort management (CRUD operations)
//! - **lifts**: Individual lift information and management
//! - **slopes**: Individual slope information and management
//! - **status**: Scraping status and operational data
//! - **auth**: User authentication and API key management
//!
//! # Architecture Overview
//!
//! The API follows a RESTful architecture with the following patterns:
//!
//! - **Resource-based URLs**: `/resorts`, `/lifts`, `/slopes`, `/status`
//! - **HTTP Methods**: GET (read), POST (create), PUT (update), DELETE (delete)
//! - **Query Parameters**: Filtering, pagination, and optional data
//! - **JSON Responses**: All responses are JSON-formatted
//! - **Error Handling**: Consistent error responses with appropriate HTTP status codes
//!
//! # Route Examples
//!
//! ```rust
//! // Register routes in main.rs
//! App::new()
//!     .service(web::scope("/api/v1")
//!         .configure(routes::config))
//! ```
//!
//! ```rust
//! // Route configuration function
//! pub fn config(cfg: &mut web::ServiceConfig) {
//!     cfg.service(
//!         web::scope("/resorts")
//!             .route("", web::get().to(resorts::get_resorts))
//!             .route("", web::post().to(resorts::create_resort))
//!             .route("/{id}", web::get().to(resorts::get_resort))
//!             .route("/{id}", web::put().to(resorts::update_resort))
//!             .route("/{id}", web::delete().to(resorts::delete_resort))
//!     );
//! }
//! ```
//!
//! # Security Considerations
//!
//! - **API Key Authentication**: All routes (except auth) require valid API keys
//! - **Input Validation**: All input data is validated before processing
//! - **SQL Injection Prevention**: Uses prepared statements with SQLx
//! - **Error Information**: Generic error messages to prevent information leakage
//!
//! # Performance Considerations
//!
//! - **Database Connection Pooling**: Uses MySQL connection pools for efficiency
//! - **Query Optimization**: Optimized SQL queries with proper indexing
//! - **Pagination**: Limits on list endpoints to prevent large data transfers
//! - **Caching**: Consider implementing response caching for frequently accessed data
//!
//! # Future Extensions
//!
//! This module can be extended with additional route handlers as the API grows:
//! - **Real-time Updates**: WebSocket endpoints for live status updates
//! - **Analytics Routes**: Usage statistics and reporting endpoints
//! - **Admin Routes**: Administrative functions and system management
//! - **Webhook Routes**: External system integration endpoints
//!
//! # Error Handling Strategy
//!
//! All route handlers follow a consistent error handling pattern:
//!
//! - **400 Bad Request**: Invalid input data or malformed requests
//! - **401 Unauthorized**: Missing or invalid authentication
//! - **404 Not Found**: Resource not found for specific IDs
//! - **500 Internal Server Error**: Database errors or unexpected failures
//!
//! Author: OpenSlope Team
//! Version: 1.0.0

pub mod resorts;
pub mod lifts;
pub mod slopes;
pub mod status;
pub mod auth;
