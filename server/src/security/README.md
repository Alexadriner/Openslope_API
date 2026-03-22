# OpenSlope API Security Module

This directory contains the comprehensive security implementation for the OpenSlope API, providing authentication, authorization, and rate limiting functionality.

## Overview

The security module is organized into specialized submodules that work together to provide a robust security framework:

- **[api_key.rs](./api_key.rs)** - Cryptographically secure API key generation
- **[hash.rs](./hash.rs)** - Password and secret hashing using Argon2
- **[jwt.rs](./jwt.rs)** - JSON Web Token creation and management
- **[subscription.rs](./subscription.rs)** - Subscription-based rate limiting
- **[mod.rs](./mod.rs)** - Module exports and documentation

## Security Architecture

### Authentication Methods

The OpenSlope API supports multiple authentication methods:

1. **API Key Authentication** (Primary)
   - Used for most API endpoints
   - Passed via query parameters: `?api_key=your_key_here`
   - Hashed and stored securely in the database
   - Generated using cryptographically secure random generation

2. **JWT Token Authentication** (Session-based)
   - Used for user sessions after login
   - Contains user information and expiration time
   - Stateful authentication with configurable expiration
   - Includes user ID, email, and admin status

### Security Features

- **Cryptographic Security**: Industry-standard algorithms and practices
- **Salt-based Hashing**: Prevents rainbow table attacks
- **Rate Limiting**: Subscription-based usage control
- **Secure Random Generation**: Cryptographically secure key generation
- **Configurable Expiration**: Flexible token and key expiration policies

## API Endpoints and Authentication

### Public Endpoints (No Authentication Required)

- **POST /signup** - User registration
- **POST /signin** - User login (returns JWT token)

### Protected Endpoints (API Key Required)

All protected endpoints require an API key passed via query parameter:

```bash
# Example API request with API key
curl "http://localhost:8080/resorts?api_key=your_api_key_here"
```

Protected endpoints include:
- **GET /me** - Get current user info
- **GET /resorts** - Get all resorts
- **GET /resorts/{id}** - Get specific resort
- **POST /resorts** - Create new resort
- **PUT /resorts/{id}** - Update resort
- **DELETE /resorts/{id}** - Delete resort
- **GET /slopes** - Get all slopes
- **GET /slopes/{id}** - Get specific slope
- **POST /slopes** - Create new slope
- **PUT /slopes/{id}** - Update slope
- **DELETE /slopes/{id}** - Delete slope
- **GET /lifts** - Get all lifts
- **GET /lifts/{id}** - Get specific lift
- **POST /lifts** - Create new lift
- **PUT /lifts/{id}** - Update lift
- **DELETE /lifts/{id}** - Delete lift
- **GET /scrape-runs** - List all scraping operations
- **GET /scrape-runs/{id}** - Get specific scraping run details
- **GET /status-snapshots** - List all status snapshots
- **GET /resorts/{resort_id}/status-snapshots** - Get snapshots for specific resort

## Security Implementation Details

### API Key Generation

```rust
use openslope_api::security::api_key;

// Generate a new API key
let api_key = api_key::generate_api_key();
// Example output: "4f7a9c2e8b1d5f3a7c9e2d4f8a6b3c5e7d9f1a2b4c6d8e0f3a5b7c9d1e3f5a7"
```

**Properties:**
- 32 bytes (256 bits) of entropy
- URL-safe base64 encoding
- 43 characters long
- Cryptographically secure random generation

### Password Hashing

```rust
use openslope_api::security::hash;

// Hash a password for storage
let password_hash = hash::hash_secret("user_password");

// Verify a password against its hash
let is_valid = hash::verify_secret("user_password", &password_hash);
```

**Properties:**
- Argon2id algorithm (memory-hard)
- Automatic salt generation
- PHC string format
- Constant-time verification

### JWT Token Creation

```rust
use openslope_api::security::jwt;

// Create a JWT token
match jwt::create_jwt(user_id, "user@example.com", false) {
    Ok(token) => println!("JWT: {}", token),
    Err(error) => eprintln!("Error: {}", error),
}
```

**Properties:**
- HS256 algorithm (HMAC with SHA-256)
- 24-hour expiration by default
- Contains user ID, email, and admin status
- Cryptographically signed

### Rate Limiting

```rust
use openslope_api::security::subscription::{get_limits, RateLimit};

// Get limits for different subscription plans
let free_limits = get_limits("Free");        // 60/min, 2,500/month
let pro_limits = get_limits("Pro");          // 1,000/min, 500,000/month
let enterprise_limits = get_limits("Enterprise"); // unlimited/unlimited
```

**Subscription Plans:**
- **Free**: 60/min, 2,500/month
- **Starter**: 300/min, 100,000/month
- **Pro**: 1,000/min, 500,000/month
- **Business**: 3,000/min, 3,000,000/month
- **Enterprise**: unlimited/unlimited

## Security Best Practices

### For API Keys
- Always use HTTPS in production
- Store API keys securely (never in client-side code)
- Implement key rotation policies
- Monitor API key usage for suspicious activity
- Set appropriate expiration times

### For Passwords
- Never store plaintext passwords
- Use strong hashing algorithms (Argon2)
- Implement rate limiting on login attempts
- Use secure password policies
- Log failed authentication attempts

### For JWT Tokens
- Set reasonable expiration times
- Use secure secret keys
- Store tokens securely on client side
- Implement token revocation for logout
- Use httpOnly cookies when possible

### For Rate Limiting
- Set appropriate limits based on subscription
- Monitor usage patterns
- Implement graceful degradation
- Log limit violations
- Consider geographic restrictions

## Integration with Authentication System

The security modules integrate with the authentication system in the following ways:

1. **User Registration** (`/signup`)
   - Passwords are hashed using `hash::hash_secret()`
   - API keys are generated using `api_key::generate_api_key()`
   - Hashed keys are stored in the database

2. **User Login** (`/signin`)
   - Passwords are verified using `hash::verify_secret()`
   - JWT tokens are created using `jwt::create_jwt()`
   - User subscription plan is retrieved for rate limiting

3. **API Authentication** (Protected endpoints)
   - API keys are verified against stored hashes
   - Rate limits are checked based on user subscription
   - Requests are processed if authentication and limits pass

## Security Configuration

### Environment Variables

- **JWT_SECRET**: Secret key for JWT signing (should be loaded from environment)
- **DATABASE_URL**: Database connection string
- **API_KEY_LENGTH**: Length of generated API keys (currently hardcoded to 32 bytes)

### Security Headers

The API includes CORS middleware for cross-origin requests:
- Allows requests from any origin (for development)
- Supports all HTTP methods and headers
- Enables credentials support

## Future Security Enhancements

Planned security improvements include:
- Two-Factor Authentication (2FA)
- API Key rotation mechanisms
- Comprehensive audit logging
- IP whitelisting support
- OAuth 2.0 integration
- Refresh token mechanism for JWT
- Token blacklisting for logout
- Custom claim support in JWT

## Security Monitoring

Recommended monitoring practices:
- Log all authentication attempts
- Monitor API usage patterns
- Alert on unusual activity
- Track rate limit violations
- Monitor token creation and usage
- Log security events and errors

## Compliance and Standards

The security implementation follows industry standards:
- **OWASP Guidelines**: Web application security best practices
- **NIST Recommendations**: Cryptographic algorithm standards
- **JWT RFC 7519**: JSON Web Token standard
- **Argon2**: Password hashing competition winner
- **HTTPS**: Secure transmission requirements

## Troubleshooting

### Common Issues

1. **API Key Not Working**
   - Check that the key is passed correctly in query parameters
   - Verify the key hasn't expired
   - Ensure the key is not corrupted during transmission

2. **JWT Token Verification Failed**
   - Check token expiration time
   - Verify the secret key is correct
   - Ensure the token format is valid

3. **Rate Limit Exceeded**
   - Check user's subscription plan
   - Verify usage tracking is working
   - Consider implementing retry logic with exponential backoff

4. **Password Verification Failed**
   - Ensure passwords are hashed before storage
   - Check that the same hashing algorithm is used
   - Verify salt generation is working correctly

## Security Testing

Recommended security testing practices:
- Test password hashing and verification
- Verify API key generation and validation
- Test JWT token creation and expiration
- Validate rate limiting functionality
- Test error handling and security responses
- Perform penetration testing on authentication flows

## Contact and Support

For security-related questions or concerns:
- Review the code comments for detailed implementation details
- Check the integration examples in the main application
- Consult the security best practices documentation
- Report security vulnerabilities through appropriate channels

---

**Author**: OpenSlope Team  
**Version**: 1.0.0  
**Last Updated**: March 2026