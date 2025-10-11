use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

// Impor modul lokal
mod config;
mod database;
mod errors;
mod handlers;
mod middleware;
mod models;
mod openapi;
// mod rate_limit; // TODO: Implement with proper tower_governor version compatibility
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
    tracing::info!("Rate limit: {} requests/minute", config.rate_limit_per_minute);
    tracing::info!("Swagger UI: {}", if config.enable_swagger { "enabled" } else { "disabled" });

    // Membuat koneksi pool ke database PostgreSQL
    let db_pool = match PgPool::connect(&config.database_url).await {
        Ok(pool) => {
            tracing::info!("Successfully connected to the database");
            pool
        }
        Err(e) => {
            tracing::error!("Failed to connect to the database: {:?}", e);
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

    // Mengkonfigurasi CORS
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // TODO: Add rate limiting when tower_governor compatibility is resolved
    // let rate_limiter = rate_limit::create_rate_limiter(config.rate_limit_per_minute);

    // Membuat router utama aplikasi
    let app = router::create_router(db_pool, config.enable_swagger)
        // .layer(rate_limiter) // TODO: Enable when rate limiting is implemented
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
