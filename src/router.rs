use crate::{handlers, middleware};
use axum::{
    middleware as axum_middleware,
    routing::{get, post, put, delete},
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
        // Rute untuk endpoint flights_decoder sesuai plan
        .route("/api/flights_decoder", get(handlers::get_flights))
        // Rute untuk Data Scan
        .route("/api/scan-data", get(handlers::get_scan_data).post(handlers::create_scan))
        // Rute untuk Barcode Decoder (TEMPORARILY DISABLED - table decode_barcode belum ada)
        // .route("/api/decode-barcode", post(handlers::decode_barcode))
        // .route("/api/decoded-barcodes", get(handlers::get_decoded_barcodes))
        // Rute untuk Sinkronisasi
        .route("/api/sync/flights", get(handlers::sync_flights))
        .route("/api/sync/flights/bulk", post(handlers::sync_flights_bulk))
        // Menyediakan state (koneksi database) ke semua handler
        .with_state(db_pool)
        // Tambahkan logging middleware untuk mencatat semua request/response termasuk 4xx errors
        .layer(axum_middleware::from_fn(middleware::logging_middleware))
}
