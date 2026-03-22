//! OpenSlope JWT Token Management Module
//!
//! This module provides JSON Web Token (JWT) functionality for session-based
//! authentication in the OpenSlope API. JWT tokens are used to maintain user
//! sessions after initial authentication, providing stateless authentication
//! that can be easily verified and scaled.
//!
//! # JWT Overview
//!
//! JSON Web Tokens are a compact, URL-safe means of representing claims to be
//! transferred between two parties. They are digitally signed using a secret
//! (with HMAC) or a public/private key pair using RSA or ECDSA.
//!
//! # Security Features
//!
//! - **Stateless Authentication**: No server-side session storage required
//! - **Digital Signatures**: Tokens are cryptographically signed to prevent tampering
//! - **Expiration Control**: Configurable token expiration times
//! - **Claim-based**: Rich user information can be embedded in tokens
//! - **Cross-domain Support**: Works across different domains and services
//!
//! # Token Structure
//!
//! JWT tokens consist of three parts separated by dots:
//! 1. **Header**: Contains token type and signing algorithm
//! 2. **Payload**: Contains claims about the user and additional data
//! 3. **Signature**: Cryptographic signature to verify token integrity
//!
//! # Claims Structure
//!
//! The module defines a custom claims structure with the following fields:
//!
//! - **sub** (Subject): User ID (i64)
//! - **email**: User's email address (String)
//! - **is_admin**: Administrative privileges flag (bool)
//! - **exp** (Expiration): Token expiration time (usize, Unix timestamp)
//!
//! # Security Considerations
//!
//! ## Secret Management
//! - **Environment Variable**: Secret should be loaded from environment variables
//! - **Strong Secrets**: Use cryptographically strong, randomly generated secrets
//! - **Secret Rotation**: Implement periodic secret rotation for enhanced security
//! - **Access Control**: Limit access to the secret key
//!
//! ## Token Security
//! - **HTTPS Required**: Always use HTTPS to protect tokens in transit
//! - **Expiration**: Set reasonable expiration times to limit exposure
//! - **Storage**: Store tokens securely on the client side (httpOnly cookies preferred)
//! - **Revocation**: Implement token revocation mechanisms for logout
//!
//! # Usage Examples
//!
//! ```rust
//! use openslope_api::security::jwt;
//!
//! // Create a JWT token
//! let user_id = 12345;
//! let email = "user@example.com";
//! let is_admin = false;
//!
//! match jwt::create_jwt(user_id, email, is_admin) {
//!     Ok(token) => {
//!         println!("Generated JWT: {}", token);
//!         // Send token to client or include in response
//!     }
//!     Err(error) => {
//!         eprintln!("Failed to create JWT: {}", error);
//!     }
//! }
//!
//! // Token format example:
//! // eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOjEyMzQ1LCJlbWFpbCI6InVzZXJAZXhhbXBsZS5jb20iLCJpc19hZG1pbiI6ZmFsc2UsImV4cCI6MTcwMDAwMDAwMH0.signature
//! ```
//!
//! # Integration with Authentication
//!
//! This module integrates with the authentication system:
//! 1. **User Login**: JWT tokens are created after successful password verification
//! 2. **Session Management**: Tokens maintain user sessions across requests
//! 3. **Authorization**: Tokens contain user information for access control
//! 4. **API Access**: Clients include tokens in Authorization header
//!
//! # Token Verification
//!
//! While this module only provides token creation, verification would typically:
//! 1. Extract token from Authorization header
//! 2. Decode and verify the token signature
//! 3. Check expiration time
//! 4. Extract user claims for authorization
//!
//! # Performance Characteristics
//!
//! - **Creation Speed**: Fast token generation (~1-5ms)
//! - **Verification Speed**: Fast token verification (~1-5ms)
//! - **Storage**: No server-side storage required
//! - **Network**: Minimal overhead (single header)
//!
//! # Configuration Options
//!
//! ## Token Expiration
//! - **Default**: 24 hours
//! - **Recommended Range**: 15 minutes to 7 days
//! - **Considerations**: Balance security vs user experience
//!
//! ## Secret Key
//! - **Current**: Hardcoded (for development only)
//! - **Production**: Should be loaded from environment variables
//! - **Length**: Minimum 256 bits recommended
//!
//! # Security Best Practices
//!
//! ## Development vs Production
//! - **Development**: Use different secrets for each environment
//! - **Production**: Use strong, randomly generated secrets
//! - **Version Control**: Never commit secrets to version control
//!
//! ## Token Management
//! - **Short Expiration**: Use shorter expiration times for sensitive applications
//! - **Refresh Tokens**: Implement refresh token mechanism for long-lived sessions
//! - **Logout**: Implement token blacklisting or short expiration for logout
//! - **Monitoring**: Log token creation and suspicious activity
//!
//! # Future Enhancements
//!
//! - **Refresh Tokens**: Add support for refresh token mechanism
//! - **Token Blacklisting**: Implement logout through token blacklisting
//! - **Multiple Algorithms**: Support for different signing algorithms
//! - **Custom Claims**: Add support for custom claim types
//! - **Token Rotation**: Automatic token refresh and rotation
//! - **Audit Logging**: Comprehensive token usage logging
//!
//! # Error Handling
//!
//! The module returns `Result<String, jsonwebtoken::errors::Error>` to handle:
//! - **Encoding Errors**: Issues during token creation
//! - **Invalid Claims**: Problems with claim structure
//! - **Secret Issues**: Problems with the signing secret
//!
//! # Algorithm Security
//!
//! - **Current**: HS256 (HMAC with SHA-256)
//! - **Alternatives**: RS256 (RSA), ES256 (ECDSA) for asymmetric signing
//! - **Key Size**: 256-bit minimum for symmetric algorithms
//!
//! Author: OpenSlope Team
//! Version: 1.0.0

use chrono::{Utc, Duration};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation};
use serde::{Serialize, Deserialize};

/// JWT Secret Key
///
/// **WARNING**: This is hardcoded for development purposes only.
/// In production, this should be loaded from environment variables.
/// The secret key is used to sign and verify JWT tokens.
const JWT_SECRET: &[u8] = b"SUPER_SECRET_CHANGE_ME"; // später aus ENV!

/// JWT Claims Structure
///
/// Defines the payload structure for JWT tokens used in the OpenSlope API.
/// Contains essential user information and metadata for session management.
///
/// # Fields
///
/// - **sub**: Subject identifier (user ID)
/// - **email**: User's email address
/// - **is_admin**: Administrative privileges flag
/// - **exp**: Token expiration time (Unix timestamp)
///
/// # Serialization
///
/// The struct implements both `Serialize` and `Deserialize` traits to support
/// JWT encoding and decoding operations.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    /// User ID (subject identifier)
    pub sub: i64,
    /// User's email address
    pub email: String,
    /// Administrative privileges flag
    pub is_admin: bool,
    /// Token expiration time (Unix timestamp)
    pub exp: usize,
}

/// Create a JWT token for a user
///
/// Generates a signed JWT token containing user information and expiration time.
/// The token can be used for stateless authentication across API requests.
///
/// # Arguments
///
/// * `user_id` - The user's unique identifier (i64)
/// * `email` - The user's email address (String slice)
/// * `is_admin` - Whether the user has administrative privileges (bool)
///
/// # Returns
///
/// A `Result` containing either:
/// - **Ok(String)**: The generated JWT token as a string
/// - **Err(jsonwebtoken::errors::Error)**: Error during token creation
///
/// # Token Properties
///
/// - **Algorithm**: HS256 (HMAC with SHA-256)
/// - **Expiration**: 24 hours from creation time
/// - **Claims**: User ID, email, admin status, and expiration time
///
/// # Example
///
/// ```rust
/// let user_id = 12345;
/// let email = "user@example.com";
/// let is_admin = false;
///
/// match create_jwt(user_id, email, is_admin) {
///     Ok(token) => println!("JWT created successfully: {}", token),
///     Err(error) => eprintln!("Failed to create JWT: {}", error),
/// }
/// ```
///
/// # Security Notes
///
/// - Tokens are signed with the application secret key
/// - Expiration time is set to 24 hours from creation
/// - All user information is embedded in the token payload
/// - Tokens should be transmitted over HTTPS only
///
/// # Implementation Details
///
/// 1. Calculates expiration time (24 hours from now)
/// 2. Creates claims structure with user information
/// 3. Encodes claims using HS256 algorithm and application secret
/// 4. Returns the resulting JWT token string
pub fn create_jwt(
    user_id: i64,
    email: &str,
    is_admin: bool,
) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
        .checked_add_signed(Duration::hours(24))
        .unwrap()
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id,
        email: email.to_string(),
        is_admin,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET),
    )
}
