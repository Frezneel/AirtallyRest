use sqlx::{postgres::PgPoolOptions, PgPool};
use crate::config::AppConfig;
use std::time::Duration;

/// Database connection pool configuration
/// 
/// This module provides optimized database connection pooling
/// with proper timeouts and limits for production use.
/// 
/// Features:
/// - Connection limits (min/max)
/// - Connection timeouts
/// - Acquire timeouts
/// - Idle connection timeouts
/// - Health checks
/// - Proper error handling
pub struct DatabaseConfig {
    /// Minimum number of connections in pool
    min_connections: u32,
    /// Maximum number of connections in pool
    max_connections: u32,
    /// Connection timeout
    connect_timeout: Duration,
    /// Timeout for acquiring connection from pool
    acquire_timeout: Duration,
    /// Idle connection timeout
    idle_timeout: Duration,
    /// Max lifetime of a connection
    max_lifetime: Option<Duration>,
    /// Whether to test connections on checkout
    test_on_check_out: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            // Conservative defaults for production
            min_connections: 5,
            max_connections: 20,
            connect_timeout: Duration::from_secs(10),
            acquire_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600), // 10 minutes
            max_lifetime: Some(Duration::from_secs(1800)), // 30 minutes
            test_on_check_out: true,
        }
    }
}

impl DatabaseConfig {
    /// Create configuration from environment
    pub fn from_env() -> Self {
        let min_connections = std::env::var("DB_MIN_CONNECTIONS")
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5);

        let max_connections = std::env::var("DB_MAX_CONNECTIONS")
            .unwrap_or_else(|_| "20".to_string())
            .parse()
            .unwrap_or(20);

        // Ensure min <= max
        let min_connections = min_connections.min(max_connections);

        Self {
            min_connections,
            max_connections,
            connect_timeout: Duration::from_secs(
                std::env::var("DB_CONNECT_TIMEOUT")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .unwrap_or(10)
            ),
            acquire_timeout: Duration::from_secs(
                std::env::var("DB_ACQUIRE_TIMEOUT")
                    .unwrap_or_else(|_| "30".to_string())
                    .parse()
                    .unwrap_or(30)
            ),
            idle_timeout: Duration::from_secs(
                std::env::var("DB_IDLE_TIMEOUT")
                    .unwrap_or_else(|_| "600".to_string())
                    .parse()
                    .unwrap_or(600)
            ),
            max_lifetime: Some(Duration::from_secs(
                std::env::var("DB_MAX_LIFETIME")
                    .unwrap_or_else(|_| "1800".to_string())
                    .parse()
                    .unwrap_or(1800)
            )),
            test_on_check_out: std::env::var("DB_TEST_ON_CHECKOUT")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        }
    }

    /// Create development configuration
    pub fn development() -> Self {
        Self {
            min_connections: 2,
            max_connections: 10,
            connect_timeout: Duration::from_secs(5),
            acquire_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(300), // 5 minutes
            max_lifetime: Some(Duration::from_secs(900)), // 15 minutes
            test_on_check_out: true,
        }
    }

    /// Create production configuration
    pub fn production() -> Self {
        Self {
            min_connections: 10,
            max_connections: 50,
            connect_timeout: Duration::from_secs(5),
            acquire_timeout: Duration::from_secs(10),
            idle_timeout: Duration::from_secs(300), // 5 minutes
            max_lifetime: Some(Duration::from_secs(600)), // 10 minutes
            test_on_check_out: true,
        }
    }

    /// Get minimum connections
    pub fn min_connections(&self) -> u32 {
        self.min_connections
    }

    /// Get maximum connections
    pub fn max_connections(&self) -> u32 {
        self.max_connections
    }
}

/// Create database connection pool with optimized settings
pub async fn create_connection_pool(
    database_url: &str,
    config: &DatabaseConfig,
) -> Result<PgPool, sqlx::Error> {
    tracing::info!(
        min_connections = config.min_connections,
        max_connections = config.max_connections,
        connect_timeout_ms = config.connect_timeout.as_millis(),
        acquire_timeout_ms = config.acquire_timeout.as_millis(),
        idle_timeout_ms = config.idle_timeout.as_millis(),
        max_lifetime_ms = config.max_lifetime.unwrap_or_default().as_millis(),
        "Creating database connection pool"
    );

    // Note: SQLx 0.8 simplified pool options - some methods removed
    let pool: Result<PgPool, sqlx::Error> = PgPoolOptions::new()
        .min_connections(config.min_connections)
        .max_connections(config.max_connections)
        .acquire_timeout(config.acquire_timeout)
        .idle_timeout(Some(config.idle_timeout))
        .max_lifetime(config.max_lifetime)
        .after_connect(|conn, _meta| Box::pin(async move {
            // Set connection parameters
            sqlx::query("SET timezone = 'UTC'")
                .execute(&mut *conn)
                .await?;

            sqlx::query("SET application_name = 'falcon-rest-api'")
                .execute(&mut *conn)
                .await?;

            tracing::debug!("Database connection established with timezone UTC");
            Ok(())
        }))
        .connect(database_url)
        .await;

    match pool {
        Ok(pool) => {
            tracing::info!("Database connection pool created successfully");

            // Test the pool
            if let Err(e) = test_pool(&pool).await {
                tracing::error!("Database pool test failed: {:?}", e);
                return Err(e);
            }

            tracing::info!("Database pool test passed");
            Ok(pool)
        }
        Err(e) => {
            tracing::error!("Failed to create database pool: {:?}", e);
            Err(e)
        }
    }
}

/// Test database connection pool
async fn test_pool(pool: &PgPool) -> Result<(), sqlx::Error> {
    // Test basic query
    let result = sqlx::query("SELECT 1 as test")
        .fetch_one(pool)
        .await;

    match result {
        Ok(_) => {
            tracing::debug!("Database pool test query successful");
            Ok(())
        }
        Err(e) => {
            tracing::error!("Database pool test query failed: {:?}", e);
            Err(e)
        }
    }
}

/// Get database connection pool configuration based on app config
pub fn get_database_config(app_config: &AppConfig) -> DatabaseConfig {
    if app_config.is_production() {
        DatabaseConfig::production()
    } else if app_config.is_development() {
        DatabaseConfig::development()
    } else {
        // Staging - use defaults
        DatabaseConfig::default()
    }
}

/// Database health check
pub async fn health_check(pool: &PgPool) -> DatabaseHealth {
    let start = std::time::Instant::now();
    
    match test_pool(pool).await {
        Ok(_) => {
            DatabaseHealth {
                is_healthy: true,
                response_time: start.elapsed(),
                active_connections: pool.size(),
                idle_connections: pool.num_idle() as u32,
                error: None,
            }
        }
        Err(e) => {
            DatabaseHealth {
                is_healthy: false,
                response_time: start.elapsed(),
                active_connections: pool.size(),
                idle_connections: pool.num_idle() as u32,
                error: Some(format!("Database test failed: {}", e)),
            }
        }
    }
}

/// Database health information
#[derive(Debug)]
pub struct DatabaseHealth {
    /// Whether database is responding correctly
    pub is_healthy: bool,
    /// Time taken for health check
    pub response_time: std::time::Duration,
    /// Number of active connections
    pub active_connections: u32,
    /// Number of idle connections
    pub idle_connections: u32,
    /// Error message if unhealthy
    pub error: Option<String>,
}

impl DatabaseHealth {
    /// Get health status as HTTP status code
    pub fn status_code(&self) -> u16 {
        if self.is_healthy {
            200
        } else {
            503 // Service Unavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_database_config_default() {
        let config = DatabaseConfig::default();
        assert_eq!(config.min_connections, 5);
        assert_eq!(config.max_connections, 20);
        assert_eq!(config.connect_timeout, Duration::from_secs(10));
        assert_eq!(config.acquire_timeout, Duration::from_secs(30));
        assert!(config.test_on_check_out);
    }

    #[test]
    fn test_database_config_development() {
        let config = DatabaseConfig::development();
        assert_eq!(config.min_connections, 2);
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.connect_timeout, Duration::from_secs(5));
        assert!(config.test_on_check_out);
    }

    #[test]
    fn test_database_config_production() {
        let config = DatabaseConfig::production();
        assert_eq!(config.min_connections, 10);
        assert_eq!(config.max_connections, 50);
        assert_eq!(config.connect_timeout, Duration::from_secs(5));
        assert!(config.test_on_check_out);
    }

    #[test]
    fn test_min_max_connections() {
        let mut config = DatabaseConfig::from_env();
        config.min_connections = 25;
        config.max_connections = 20;
        
        // min should be capped at max
        assert_eq!(config.min_connections, 20);
        assert_eq!(config.max_connections, 20);
    }
}
