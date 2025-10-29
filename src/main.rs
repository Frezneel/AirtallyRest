use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_appender::rolling::{RollingFileAppender, Rotation};
use axum::http::{Method, header, HeaderValue, HeaderName};
use crate::database_config::{create_connection_pool, get_database_config};

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

    tracing::info!("Starting AirTally REST API");
    tracing::info!("Environment: {}", config.environment);
    tracing::info!("Server address: {}", config.server_address());
    tracing::info!("Swagger UI: {}", if config.enable_swagger { "enabled" } else { "disabled" });
    tracing::info!("Security: API Key authentication enabled");
    tracing::info!("Security: CORS configured");

    // Membuat koneksi pool ke database PostgreSQL dengan konfigurasi optimasi
    let db_config = get_database_config(&config);
    let db_pool = match create_connection_pool(&config.database_url, &db_config).await {
        Ok(pool) => {
            tracing::info!("Successfully connected to the database with optimized pool configuration");
            tracing::info!("Pool config: min={}, max={}", db_config.min_connections(), db_config.max_connections());
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

    // Mengkonfigurasi CORS - Allow all origins for simplicity
    let cors = CorsLayer::permissive()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([
            header::CONTENT_TYPE,
            header::ACCEPT,
            HeaderName::from_static("x-api-key"),
        ]);

    tracing::info!("CORS: Permissive mode (all origins allowed)");

    // Membuat router utama aplikasi
    // Security: Only API Key authentication (no rate limiting, no IP whitelist)
    let app = router::create_router(db_pool, config.enable_swagger)
        .layer(axum::middleware::from_fn_with_state(config.clone(), auth_middleware::api_key_only_middleware))
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
