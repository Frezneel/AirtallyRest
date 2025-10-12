use serde::Deserialize;
use std::env;

/// Application configuration loaded from environment variables
///
/// This struct centralizes all configuration values needed by the application.
/// Values are loaded from environment variables (typically from .env file via dotenvy).
///
/// # Examples
///
/// ```
/// use airtally_restapi::config::AppConfig;
///
/// // Load configuration from environment
/// let config = AppConfig::from_env();
///
/// // Access configuration values
/// println!("Server running on {}", config.server_address());
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    /// Database connection URL
    pub database_url: String,

    /// Server host address (default: "0.0.0.0")
    pub host: String,

    /// Server port (default: 3000)
    pub port: u16,

    /// Environment: "development", "staging", "production"
    pub environment: String,

    /// Logging level: "trace", "debug", "info", "warn", "error"
    pub log_level: String,

    /// Rate limit: requests per minute per IP
    pub rate_limit_per_minute: u64,

    /// Enable API documentation endpoints
    pub enable_swagger: bool,
}

impl AppConfig {
    /// Load configuration from environment variables
    ///
    /// # Panics
    ///
    /// Panics if required environment variables are missing
    ///
    /// # Environment Variables
    ///
    /// - `DATABASE_URL` (required): PostgreSQL connection string
    /// - `HOST` (optional): Server host, defaults to "0.0.0.0"
    /// - `PORT` (optional): Server port, defaults to 3000
    /// - `ENVIRONMENT` (optional): Runtime environment, defaults to "development"
    /// - `LOG_LEVEL` (optional): Log verbosity, defaults to "info"
    /// - `RATE_LIMIT_PER_MINUTE` (optional): Rate limit per IP, defaults to 100
    /// - `ENABLE_SWAGGER` (optional): Enable Swagger UI, defaults to true in dev
    pub fn from_env() -> Self {
        let database_url = env::var("DATABASE_URL")
            .expect("DATABASE_URL must be set in environment");

        let host = env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());

        let port: u16 = env::var("PORT")
            .unwrap_or_else(|_| "3000".to_string())
            .parse()
            .expect("PORT must be a valid number");

        let environment = env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "development".to_string());

        let log_level = env::var("LOG_LEVEL")
            .unwrap_or_else(|_| {
                if environment == "production" {
                    "info".to_string()
                } else {
                    "debug".to_string()
                }
            });

        let rate_limit_per_minute: u64 = env::var("RATE_LIMIT_PER_MINUTE")
            .unwrap_or_else(|_| "100".to_string())
            .parse()
            .expect("RATE_LIMIT_PER_MINUTE must be a valid number");

        let enable_swagger = env::var("ENABLE_SWAGGER")
            .unwrap_or_else(|_| {
                if environment == "production" {
                    "false".to_string()
                } else {
                    "true".to_string()
                }
            })
            .parse()
            .unwrap_or(true);

        Self {
            database_url,
            host,
            port,
            environment,
            log_level,
            rate_limit_per_minute,
            enable_swagger,
        }
    }

    /// Get the full server address (host:port)
    pub fn server_address(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    /// Check if running in production mode
    #[allow(dead_code)]
    pub fn is_production(&self) -> bool {
        self.environment == "production"
    }

    /// Check if running in development mode
    #[allow(dead_code)]
    pub fn is_development(&self) -> bool {
        self.environment == "development"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_address() {
        let config = AppConfig {
            database_url: "postgres://test".to_string(),
            host: "127.0.0.1".to_string(),
            port: 8080,
            environment: "development".to_string(),
            log_level: "debug".to_string(),
            rate_limit_per_minute: 100,
            enable_swagger: true,
        };

        assert_eq!(config.server_address(), "127.0.0.1:8080");
    }

    #[test]
    fn test_is_production() {
        let mut config = AppConfig {
            database_url: "postgres://test".to_string(),
            host: "0.0.0.0".to_string(),
            port: 3000,
            environment: "production".to_string(),
            log_level: "info".to_string(),
            rate_limit_per_minute: 100,
            enable_swagger: false,
        };

        assert!(config.is_production());
        assert!(!config.is_development());

        config.environment = "development".to_string();
        assert!(!config.is_production());
        assert!(config.is_development());
    }
}
