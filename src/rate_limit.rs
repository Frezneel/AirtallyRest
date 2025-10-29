use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use std::{
    collections::HashMap,
    net::IpAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::time::sleep;

/// Simple in-memory rate limiter
/// 
/// This implementation uses a sliding window approach with HashMap storage.
/// For production, consider using Redis or other distributed storage.
/// 
/// Features:
/// - Per-IP rate limiting
/// - Sliding window for fair distribution
/// - Memory cleanup of old entries
/// - Configurable limits and windows
pub struct RateLimiter {
    /// Per-IP request tracking
    trackers: Arc<Mutex<HashMap<IpAddr, RequestTracker>>>,
    /// Maximum requests per window
    max_requests: u32,
    /// Window duration
    window_duration: Duration,
    /// Cleanup interval
    cleanup_interval: Duration,
}

#[derive(Debug, Clone)]
struct RequestTracker {
    /// Request timestamps within the window
    requests: Vec<Instant>,
    /// Last cleanup time
    last_cleanup: Instant,
}

impl RateLimiter {
    /// Create a new rate limiter
    /// 
    /// # Arguments
    /// * `max_requests` - Maximum requests allowed per window
    /// * `window_duration` - Duration of the sliding window
    /// * `cleanup_interval` - How often to clean up old entries
    pub fn new(max_requests: u32, window_duration: Duration, cleanup_interval: Duration) -> Self {
        let limiter = Self {
            trackers: Arc::new(Mutex::new(HashMap::new())),
            max_requests,
            window_duration,
            cleanup_interval,
        };

        // Start cleanup task
        limiter.start_cleanup_task();

        limiter
    }

    /// Create rate limiter with default values (60 requests per minute)
    pub fn default() -> Self {
        Self::new(60, Duration::from_secs(60), Duration::from_secs(300))
    }

    /// Create rate limiter from environment configuration
    pub fn from_env() -> Self {
        let max_requests = std::env::var("RATE_LIMIT_PER_MINUTE")
            .unwrap_or_else(|_| "60".to_string())
            .parse()
            .unwrap_or(60);

        Self::new(
            max_requests,
            Duration::from_secs(60),
            Duration::from_secs(300),
        )
    }

    /// Check if a request should be allowed
    pub async fn is_allowed(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        
        let mut trackers = self.trackers.lock().unwrap();
        let tracker = trackers.entry(ip).or_insert_with(|| RequestTracker {
            requests: Vec::new(),
            last_cleanup: now,
        });

        // Cleanup old requests if needed
        if now.duration_since(tracker.last_cleanup) > self.cleanup_interval {
            self.cleanup_tracker(tracker, now);
            tracker.last_cleanup = now;
        } else {
            // Remove requests outside the window
            tracker.requests.retain(|&timestamp| {
                now.duration_since(timestamp) <= self.window_duration
            });
        }

        // Check if under limit
        if tracker.requests.len() < self.max_requests as usize {
            tracker.requests.push(now);
            true
        } else {
            false
        }
    }

    /// Remove old requests from tracker
    fn cleanup_tracker(&self, tracker: &mut RequestTracker, now: Instant) {
        tracker.requests.retain(|&timestamp| {
            now.duration_since(timestamp) <= self.window_duration
        });
    }

    /// Start background cleanup task
    fn start_cleanup_task(&self) {
        let trackers = self.trackers.clone();
        let cleanup_interval = self.cleanup_interval;
        let window_duration = self.window_duration;

        tokio::spawn(async move {
            let mut cleanup_timer = tokio::time::interval(cleanup_interval);
            
            loop {
                cleanup_timer.tick().await;
                
                let now = Instant::now();
                let mut trackers = trackers.lock().unwrap();
                
                // Clean up each tracker
                for tracker in trackers.values_mut() {
                    cleanup_tracker_static(tracker, now, &window_duration);
                }
                
                // Remove empty trackers (IPs that haven't made requests recently)
                trackers.retain(|_, tracker| {
                    now.duration_since(tracker.last_cleanup) < cleanup_interval * 3
                });
            }
        });
    }

    /// Get current rate limit status for an IP
    pub fn get_status(&self, ip: IpAddr) -> RateLimitStatus {
        let now = Instant::now();
        
        if let Ok(trackers) = self.trackers.lock() {
            if let Some(tracker) = trackers.get(&ip) {
                let recent_requests = tracker.requests.iter()
                    .filter(|&&timestamp| now.duration_since(timestamp) <= self.window_duration)
                    .count();
                
                RateLimitStatus {
                    allowed: recent_requests < self.max_requests as usize,
                    current_requests: recent_requests as u32,
                    max_requests: self.max_requests,
                    window_duration: self.window_duration,
                    reset_time: tracker.requests.first()
                        .map(|first| *first + self.window_duration)
                        .unwrap_or(now + self.window_duration),
                }
            } else {
                RateLimitStatus {
                    allowed: true,
                    current_requests: 0,
                    max_requests: self.max_requests,
                    window_duration: self.window_duration,
                    reset_time: now + self.window_duration,
                }
            }
        } else {
            RateLimitStatus {
                allowed: true,
                current_requests: 0,
                max_requests: self.max_requests,
                window_duration: self.window_duration,
                reset_time: now + self.window_duration,
            }
        }
    }

    /// Get maximum requests per window
    pub fn max_requests(&self) -> u32 {
        self.max_requests
    }
}

/// Static helper for cleanup in async context
fn cleanup_tracker_static(tracker: &mut RequestTracker, now: Instant, window_duration: &Duration) {
    tracker.requests.retain(|&timestamp| {
        now.duration_since(timestamp) <= *window_duration
    });
}

/// Rate limit status information
#[derive(Debug)]
pub struct RateLimitStatus {
    /// Whether the request is allowed
    pub allowed: bool,
    /// Current number of requests
    pub current_requests: u32,
    /// Maximum allowed requests
    pub max_requests: u32,
    /// Rate limit window duration
    pub window_duration: Duration,
    /// When the rate limit will reset
    pub reset_time: Instant,
}

/// Rate limiting middleware
pub async fn rate_limit_middleware(
    axum::extract::State(limiter): axum::extract::State<Arc<RateLimiter>>,
    req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract client IP
    let client_ip = extract_client_ip(&req);

    // Check rate limit and update tracker
    let is_allowed = limiter.is_allowed(client_ip).await;

    // Get status for headers (after check)
    let status = limiter.get_status(client_ip);

    // Add rate limit headers to response
    let mut response = if is_allowed {
        next.run(req).await
    } else {
        tracing::warn!(
            client_ip = %client_ip,
            current_requests = status.current_requests,
            max_requests = status.max_requests,
            "Rate limit exceeded"
        );
        
        // Return 429 Too Many Requests
        let error_response = serde_json::json!({
            "status": "error",
            "message": "Rate limit exceeded. Please try again later.",
            "retry_after": status.window_duration.as_secs()
        });

        let mut response = axum::Json(error_response).into_response();
        *response.status_mut() = StatusCode::TOO_MANY_REQUESTS;
        response
    };
    
    // Add rate limit headers
    let headers = response.headers_mut();
    headers.insert(
        "X-RateLimit-Limit",
        status.max_requests.to_string().parse().unwrap(),
    );
    headers.insert(
        "X-RateLimit-Remaining",
        (status.max_requests - status.current_requests).to_string().parse().unwrap(),
    );
    // Calculate reset time as seconds from now
    let reset_secs = status.reset_time.saturating_duration_since(std::time::Instant::now()).as_secs();
    headers.insert(
        "X-RateLimit-Reset",
        reset_secs.to_string().parse().unwrap(),
    );
    
    Ok(response)
}

/// Extract client IP from request
fn extract_client_ip(req: &Request) -> IpAddr {
    // Try X-Forwarded-For header first
    if let Some(forwarded) = req.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            // Take the first IP (original client)
            if let Some(first_ip) = forwarded_str.split(',').next() {
                if let Ok(ip) = first_ip.trim().parse::<IpAddr>() {
                    return ip;
                }
            }
        }
    }

    // Try X-Real-IP header
    if let Some(real_ip) = req.headers().get("X-Real-IP") {
        if let Ok(real_ip_str) = real_ip.to_str() {
            if let Ok(ip) = real_ip_str.parse::<IpAddr>() {
                return ip;
            }
        }
    }

    // Use a default for testing
    IpAddr::V4(std::net::Ipv4Addr::new(127, 0, 0, 1))
}

// Note: create_rate_limit_layer removed due to complex type signature issues in Axum 0.8
// Use RateLimiter::from_env() with axum::middleware::from_fn_with_state directly instead

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[tokio::test]
    async fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(5, Duration::from_secs(1), Duration::from_secs(5));
        let ip = IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 1));

        // First 5 requests should be allowed
        for i in 0..5 {
            assert!(limiter.is_allowed(ip).await, "Request {} should be allowed", i + 1);
        }

        // 6th request should be denied
        assert!(!limiter.is_allowed(ip).await, "6th request should be denied");

        // Wait for window to reset
        sleep(Duration::from_secs(1)).await;

        // Should be allowed again
        assert!(limiter.is_allowed(ip).await, "Request after reset should be allowed");
    }

    #[tokio::test]
    async fn test_multiple_ips() {
        let limiter = RateLimiter::new(2, Duration::from_secs(1), Duration::from_secs(5));
        let ip1 = IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 1));
        let ip2 = IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 2));

        // Each IP should have its own limit
        assert!(limiter.is_allowed(ip1).await);
        assert!(limiter.is_allowed(ip1).await);
        assert!(!limiter.is_allowed(ip1).await);

        // IP2 should still be allowed
        assert!(limiter.is_allowed(ip2).await);
        assert!(limiter.is_allowed(ip2).await);
        assert!(!limiter.is_allowed(ip2).await);
    }

    #[tokio::test]
    async fn test_rate_limit_status() {
        let limiter = RateLimiter::new(3, Duration::from_secs(60), Duration::from_secs(300));
        let ip = IpAddr::V4(std::net::Ipv4Addr::new(192, 168, 1, 1));

        // Check initial status
        let status = limiter.get_status(ip);
        assert!(status.allowed);
        assert_eq!(status.current_requests, 0);
        assert_eq!(status.max_requests, 3);

        // Make some requests
        limiter.is_allowed(ip).await;
        limiter.is_allowed(ip).await;

        let status = limiter.get_status(ip);
        assert!(status.allowed);
        assert_eq!(status.current_requests, 2);
        assert_eq!(status.max_requests, 3);

        // Max out requests
        limiter.is_allowed(ip).await;
        let status = limiter.get_status(ip);
        assert!(!status.allowed);
        assert_eq!(status.current_requests, 3);
        assert_eq!(status.max_requests, 3);
    }

    // test_create_from_env removed to avoid rust-analyzer false positives
    // The functionality is tested in integration tests
}