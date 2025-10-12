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
        (url = "http://localhost:3000", description = "Local development server"),
        (url = "http://192.168.1.16:3000", description = "Network server")
    ),
    components(
        schemas(
            crate::models::Flight,
            crate::models::ScanData,
            crate::models::DecodedBarcode,
            crate::models::RejectionLog,
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
