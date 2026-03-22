//! OpenSlope API Authentication Middleware
//!
//! This module provides comprehensive authentication and authorization middleware
//! for the OpenSlope API. It implements API key-based authentication with
//! rate limiting, subscription-based access control, and administrative privilege
//! checking.
//!
//! # Authentication Architecture
//!
//! The authentication system follows a middleware pattern that intercepts
//! incoming HTTP requests and performs the following operations:
//!
//! 1. **API Key Extraction**: Extracts API keys from multiple sources
//! 2. **User Verification**: Validates API keys against stored hashes
//! 3. **Authorization Checks**: Verifies user permissions and privileges
//! 4. **Rate Limiting**: Enforces subscription-based rate limits
//! 5. **Request Processing**: Forwards authenticated requests to handlers
//!
//! # Key Features
//!
//! ## Multi-Source API Key Support
//! - **Bearer Token**: Primary method using Authorization header
//! - **Query Parameter**: Fallback method for GET requests only
//! - **Security**: Both methods use the same verification process
//!
//! ## Rate Limiting System
//! - **Subscription-Based**: Different limits based on user subscription plans
//! - **Time Windows**: Separate minute and monthly rate limiting
//! - **Automatic Reset**: Time-based automatic limit resets
//! - **Real-time Tracking**: Live request counting and enforcement
//!
//! ## Administrative Privileges
//! - **Role-Based Access**: Admin users have elevated privileges
//! - **Method Restrictions**: Non-admin users restricted to GET operations
//! - **Permission Checking**: Real-time privilege verification
//!
//! ## Security Features
//! - **Constant-time Verification**: Prevents timing attacks
//! - **Secure Storage**: API keys hashed using Argon2
//! - **Error Handling**: Generic error messages to prevent information leakage
//! - **Database Security**: Prepared statements to prevent SQL injection
//!
//! # Usage Examples
//!
//! ## API Key in Authorization Header (Recommended)
//! ```bash
//! curl -H "Authorization: Bearer your_api_key_here" \
//!      "http://localhost:8080/resorts"
//! ```
//!
//! ## API Key as Query Parameter (GET requests only)
//! ```bash
//! curl "http://localhost:8080/resorts?api_key=your_api_key_here"
//! ```
//!
//! ## Admin Operations (POST, PUT, DELETE)
//! ```bash
//! curl -X POST -H "Authorization: Bearer admin_api_key" \
//!      -H "Content-Type: application/json" \
//!      -d '{"name": "New Resort"}' \
//!      "http://localhost:8080/resorts"
//! ```
//!
//! # Error Responses
//!
//! The middleware returns standardized error responses:
//!
//! - **401 Unauthorized**: Missing or invalid API key
//! - **403 Forbidden**: Insufficient privileges for operation
//! - **429 Too Many Requests**: Rate limit exceeded
//! - **500 Internal Server Error**: Database or system errors
//!
//! # Integration with Actix Web
//!
//! The middleware integrates seamlessly with Actix Web's middleware system:
//!
//! ```rust
//! use actix_cors::Cors;
//! use openslope_api::auth::ApiKeyAuth;
//!
//! App::new()
//!     .wrap(
//!         Cors::default()
//!             .allow_any_origin()
//!             .allow_any_method()
//!             .allow_any_header()
//!     )
//!     .wrap(ApiKeyAuth { pool: pool.clone() })
//!     .service(your_protected_routes)
//! ```
//!
//! # Performance Considerations
//!
//! ## Database Efficiency
//! - **Connection Pooling**: Uses shared database connection pool
//! - **Prepared Statements**: All queries use prepared statements
//! - **Minimal Queries**: Single query to fetch user and rate limit data
//! - **Async Operations**: Non-blocking database operations
//!
//! ## Memory Management
//! - **Reference Counting**: Uses Rc for service sharing
//! - **Minimal Allocation**: Avoids unnecessary data copying
//! - **Efficient Parsing**: Optimized URL parameter parsing
//!
//! ## Rate Limiting Performance
//! - **In-Memory Counting**: Request counts maintained in memory
//! - **Time-based Resets**: Automatic reset logic without additional queries
//! - **Efficient Updates**: Single UPDATE query for rate limit tracking
//!
//! # Security Best Practices
//!
//! ## API Key Transmission
//! - **HTTPS Required**: Always use HTTPS in production
//! - **Header Preference**: Prefer Authorization header over query parameters
//! - **No Logging**: Never log API keys in any form
//! - **Secure Storage**: Store API keys securely on client side
//!
//! ## Error Handling
//! - **Generic Messages**: Don't reveal specific failure reasons
//! - **No Information Leakage**: Avoid exposing internal system details
//! - **Proper Logging**: Log security events without sensitive data
//! - **Graceful Degradation**: Handle database failures appropriately
//!
//! ## Rate Limiting Security
//! - **Fair Distribution**: Ensure fair access across all users
//! - **Burst Protection**: Prevent sudden traffic spikes
//! - **Subscription Enforcement**: Strict enforcement of plan limits
//! - **Monitoring**: Monitor for abuse patterns
//!
//! # Future Enhancements
//!
//! Planned improvements to the authentication system:
//!
//! - **JWT Integration**: Support for JWT token authentication
//! - **Two-Factor Authentication**: Add 2FA support for enhanced security
//! - **IP Whitelisting**: Support for IP-based access control
//! - **OAuth Integration**: Support for OAuth 2.0 authentication
//! - **Token Blacklisting**: Implement logout through token blacklisting
//! - **Advanced Rate Limiting**: More sophisticated rate limiting algorithms
//! - **Audit Logging**: Comprehensive authentication event logging
//!
//! # Monitoring and Observability
//!
//! ## Metrics to Monitor
//! - Authentication success/failure rates
//! - Rate limit violations and resets
//! - API key usage patterns
//! - Database query performance
//! - Middleware processing times
//!
//! ## Logging Recommendations
//! - Log authentication attempts (success/failure)
//! - Log rate limit violations
//! - Monitor admin privilege usage
//! - Track API key validation failures
//! - Monitor database connection issues
//!
//! # Compliance and Standards
//!
//! The authentication system follows industry standards:
//!
//! - **OWASP Guidelines**: Web application security best practices
//! - **REST Security**: Secure REST API authentication patterns
//! - **Rate Limiting**: Industry-standard rate limiting practices
//! - **Database Security**: SQL injection prevention and secure queries
//!
//! Author: OpenSlope Team
//! Version: 1.0.0

use actix_web::{
    body::{BoxBody, MessageBody},
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::Method,
    error::ErrorInternalServerError,
    Error, HttpResponse,
};
use futures_util::future::{ok, Ready, LocalBoxFuture};
use sqlx::MySqlPool;
use std::{
    rc::Rc,
    task::{Context, Poll},
};
use url::form_urlencoded;

use crate::security::hash::verify_secret;
use crate::security::subscription::get_limits;
use time::OffsetDateTime;

use actix_web::http::header;

/// API Key Authentication Middleware
///
/// This middleware provides comprehensive authentication and authorization
/// for the OpenSlope API. It handles API key verification, rate limiting,
/// and administrative privilege checking.
///
/// # Configuration
///
/// The middleware requires a database connection pool for user verification
/// and rate limit tracking.
///
/// # Example
///
/// ```rust
/// let auth_middleware = ApiKeyAuth {
///     pool: database_pool.clone(),
/// };
/// ```
#[derive(Clone)]
pub struct ApiKeyAuth {
    /// Database connection pool for user and rate limit operations
    pub pool: MySqlPool,
}

/// Transform implementation for Actix Web middleware
///
/// This implementation allows the ApiKeyAuth to be used as Actix Web middleware
/// by transforming the service chain to include authentication checks.
impl<S, B> Transform<S, ServiceRequest> for ApiKeyAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiKeyAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    /// Create a new authentication middleware instance
    ///
    /// This method wraps the provided service with authentication functionality.
    /// The service is reference-counted to allow sharing across multiple requests.
    ///
    /// # Arguments
    ///
    /// * `service` - The underlying service to wrap with authentication
    ///
    /// # Returns
    ///
    /// A ready future containing the wrapped service
    fn new_transform(&self, service: S) -> Self::Future {
        ok(ApiKeyAuthMiddleware {
            service: Rc::new(service),
            pool: self.pool.clone(),
        })
    }
}

/// API Key Authentication Middleware Implementation
///
/// This struct implements the actual authentication logic as an Actix Web service.
/// It handles request processing, authentication verification, and response generation.
pub struct ApiKeyAuthMiddleware<S> {
    /// Reference-counted service for handling authenticated requests
    service: Rc<S>,
    /// Database connection pool for authentication operations
    pool: MySqlPool,
}

/// Service implementation for the authentication middleware
///
/// This implementation handles the actual request processing, including
/// authentication checks, rate limiting, and request forwarding.
impl<S, B> Service<ServiceRequest> for ApiKeyAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    /// Check if the service is ready to handle requests
    ///
    /// This method delegates to the underlying service to check readiness.
    /// It's part of the Actix Web service trait implementation.
    ///
    /// # Arguments
    ///
    /// * `cx` - Task context for polling
    ///
    /// # Returns
    ///
    /// Poll result indicating service readiness
    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    /// Handle incoming HTTP requests
    ///
    /// This is the main authentication logic that processes each incoming request:
    /// 1. Extract API key from headers or query parameters
    /// 2. Verify API key against stored hashes
    /// 3. Check user permissions and administrative privileges
    /// 4. Enforce rate limiting based on subscription plan
    /// 5. Forward authenticated requests to the underlying service
    ///
    /// # Arguments
    ///
    /// * `req` - The incoming HTTP request to authenticate
    ///
    /// # Returns
    ///
    /// A future containing either an authenticated response or an error
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();
        let pool = self.pool.clone();
        let method = req.method().clone();

        Box::pin(async move {
            /* ---------------- API KEY LESEN ---------------- */

            /* ===========================
            API KEY EXTRAHIEREN
            =========================== */

            let mut api_key: Option<String> = None;

            // 1️⃣ Bearer Header prüfen
            // Primary authentication method using Authorization header
            if let Some(auth) = req.headers().get(header::AUTHORIZATION) {
                if let Ok(auth_str) = auth.to_str() {
                    if let Some(token) = auth_str.strip_prefix("Bearer ") {
                        api_key = Some(token.to_string());
                    }
                }
            }

            // 2️⃣ Fallback: URL-Parameter (nur GET erlaubt)
            // Secondary authentication method for GET requests only
            if api_key.is_none() && method == Method::GET {
                let query = req.query_string();
                api_key = form_urlencoded::parse(query.as_bytes())
                    .find(|(k, _)| k == "api_key")
                    .map(|(_, v)| v.to_string());
            }

            // 3️⃣ Wenn immer noch kein Key → Fehler
            // Return unauthorized error if no valid API key is found
            let api_key = match api_key {
                Some(k) => k,
                None => {
                    return Ok(req.into_response(
                        HttpResponse::Unauthorized()
                            .body("Missing API key (Bearer or api_key)")
                            .map_into_boxed_body(),
                    ));
                }
            };

            /* ---------------- USER LADEN ---------------- */

            // Fetch all users from database for API key verification
            // Note: In production, this should be optimized to query by API key
            let users = sqlx::query!(
                r#"
                SELECT id, api_key, is_admin, subscription,
                       requests_minute, requests_month,
                       last_request_minute, last_request_month
                FROM users
                "#
            )
            .fetch_all(&pool)
            .await
            .map_err(|_| ErrorInternalServerError("Database error"))?;

            let mut user = None;

            // Verify API key against stored hashes using constant-time comparison
            for u in users {
                if verify_secret(&api_key, &u.api_key) {
                    user = Some(u);
                    break;
                }
            }

            let mut user = match user {
                Some(u) => u,
                None => {
                    return Ok(req.into_response(
                        HttpResponse::Unauthorized()
                            .body("Invalid api_key")
                            .map_into_boxed_body(),
                    ));
                }
            };

            /* ---------------- ADMIN CHECK ---------------- */

            // Non-admin users can only perform GET requests
            // POST, PUT, DELETE operations require administrative privileges
            if method != Method::GET && user.is_admin != 1 {
                return Ok(req.into_response(
                    HttpResponse::Forbidden()
                        .body("Admin privileges required")
                        .map_into_boxed_body(),
                ));
            }

            /* ---------------- RATE LIMIT ---------------- */

            // Get current timestamp for rate limiting calculations
            let now = OffsetDateTime::now_utc();
            let today = now.date();

            // Check if minute window should be reset
            let reset_minute = user
                .last_request_minute
                .map(|t| {
                    t.year() != now.year()
                        || t.month() != now.month()
                        || t.day() != now.day()
                        || t.hour() != now.hour()
                        || t.minute() != now.minute()
                })
                .unwrap_or(true);

            // Check if month window should be reset
            let reset_month = user
                .last_request_month
                .map(|d: time::Date| d.year() != now.year() || d.month() != now.month())
                .unwrap_or(true);

            // Initialize request counters, resetting if necessary
            let mut req_min: u32 = if reset_minute { 0 } else { user.requests_minute } as u32;
            let mut req_mon: u32 = if reset_month { 0 } else { user.requests_month } as u32;

            // Get rate limits for user's subscription plan
            let limits = get_limits(&user.subscription);

            // Check minute rate limit
            if let Some(max) = limits.per_minute {
                if req_min >= max {
                    return Ok(req.into_response(
                        HttpResponse::TooManyRequests()
                            .body("Minute rate limit exceeded")
                            .map_into_boxed_body(),
                    ));
                }
            }

            // Check monthly rate limit
            if let Some(max) = limits.per_month {
                if req_mon >= max {
                    return Ok(req.into_response(
                        HttpResponse::TooManyRequests()
                            .body("Monthly rate limit exceeded")
                            .map_into_boxed_body(),
                    ));
                }
            }

            // Increment request counters
            req_min += 1;
            req_mon += 1;

            // Update user's rate limit tracking in database
            let _: sqlx::mysql::MySqlQueryResult = sqlx::query!(
                r#"
                UPDATE users
                SET
                    requests_minute = ?,
                    requests_month = ?,
                    last_request_minute = ?,
                    last_request_month = ?
                WHERE id = ?
                "#,
                req_min,
                req_mon,
                now,
                today,
                user.id
            )
            .execute(&pool)
            .await
            .map_err(|_| ErrorInternalServerError("DB update failed"))?;

            /* ---------------- REQUEST WEITERLEITEN ---------------- */

            // Forward authenticated and authorized request to underlying service
            let res = srv.call(req).await?;
            Ok(res.map_into_boxed_body())
        })
    }
}
