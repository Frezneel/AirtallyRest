use crate::{handlers, handlers_auth, middleware, openapi, jwt_middleware};
use axum::{
    middleware as axum_middleware,
    routing::{get, post, put, delete},
    Router,
};
use sqlx::PgPool;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

/// Create the main application router with optional Swagger UI
///
/// # Arguments
///
/// * `db_pool` - PostgreSQL connection pool
/// * `enable_swagger` - Enable Swagger UI documentation endpoint
///
/// # Returns
///
/// Configured Axum router with all API routes
pub fn create_router(db_pool: PgPool, enable_swagger: bool) -> Router {
    // Public routes (no authentication required)
    let public_routes = Router::new()
        // Health check endpoint
        .route("/health", get(handlers::health_check))
        // Authentication endpoints
        .route("/api/auth/login", post(handlers_auth::login))
        // Starter data and code translation (used for offline capability and app startup)
        .route("/api/starter-data/version", get(handlers::get_starter_data_version))
        .route("/api/codes/airports", get(handlers::get_airport_codes))
        .route("/api/codes/airlines", get(handlers::get_airline_codes))
        .route("/api/codes/classes", get(handlers::get_cabin_class_codes));

    // Protected routes (JWT authentication required)
    let protected_routes = Router::new()
        // Auth endpoints (authenticated users)
        .route("/api/auth/logout", post(handlers_auth::logout))
        .route("/api/auth/me", get(handlers_auth::get_current_user))
        .route("/api/auth/change-password", post(handlers_auth::change_password))
        // User management endpoints (require users.* permissions)
        .route("/api/users", get(handlers_auth::list_users).post(handlers_auth::create_user))
        .route(
            "/api/users/{id}",
            get(handlers_auth::get_user_by_id)
                .put(handlers_auth::update_user)
                .delete(handlers_auth::delete_user),
        )
        .route("/api/users/{id}/reset-password", post(handlers_auth::reset_user_password))
        // Role management endpoints
        .route("/api/roles", get(handlers_auth::list_roles))
        .route("/api/roles/{id}", get(handlers_auth::get_role_by_id))
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
        // Apply JWT authentication middleware to all protected routes
        .layer(axum_middleware::from_fn_with_state(db_pool.clone(), jwt_middleware::jwt_auth_middleware));

    // Combine public and protected routes
    let mut router = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        // Menyediakan state (koneksi database) ke semua handler
        .with_state(db_pool)
        // Tambahkan logging middleware untuk mencatat semua request/response termasuk 4xx errors
        .layer(axum_middleware::from_fn(middleware::logging_middleware));

    // Conditionally add Swagger UI
    if enable_swagger {
        let openapi = openapi::ApiDoc::openapi();
        router = router.merge(
            SwaggerUi::new("/swagger-ui")
                .url("/api-docs/openapi.json", openapi)
                .config(openapi::create_swagger_config())
        );
        tracing::info!("Swagger UI enabled at /swagger-ui");
    }

    router
}
