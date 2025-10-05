use crate::{handlers, middleware};
use axum::{
    middleware as axum_middleware,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;

pub fn create_router(db_pool: PgPool) -> Router {
    Router::new()
        // Rute untuk Manajemen Penerbangan
        .route("/api/flights", get(handlers::get_flights).post(handlers::create_flight))
        .route(
            "/api/flights/{id}",
            get(handlers::get_flight_by_id)
                .put(handlers::update_flight)
                .delete(handlers::delete_flight),
        )
        .route("/api/flights/{id}/statistics", get(handlers::get_flight_statistics))
        .route("/api/flights/{id}/decoded-statistics", get(handlers::get_decoded_statistics))
        // Rute untuk endpoint flights_decoder sesuai plan
        .route("/api/flights_decoder", get(handlers::get_flights))
        // Rute untuk Data Scan
        .route("/api/scan-data", get(handlers::get_scan_data).post(handlers::create_scan))
        // Rute untuk Barcode Decoder
        .route("/api/decode-barcode", post(handlers::decode_barcode))
        .route("/api/decoded-barcodes", get(handlers::get_decoded_barcodes))
        // Rute untuk Sinkronisasi
        .route("/api/sync/flights", get(handlers::sync_flights))
        .route("/api/sync/flights/bulk", post(handlers::sync_flights_bulk))
        // Rute untuk Rejection Logging
        .route("/api/rejection-logs", get(handlers::get_rejection_logs).post(handlers::create_rejection_log))
        .route("/api/rejection-logs/stats", get(handlers::get_rejection_stats))
        // Rute untuk Translation/Code Mapping
        .route("/api/codes/airports", get(handlers::get_airport_codes))
        .route("/api/codes/airlines", get(handlers::get_airline_codes))
        .route("/api/codes/classes", get(handlers::get_cabin_class_codes))
        .route("/api/codes/status", get(handlers::get_passenger_status_codes))
        .route("/api/starter-data/version", get(handlers::get_starter_data_version))
        // Menyediakan state (koneksi database) ke semua handler
        .with_state(db_pool)
        // Tambahkan logging middleware untuk mencatat semua request/response termasuk 4xx errors
        .layer(axum_middleware::from_fn(middleware::logging_middleware))
}
