use sqlx::PgPool;
use std::{net::SocketAddr, sync::Arc};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use axum::http::{Method, header};
use crate::{rate_limit::RateLimiter, database_config::{create_connection_pool, get_database_config}};

// Impor modul lokal
mod auth_middleware;
mod config;
mod database;
mod database_config;
mod errors;
mod handlers;
mod middleware;
mod models;
mod openapi;
mod rate_limit; // Custom rate limiting implementation
mod router;
mod barcode_parser;  // Shared IATA BCBP parser (synchronized with mobile app)

#[tokio::main]
async fn main() {
    // Setup file appender for error logs
    let file_appender = RollingFileAppender::new(
        Rotation::DAILY,
        "logs",  // Directory untuk menyimpan log files
        "airtally-errors.log",  // Nama file log
    );

    // Load konfigurasi dari file .env terlebih dahulu
    dotenvy::dotenv().ok();
    let config = config::AppConfig::from_env();

    // Inisialisasi logging dengan output ke console dan file menggunakan log_level dari config
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.log_level.clone().into()),
        )
        .with(tracing_subscriber::fmt::layer())  // Console output
        .with(
            tracing_subscriber::fmt::layer()
                .with_writer(file_appender)
                .with_ansi(false)  // Disable ANSI colors in file
                .with_target(true)
                .with_line_number(true)
        )
        .init();

    tracing::info!("Starting AirTally REST API with security hardening");
    tracing::info!("Environment: {}", config.environment);
    tracing::info!("Server address: {}", config.server_address());
    tracing::info!("Rate limit: {} requests/minute", config.rate_limit_per_minute);
    tracing::info!("Swagger UI: {}", if config.enable_swagger { "enabled" } else { "disabled" });
    tracing::info!("Security: API Key authentication enabled");
    tracing::info!("Security: IP filtering enabled");
    tracing::info!("Security: Rate limiting enabled");
    tracing::info!("Security: CORS restricted");

    // Membuat koneksi pool ke database PostgreSQL dengan konfigurasi optimasi
    let db_config = get_database_config(&config);
    let db_pool = match create_connection_pool(&config.database_url, &db_config).await {
        Ok(pool) => {
            tracing::info!("Successfully connected to the database with optimized pool configuration");
            tracing::info!("Pool config: min={}, max={}", db_config.min_connections, db_config.max_connections);
            pool
        }
        Err(e) => {
            tracing::error!("Failed to create database pool: {:?}", e);
            std::process::exit(1);
        }
    };

    // Menjalankan migrasi database saat aplikasi dimulai
    match sqlx::migrate!("./migrations").run(&db_pool).await {
        Ok(_) => tracing::info!("Database migrations ran successfully"),
        Err(e) => {
            tracing::error!("Failed to run database migrations: {:?}", e);
            std::process::exit(1);
        }
    }

    // Mengkonfigurasi CORS - Production: restrict to airport network only
    let cors = if config.is_production() {
        // Production: Only allow specific origins
        CorsLayer::new()
            .allow_origin("http://192.168.1.100".parse::<HeaderValue>().unwrap())
            .allow_origin("http://192.168.100.1".parse::<HeaderValue>().unwrap())
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([
                header::CONTENT_TYPE,
                header::ACCEPT,
                header::X_API_KEY, // API key header
            ])
    } else {
        // Development: Allow localhost and local network
        CorsLayer::new()
            .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
            .allow_origin("http://127.0.0.1:3000".parse::<HeaderValue>().unwrap())
            .allow_origin("http://192.168.1.100".parse::<HeaderValue>().unwrap())
            .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
            .allow_headers([
                header::CONTENT_TYPE,
                header::ACCEPT,
                header::X_API_KEY,
            ])
    };

    // Initialize rate limiter
    let rate_limiter = Arc::new(RateLimiter::from_env());
    tracing::info!("Rate limiting enabled: {} requests/minute", rate_limiter.max_requests);

    // Membuat router utama aplikasi
    let app = router::create_router(db_pool, config.enable_swagger)
        .layer(axum::middleware::from_fn_with_state(config.clone(), auth_middleware::api_auth_middleware))
        .layer(axum::middleware::from_fn_with_state(rate_limiter.clone(), rate_limit::rate_limit_middleware))
        .layer(axum::middleware::from_fn(auth_middleware::security_logging_middleware))
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    // Menjalankan server menggunakan konfigurasi
    let addr: SocketAddr = config.server_address()
        .parse()
        .expect("Failed to parse server address");

    tracing::info!("Server listening on {} (accessible from network)", addr);
    tracing::info!("Local access: http://127.0.0.1:{}", config.port);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
