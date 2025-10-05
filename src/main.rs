use sqlx::PgPool;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing_appender::rolling::{RollingFileAppender, Rotation};

// Impor modul lokal
mod database;
mod errors;
mod handlers;
mod middleware;
mod models;
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

    // Inisialisasi logging dengan output ke console dan file
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "debug".into()), // Show all debug level and above
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

    // Load konfigurasi dari file .env
    dotenvy::dotenv().ok();
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    // Membuat koneksi pool ke database PostgreSQL
    let db_pool = match PgPool::connect(&database_url).await {
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

    // Membuat router utama aplikasi
    let app = router::create_router(db_pool)
        .layer(TraceLayer::new_for_http())
        .layer(cors);

    // Menjalankan server - bind ke 0.0.0.0 agar bisa diakses dari jaringan lain
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000)); // 0.0.0.0:3000
    tracing::info!("Server listening on {} (accessible from network)", addr);
    tracing::info!("Local access: http://127.0.0.1:3000");
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
