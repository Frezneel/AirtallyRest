use utoipa::OpenApi;

/// OpenAPI documentation for AirTally REST API
///
/// This module provides comprehensive API documentation using OpenAPI 3.0 specification.
/// Access the interactive Swagger UI at `/swagger-ui` when enabled.
#[derive(OpenApi)]
#[openapi(
    info(
        title = "AirTally REST API",
        version = "1.0.0",
        description = "REST API for AirTally barcode scanning system. \
                      \n\nThis API handles:\n\
                      - Flight management\n\
                      - Barcode scanning and decoding\n\
                      - Data synchronization\n\
                      - Rejection logging\n\
                      - Code translation (airports, airlines, cabin classes)",
        contact(
            name = "AirTally Support",
            email = "support@airtally.com"
        ),
        license(
            name = "MIT",
            url = "https://opensource.org/licenses/MIT"
        )
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development server")
    ),
    paths(
        crate::handlers::create_flight,
        crate::handlers::get_flights,
        crate::handlers::get_flight_by_id,
        crate::handlers::update_flight,
        crate::handlers::delete_flight,
        crate::handlers::get_flight_statistics,
        crate::handlers::get_decoded_statistics,
        crate::handlers::create_scan,
        crate::handlers::get_scan_data,
        crate::handlers::decode_barcode,
        crate::handlers::get_decoded_barcodes,
        crate::handlers::sync_flights,
        crate::handlers::sync_flights_bulk,
        crate::handlers::create_rejection_log,
        crate::handlers::get_rejection_logs,
        crate::handlers::get_rejection_stats,
        crate::handlers::get_airport_codes,
        crate::handlers::get_airline_codes,
        crate::handlers::get_cabin_class_codes,
        crate::handlers::get_starter_data_version,
    ),
    components(
        schemas(
            crate::models::Flight,
            crate::models::CreateFlight,
            crate::models::UpdateFlight,
            crate::models::FlightStatistics,
            crate::models::DecodedStatistics,
            crate::models::ScanData,
            crate::models::ScanDataInput,
            crate::models::DecodedBarcode,
            crate::models::DecodeRequest,
            crate::models::RejectionLog,
            crate::models::CreateRejectionLog,
            crate::models::AirportCode,
            crate::models::AirlineCode,
            crate::models::CabinClassCode,
        )
    ),
    tags(
        (name = "Flights", description = "Flight management endpoints"),
        (name = "Scanning", description = "Barcode scanning and decoding"),
        (name = "Sync", description = "Data synchronization"),
        (name = "Codes", description = "Code translation and mapping"),
        (name = "Logs", description = "Rejection and error logs")
    )
)]
pub struct ApiDoc;

/// Create Swagger UI configuration
pub fn create_swagger_config() -> utoipa_swagger_ui::Config<'static> {
    utoipa_swagger_ui::Config::default()
        .try_it_out_enabled(true)
        .display_request_duration(true)
        .show_extensions(true)
        .filter(true)
}
