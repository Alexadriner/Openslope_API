//! OpenSlope API Security Module
//!
//! This module provides comprehensive security functionality for the OpenSlope API,
//! including API key generation, password hashing, subscription-based rate limiting,
//! and JWT token management. The security system is designed to protect user data
//! and API resources while providing flexible authentication and authorization mechanisms.
//!
//! # Security Architecture
//!
//! The security module is organized into several specialized submodules:
//!
//! - **api_key**: Cryptographically secure API key generation and management
//! - **hash**: Password and secret hashing using Argon2 for secure storage
//! - **subscription**: Rate limiting and subscription plan management
//! - **jwt**: JSON Web Token creation and validation for session management
//!
//! # Security Features
//!
//! ## Authentication Methods
//! - **API Key Authentication**: Primary authentication method for API endpoints
//! - **JWT Tokens**: Session-based authentication with configurable expiration
//! - **Password Hashing**: Secure password storage using Argon2 algorithm
//!
//! ## Rate Limiting
//! - **Subscription-based Limits**: Different rate limits based on user subscription plans
//! - **Per-minute and Per-month Limits**: Granular control over API usage
//! - **Enterprise Plans**: Unlimited access for enterprise customers
//!
//! ## Key Security Principles
//! - **Cryptographic Security**: Uses industry-standard algorithms and practices
//! - **Secure Random Generation**: Cryptographically secure random number generation
//! - **Salt-based Hashing**: Prevents rainbow table attacks
//! - **Configurable Expiration**: Flexible token and key expiration policies
//!
//! # Usage Examples
//!
//! ```rust
//! use openslope_api::security::{api_key, hash, subscription, jwt};
//!
//! // Generate a new API key
//! let api_key = api_key::generate_api_key();
//!
//! // Hash a password for storage
//! let password_hash = hash::hash_secret("user_password");
//!
//! // Verify a password against its hash
//! let is_valid = hash::verify_secret("user_password", &password_hash);
//!
//! // Get rate limits for a subscription plan
//! let limits = subscription::get_limits("Pro");
//!
//! // Create a JWT token
//! let token = jwt::create_jwt(123, "user@example.com", false);
//! ```
//!
//! # Security Best Practices
//!
//! - **API Keys**: Always use HTTPS in production to protect API keys in transit
//! - **Password Storage**: Never store plaintext passwords, always use hashing
//! - **JWT Secrets**: Use strong, randomly generated secrets for JWT signing
//! - **Rate Limiting**: Implement appropriate rate limits to prevent abuse
//! - **Token Expiration**: Set reasonable expiration times for JWT tokens
//!
//! # Future Security Enhancements
//!
//! - **Two-Factor Authentication**: Add 2FA support for enhanced security
//! - **API Key Rotation**: Implement automatic API key rotation
//! - **Audit Logging**: Add comprehensive security event logging
//! - **IP Whitelisting**: Support for IP-based access control
//! - **OAuth Integration**: Support for OAuth 2.0 authentication
//!
//! Author: OpenSlope Team
//! Version: 1.0.0

pub mod api_key;
pub mod hash;
pub mod subscription;
