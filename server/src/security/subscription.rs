//! OpenSlope Subscription and Rate Limiting Module
//!
//! This module provides subscription-based rate limiting functionality for the
//! OpenSlope API. It defines different subscription plans with corresponding
//! rate limits to manage API usage and ensure fair access to resources.
//!
//! # Subscription Plans
//!
//! The module defines five subscription tiers, each with different rate limits:
//!
//! ## Free Plan
//! - **Per Minute**: 60 requests
//! - **Per Month**: 2,500 requests
//! - **Target Users**: Individual developers, hobbyists, testing
//!
//! ## Starter Plan
//! - **Per Minute**: 300 requests
//! - **Per Month**: 100,000 requests
//! - **Target Users**: Small projects, startups, light production use
//!
//! ## Pro Plan
//! - **Per Minute**: 1,000 requests
//! - **Per Month**: 500,000 requests
//! - **Target Users**: Growing applications, moderate production use
//!
//! ## Business Plan
//! - **Per Minute**: 3,000 requests
//! - **Per Month**: 3,000,000 requests
//! - **Target Users**: Enterprise applications, high-volume usage
//!
//! ## Enterprise Plan
//! - **Per Minute**: Unlimited
//! - **Per Month**: Unlimited
//! - **Target Users**: Large-scale deployments, custom requirements
//!
//! # Rate Limiting Strategy
//!
//! The rate limiting system uses a dual-limit approach:
//!
//! 1. **Per-Minute Limits**: Prevents short-term abuse and ensures API responsiveness
//! 2. **Per-Month Limits**: Controls long-term usage and billing
//!
//! ## Limit Types
//!
//! - **Finite Limits**: Represented as `Some(u32)` with specific request counts
//! - **Unlimited**: Represented as `None`, allowing unrestricted access
//!
//! # Usage Examples
//!
//! ```rust
//! use openslope_api::security::subscription::{get_limits, RateLimit};
//!
//! // Get limits for different plans
//! let free_limits = get_limits("Free");
//! assert_eq!(free_limits.per_minute, Some(60));
//! assert_eq!(free_limits.per_month, Some(2500));
//!
//! let pro_limits = get_limits("Pro");
//! assert_eq!(pro_limits.per_minute, Some(1000));
//! assert_eq!(pro_limits.per_month, Some(500000));
//!
//! let enterprise_limits = get_limits("Enterprise");
//! assert_eq!(enterprise_limits.per_minute, None);
//! assert_eq!(enterprise_limits.per_month, None);
//!
//! // Handle unknown plans (defaults to Free)
//! let unknown_limits = get_limits("Unknown");
//! assert_eq!(unknown_limits.per_minute, Some(60));
//! ```
//!
//! # Integration with API
//!
//! This module integrates with the API authentication system:
//! 1. **User Authentication**: User's subscription plan is retrieved during authentication
//! 2. **Rate Limit Checking**: API requests are checked against user's limits
//! 3. **Usage Tracking**: Request counts are tracked per user and time period
//! 4. **Limit Enforcement**: Requests are rejected when limits are exceeded
//!
//! # Rate Limiting Implementation
//!
//! While this module defines the limits, the actual rate limiting implementation
//! would typically involve:
//!
//! - **Redis/Memory Storage**: Track request counts per user
//! - **Time Window Management**: Handle minute and month boundaries
//! - **Atomic Operations**: Ensure thread-safe counter updates
//! - **Graceful Degradation**: Handle storage failures appropriately
//!
//! # Performance Characteristics
//!
//! - **Lookup Speed**: O(1) hash map lookup for plan limits
//! - **Memory Usage**: Minimal - only stores limit values
//! - **Thread Safety**: All functions are thread-safe (no shared state)
//!
//! # Security Considerations
//!
//! - **Plan Validation**: Unknown plans default to Free limits
//! - **Integer Overflow**: Use appropriate integer types for large request counts
//! - **Time Accuracy**: Ensure accurate time tracking for monthly limits
//! - **Storage Security**: Protect rate limit counters from tampering
//!
//! # Future Enhancements
//!
//! - **Dynamic Limits**: Allow runtime configuration of rate limits
//! - **Usage Analytics**: Track and report usage patterns
//! - **Burst Limits**: Support for burst requests within limits
//! - **Geographic Limits**: Different limits based on user location
//! - **API Key Limits**: Per-API-key rate limiting
//! - **Real-time Monitoring**: Live rate limit status and alerts
//!
//! # Business Logic
//!
//! The rate limits are designed to:
//! - **Encourage Upgrades**: Free tier provides basic functionality
//! - **Prevent Abuse**: Reasonable limits on all tiers
//! - **Scale with Needs**: Higher tiers support growing applications
//! - **Maintain Performance**: Protect API from overload
//!
//! # Error Handling
//!
//! - **Unknown Plans**: Default to Free plan limits for safety
//! - **Invalid Input**: Plan names are treated as strings (no validation)
//! - **Future Compatibility**: Easy to add new plans without breaking changes
//!
//! Author: OpenSlope Team
//! Version: 1.0.0

/// Rate limit configuration for a subscription plan
///
/// Defines the maximum number of API requests allowed per time period.
/// Uses Option types to support both finite and unlimited limits.
#[derive(Debug, Clone)]
pub struct RateLimit {
    /// Maximum requests per minute (None = unlimited)
    pub per_minute: Option<u32>,
    /// Maximum requests per month (None = unlimited)
    pub per_month: Option<u32>,
}

/// Get rate limits for a specific subscription plan
///
/// Returns the rate limit configuration for the specified subscription plan.
/// Unknown plans default to Free plan limits for security.
///
/// # Arguments
///
/// * `plan` - The subscription plan name (case-sensitive)
///
/// # Returns
///
/// A `RateLimit` struct containing the per-minute and per-month limits
/// for the specified plan.
///
/// # Supported Plans
///
/// - "Free": 60/min, 2,500/month
/// - "Starter": 300/min, 100,000/month
/// - "Pro": 1,000/min, 500,000/month
/// - "Business": 3,000/min, 3,000,000/month
/// - "Enterprise": unlimited/unlimited
/// - Any other value: defaults to Free plan limits
///
/// # Example
///
/// ```rust
/// let limits = get_limits("Pro");
/// assert_eq!(limits.per_minute, Some(1000));
/// assert_eq!(limits.per_month, Some(500000));
/// ```
///
/// # Implementation Notes
///
/// - Uses string matching for plan names (case-sensitive)
/// - Unknown plans default to Free plan for security
/// - Returns owned `RateLimit` struct (no references)
/// - All limits are inclusive (exactly the specified number of requests allowed)
pub fn get_limits(plan: &str) -> RateLimit {
    match plan {
        "Free" => RateLimit {
            per_minute: Some(60),
            per_month: Some(2_500),
        },
        "Starter" => RateLimit {
            per_minute: Some(300),
            per_month: Some(100_000),
        },
        "Pro" => RateLimit {
            per_minute: Some(1_000),
            per_month: Some(500_000),
        },
        "Business" => RateLimit {
            per_minute: Some(3_000),
            per_month: Some(3_000_000),
        },
        "Enterprise" => RateLimit {
            per_minute: None,
            per_month: None,
        },
        _ => RateLimit {
            per_minute: Some(60),
            per_month: Some(2_500),
        },
    }
}
