use std::sync::Arc;
use tower_governor::{
    governor::{clock::QuantaClock, GovernorConfigBuilder, middleware::NoOpMiddleware},
    key_extractor::SmartIpKeyExtractor,
    GovernorLayer,
};

// Type alias for the complete GovernorLayer type
type RateLimiter = GovernorLayer<SmartIpKeyExtractor, NoOpMiddleware<QuantaClock>>;

/// Create rate limiting layer using tower-governor
///
/// # Arguments
///
/// * `requests_per_minute` - Maximum requests allowed per minute per IP
///
/// # Returns
///
/// Configured GovernorLayer for rate limiting with automatic IP-based key extraction
///
/// # Example
///
/// ```rust
/// let limiter = create_rate_limiter(100); // 100 req/min per IP
/// ```
pub fn create_rate_limiter(requests_per_minute: u64) -> RateLimiter {
    // Convert requests per minute to per-second rate for finer granularity
    let requests_per_second = (requests_per_minute as f64 / 60.0).max(0.1) as u64;
    let burst_size = requests_per_minute.max(1) as u32;

    // Build governor configuration with per-second rate
    let governor_config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(requests_per_second)
            .burst_size(burst_size)
            .finish()
            .expect("Failed to build rate limiter configuration")
    );

    // Create GovernorLayer with the configuration
    GovernorLayer {
        config: governor_config,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_rate_limiter() {
        let _limiter = create_rate_limiter(100);
        // Just ensure it doesn't panic
    }

    #[test]
    fn test_minimum_rate() {
        let _limiter = create_rate_limiter(0); // Should default to 1
    }
}
