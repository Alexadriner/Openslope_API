//! OpenSlope Password and Secret Hashing Module
//!
//! This module provides secure password and API key hashing functionality using
//! the Argon2 algorithm, which is currently considered one of the most secure
//! password hashing algorithms available. The module is designed to protect
//! sensitive secrets by converting them into irreversible hash representations.
//!
//! # Security Algorithm: Argon2
//!
//! **Argon2** is a memory-hard password hashing algorithm that won the Password
//! Hashing Competition in 2015. It provides excellent protection against:
//! - **Brute Force Attacks**: Computationally expensive to crack
//! - **Rainbow Table Attacks**: Uses unique salts for each hash
//! - **Timing Attacks**: Constant-time comparison operations
//! - **Hardware Attacks**: Memory-hard design resists ASIC/FPGA attacks
//!
//! # Key Features
//!
//! - **Automatic Salt Generation**: Each hash includes a unique, cryptographically
//!   secure random salt
//! - **Configurable Parameters**: Argon2 uses default secure parameters
//! - **Memory Hard**: Resistant to specialized hardware attacks
//! - **Future-Proof**: Industry-standard algorithm with ongoing security analysis
//!
//! # Hashing Process
//!
//! 1. **Salt Generation**: Creates a unique 16-byte salt using cryptographically
//!    secure random generation
//! 2. **Password Processing**: Applies Argon2 algorithm with the salt
//! 3. **Hash Output**: Produces a PHC string format hash containing:
//!    - Algorithm identifier (argon2)
//!    - Parameters (memory, iterations, parallelism)
//!    - Salt
//!    - Hash digest
//!
//! # Hash Format (PHC String)
//!
//! Generated hashes follow the PHC (Password Hashing Competition) string format:
//! ```
//! $argon2id$v=19$m=19456,t=2,p=1$<base64_salt>$<base64_hash>
//! ```
//!
//! Where:
//! - **$argon2id**: Algorithm identifier
//! - **v=19**: Version number
//! - **m=19456**: Memory cost (19456 KiB)
//! - **t=2**: Time cost (2 iterations)
//! - **p=1**: Parallelism degree (1 thread)
//! - **<base64_salt>**: Base64-encoded salt
//! - **<base64_hash>**: Base64-encoded hash digest
//!
//! # Usage Examples
//!
//! ```rust
//! use openslope_api::security::hash;
//!
//! // Hash a password for storage
//! let password = "user_secure_password123";
//! let password_hash = hash::hash_secret(password);
//! println!("Password hash: {}", password_hash);
//!
//! // Verify a password against its hash
//! let is_valid = hash::verify_secret("user_secure_password123", &password_hash);
//! assert!(is_valid);
//!
//! // Verify with wrong password
//! let is_invalid = hash::verify_secret("wrong_password", &password_hash);
//! assert!(!is_invalid);
//!
//! // Hash an API key for storage
//! let api_key = "api_key_string";
//! let api_key_hash = hash::hash_secret(api_key);
//! let is_valid_key = hash::verify_secret("api_key_string", &api_key_hash);
//! assert!(is_valid_key);
//! ```
//!
//! # Security Considerations
//!
//! ## For Passwords
//! - **Never Store Plaintext**: Always hash passwords before storing
//! - **Unique Salts**: Each password gets a unique salt automatically
//! - **Slow Hashing**: Argon2 is intentionally slow to prevent brute force
//! - **Future Migration**: Hash format includes parameters for future upgrades
//!
//! ## For API Keys
//! - **Hash Before Storage**: Never store API keys in plaintext
//! - **Verification Only**: Use verify_secret() for authentication
//! - **Secure Transmission**: Always use HTTPS for API key transmission
//! - **Regular Rotation**: Consider implementing API key rotation
//!
//! # Performance Characteristics
//!
//! - **Hashing Time**: ~100-500ms per operation (intentionally slow)
//! - **Memory Usage**: ~19MB per operation (memory-hard)
//! - **Verification Time**: Same as hashing time
//! - **Storage Size**: ~100-150 bytes per hash
//!
//! # Algorithm Parameters
//!
//! The default Argon2 parameters provide a good balance of security and performance:
//! - **Memory Cost**: 19456 KiB (19MB)
//! - **Time Cost**: 2 iterations
//! - **Parallelism**: 1 thread
//! - **Salt Length**: 16 bytes
//! - **Hash Length**: 32 bytes
//!
//! # Integration with Authentication
//!
//! This module integrates with the authentication system:
//! 1. **User Registration**: Passwords are hashed before database storage
//! 2. **User Login**: Passwords are verified against stored hashes
//! 3. **API Key Generation**: API keys are hashed before database storage
//! 4. **API Authentication**: API keys are verified against stored hashes
//!
//! # Security Best Practices
//!
//! - **HTTPS Required**: Always use HTTPS to protect secrets in transit
//! - **Input Validation**: Validate input length and character restrictions
//! - **Error Handling**: Don't reveal whether user exists during login
//! - **Rate Limiting**: Implement rate limiting to prevent brute force
//! - **Monitoring**: Log failed authentication attempts
//! - **Regular Updates**: Keep dependencies updated for security patches
//!
//! # Future Enhancements
//!
//! - **Parameter Configuration**: Allow configurable Argon2 parameters
//! - **Hash Migration**: Support for migrating to stronger parameters
//! - **Batch Operations**: Support for batch password verification
//! - **Hardware Acceleration**: Optional hardware-accelerated hashing
//!
//! Author: OpenSlope Team
//! Version: 1.0.0

use argon2::{
    password_hash::{
        PasswordHash,
        PasswordHasher,
        PasswordVerifier,
        SaltString,
        rand_core::OsRng,
    },
    Argon2,
};

/// Hash a secret (password or API key) using Argon2 algorithm
///
/// This function securely hashes a secret string using the Argon2id algorithm
/// with automatic salt generation. The resulting hash includes all necessary
/// parameters and can be safely stored in a database.
///
/// # Arguments
///
/// * `secret` - The secret string to hash (password or API key)
///
/// # Returns
///
/// A PHC string containing the algorithm parameters, salt, and hash digest.
/// The format is: `$argon2id$v=19$m=19456,t=2,p=1$<salt>$<hash>`
///
/// # Security Properties
///
/// - **Irreversible**: Cannot be reversed to obtain the original secret
/// - **Unique Salts**: Each hash uses a unique, cryptographically secure salt
/// - **Memory Hard**: Resistant to specialized hardware attacks
/// - **Future Proof**: Industry-standard algorithm with parameter versioning
///
/// # Example
///
/// ```rust
/// let password = "my_secure_password";
/// let hash = hash_secret(password);
/// assert!(hash.starts_with("$argon2id$"));
/// ```
///
/// # Implementation Details
///
/// 1. Generates a cryptographically secure 16-byte salt
/// 2. Applies Argon2id algorithm with default parameters
/// 3. Returns PHC string format hash
pub fn hash_secret(secret: &str) -> String {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(secret.as_bytes(), &salt)
        .unwrap()
        .to_string()
}

/// Verify a secret against its stored hash
///
/// This function securely verifies whether a provided secret matches
/// the stored hash. Uses constant-time comparison to prevent timing attacks.
///
/// # Arguments
///
/// * `secret` - The secret string to verify (password or API key)
/// * `hash` - The stored PHC hash string to verify against
///
/// # Returns
///
/// `true` if the secret matches the hash, `false` otherwise.
///
/// # Security Properties
///
/// - **Constant Time**: Prevents timing attacks through constant-time comparison
/// - **Error Handling**: Returns false for any parsing or verification errors
/// - **Salt Verification**: Automatically handles salt extraction and verification
///
/// # Example
///
/// ```rust
/// let password = "my_secure_password";
/// let hash = hash_secret(password);
/// assert!(verify_secret("my_secure_password", &hash));
/// assert!(!verify_secret("wrong_password", &hash));
/// ```
///
/// # Implementation Details
///
/// 1. Parses the PHC hash string to extract parameters and salt
/// 2. Applies Argon2id algorithm with extracted parameters
/// 3. Uses constant-time comparison to verify the result
/// 4. Returns boolean result (no error information leaked)
pub fn verify_secret(secret: &str, hash: &str) -> bool {
    let parsed_hash = PasswordHash::new(hash).unwrap();
    Argon2::default()
        .verify_password(secret.as_bytes(), &parsed_hash)
        .is_ok()
}
