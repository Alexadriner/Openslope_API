//! OpenSlope API Models Module
//!
//! This module serves as the central registry for all data models used throughout
//! the OpenSlope API. It provides a clean interface to access the different model
//! types that represent ski resorts, lifts, slopes, and database entities.
//!
//! # Module Organization
//!
//! The models module is organized into several submodules:
//!
//! - **resort**: API response models for ski resort data
//! - **db**: Database row structures for direct SQL mapping
//!
//! # Usage
//!
//! ```rust
//! use openslope_api::models::{resort::ResortResponse, db::ResortRow};
//!
//! // Use API response models for JSON serialization
//! let resort_response: ResortResponse = get_resort_data();
//!
//! // Use database models for SQL operations
//! let resort_row: ResortRow = sqlx::query_as!(ResortRow, "SELECT * FROM resorts")
//!     .fetch_one(&pool)
//!     .await?;
//! ```
//!
//! # Design Patterns
//!
//! - **Separation of Concerns**: Database models (`db` module) are separate from API models (`resort` module)
//! - **Type Safety**: All models use appropriate Rust types with proper null handling
//! - **Serialization**: API models implement `Serialize` for JSON responses
//! - **Database Mapping**: Database models implement `sqlx::FromRow` for automatic SQL mapping
//!
//! # Future Extensions
//!
//! This module can be extended with additional model types as the API grows:
//! - User models for authentication
//! - Lift and slope specific API models
//! - Real-time data models for lift/slope status
//! - Analytics and reporting models
//!
//! Author: OpenSlope Team
//! Version: 1.0.0

pub mod resort;
pub mod db;
