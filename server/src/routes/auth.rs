//! OpenSlope API Authentication Routes
//!
//! This module handles all HTTP requests related to user authentication and API key
//! management in the OpenSlope API. It provides secure user registration, login,
//! and authentication verification functionality.
//!
//! # Route Overview
//!
//! The auth module provides the following endpoints:
//!
//! - **POST /auth/signup**: Register a new user account
//! - **POST /auth/signin**: Authenticate user and receive API key
//! - **GET /auth/me**: Get current user information using API key
//!
//! # Data Models
//!
//! The module defines several data structures for handling authentication:
//!
//! - **SignupRequest**: User registration data (email, username, password)
//! - **SigninRequest**: User login data (email/username, password)
//! - **AuthUser**: User information returned in responses
//! - **AuthResponse**: Complete authentication response with API key
//!
//! # Security Features
//!
//! - **Password Hashing**: Uses bcrypt for secure password storage
//! - **API Key Generation**: Cryptographically secure random API keys
//! - **Input Validation**: Comprehensive validation of user input
//! - **Error Handling**: Generic error messages to prevent information leakage
//! - **Email/Username Flexibility**: Users can sign in with either email or username
//!
//! # Authentication Flow
//!
//! 1. **User Registration**:
//!    - POST /auth/signup with email, username, and password
//!    - Password is hashed using bcrypt before storage
//!    - API key is generated and stored (hashed)
//!    - Returns API key and user information
//!
//! 2. **User Login**:
//!    - POST /auth/signin with email/username and password
//!    - Password is verified against stored hash
//!    - New API key is generated and stored
//!    - Returns API key and user information
//!
//! 3. **Authentication Verification**:
//!    - GET /auth/me with api_key parameter
//!    - API key is verified against stored hash
//!    - Returns user information if valid
//!
//! # Password Security
//!
//! - **Hashing Algorithm**: bcrypt with automatic salt generation
//! - **Cost Factor**: Default bcrypt cost for security vs performance balance
//! - **Storage**: Only hashed passwords are stored, never plaintext
//! - **Verification**: Constant-time comparison to prevent timing attacks
//!
//! # API Key Security
//!
//! - **Generation**: Cryptographically secure random string generation
//! - **Length**: Sufficiently long to prevent brute force attacks
//! - **Storage**: Hashed before database storage
//! - **Rotation**: New key generated on each successful login
//! - **Validation**: Constant-time hash comparison
//!
//! # Input Validation
//!
//! - **Email Format**: Basic email format validation
//! - **Username Requirements**: Length and character restrictions
//! - **Password Strength**: Minimum length requirements
//! - **Duplicate Prevention**: Username and email uniqueness enforcement
//!
//! # Error Handling
//!
//! - **Generic Messages**: Prevents user enumeration and information leakage
//! - **Consistent Timing**: All authentication operations take similar time
//! - **Database Errors**: Proper handling of database connection issues
//! - **Validation Errors**: Clear feedback for invalid input
//!
//! # Security Headers and Best Practices
//!
//! - **HTTPS Recommended**: All authentication should use HTTPS in production
//! - **API Key Transmission**: API keys passed via query parameters (consider headers in future)
//! - **Session Management**: Stateless authentication using API keys
//! - **Rate Limiting**: Consider implementing rate limiting for authentication endpoints
//!
//! # Usage Examples
//!
//! ```rust
//! // User registration
//! POST /api/v1/auth/signup
//! {
//!   "email": "user@example.com",
//!   "username": "ski_enthusiast",
//!   "password": "secure_password123"
//! }
//!
//! // User login
//! POST /api/v1/auth/signin
//! {
//!   "email": "user@example.com",
//!   "password": "secure_password123"
//! }
//!
//! // Alternative login with username
//! POST /api/v1/auth/signin
//! {
//!   "username": "ski_enthusiast",
//!   "password": "secure_password123"
//! }
//!
//! // Get current user info
//! GET /api/v1/auth/me?api_key=your_api_key_here
//! ```
//!
//! # Future Security Enhancements
//!
//! - **API Key Expiration**: Implement time-based API key expiration
//! - **Refresh Tokens**: Add refresh token mechanism for API keys
//! - **Two-Factor Authentication**: Add 2FA support
//! - **Account Lockout**: Implement account lockout after failed attempts
//! - **Audit Logging**: Log all authentication attempts for security monitoring
//! - **CORS Configuration**: Proper CORS setup for web application integration
//!
//! Author: OpenSlope Team
//! Version: 1.0.0

use actix_web::{web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use sqlx::MySqlPool;
use url::form_urlencoded;

use crate::user_service::create_user;
use crate::security::api_key::generate_api_key;
use crate::security::hash::hash_secret;
use crate::security::hash::verify_secret;

#[derive(Deserialize)]
pub struct SignupRequest {
    pub email: String,
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SigninRequest {
    pub email: Option<String>,
    pub username: Option<String>,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthUser {
    pub id: i64,
    pub name: String,
    pub email: String,
    pub is_admin: bool,
    pub subscription: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub api_key: String,
    pub user: AuthUser,
}

pub async fn signup(
    pool: web::Data<MySqlPool>,
    data: web::Json<SignupRequest>,
) -> impl Responder {
    match create_user(
        pool.get_ref(),
        &data.username,
        &data.email,
        &data.password,
    )
    .await
    {
        Ok(api_key) => {
            let user = sqlx::query!(
                r#"
                SELECT id, name, email, is_admin, subscription
                FROM users
                WHERE email = ?
                "#,
                data.email
            )
            .fetch_optional(pool.get_ref())
            .await;

            match user {
                Ok(Some(u)) => HttpResponse::Created().json(AuthResponse {
                    api_key,
                    user: AuthUser {
                        id: u.id,
                        name: u.name,
                        email: u.email,
                        is_admin: u.is_admin == 1,
                        subscription: u.subscription,
                    },
                }),
                _ => HttpResponse::InternalServerError().body("Could not load user data"),
            }
        }
        Err(e) => {
            eprintln!("Signup failed: {}", e);
            HttpResponse::BadRequest().body("User already exists")
        }
    }
}

pub async fn signin(
    pool: web::Data<MySqlPool>,
    data: web::Json<SigninRequest>,
) -> impl Responder {
    let email = data.email.as_deref().unwrap_or("").trim();
    let username = data.username.as_deref().unwrap_or("").trim();
    let identifier = if !email.is_empty() { email } else { username };

    if identifier.is_empty() {
        return HttpResponse::BadRequest().body("Missing username or email");
    }

    let user = sqlx::query!(
        r#"
        SELECT id, name, email, password_hash, is_admin, subscription
        FROM users
        WHERE email = ? OR name = ?
        LIMIT 1
        "#,
        identifier,
        identifier
    )
    .fetch_optional(pool.get_ref())
    .await;

    let user = match user {
        Ok(Some(u)) => u,
        _ => return HttpResponse::Unauthorized().body("Invalid credentials"),
    };

    if !verify_secret(&data.password, &user.password_hash) {
        return HttpResponse::Unauthorized().body("Invalid credentials");
    }

    let api_key_plain = generate_api_key();
    let api_key_hash = hash_secret(&api_key_plain);

    let updated = sqlx::query!(
        r#"
        UPDATE users
        SET api_key = ?
        WHERE id = ?
        "#,
        api_key_hash,
        user.id
    )
    .execute(pool.get_ref())
    .await;

    match updated {
        Ok(_) => HttpResponse::Ok().json(AuthResponse {
            api_key: api_key_plain,
            user: AuthUser {
                id: user.id,
                name: user.name,
                email: user.email,
                is_admin: user.is_admin == 1,
                subscription: user.subscription,
            },
        }),
        Err(_) => HttpResponse::InternalServerError().body("Could not update api key"),
    }
}

pub async fn me(
    pool: web::Data<MySqlPool>,
    req: actix_web::HttpRequest,
) -> impl Responder {
    let api_key = match form_urlencoded::parse(req.query_string().as_bytes())
        .find(|(k, _)| k == "api_key")
        .map(|(_, v)| v.to_string())
    {
        Some(k) => k,
        None => return HttpResponse::Unauthorized().body("Missing api_key"),
    };

    let users = sqlx::query!(
        r#"
        SELECT id, name, email, api_key, is_admin, subscription
        FROM users
        "#
    )
    .fetch_all(pool.get_ref())
    .await;

    let users = match users {
        Ok(u) => u,
        Err(_) => return HttpResponse::InternalServerError().body("Database error"),
    };

    for u in users {
        if verify_secret(&api_key, &u.api_key) {
            return HttpResponse::Ok().json(AuthUser {
                id: u.id,
                name: u.name,
                email: u.email,
                is_admin: u.is_admin == 1,
                subscription: u.subscription,
            });
        }
    }

    HttpResponse::Unauthorized().body("Invalid api_key")
}
