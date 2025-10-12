use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use validator::ValidationErrors;

// Enum untuk menangani berbagai jenis error di aplikasi
#[derive(Debug)]
#[allow(dead_code)] // Some variants are reserved for future use
pub enum AppError {
    DatabaseError(sqlx::Error),
    ValidationError(ValidationErrors),
    FlightNotFound,
    DuplicateFlight,
    InvalidDepartureTime,
    InvalidBarcodeFormat,
    // Tambahkan jenis error lain di sini jika diperlukan
}

// Implementasi konversi dari error lain ke AppError
impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::DatabaseError(err)
    }
}

impl From<ValidationErrors> for AppError {
    fn from(err: ValidationErrors) -> Self {
        AppError::ValidationError(err)
    }
}

// Implementasi bagaimana AppError diubah menjadi HTTP Response
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message, error_code, details) = match self {
            AppError::DatabaseError(ref e) => {
                tracing::error!(
                    error = ?e,
                    error_type = "DatabaseError",
                    "Database operation failed"
                );
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Internal server error".to_string(),
                    "INTERNAL_ERROR".to_string(),
                    json!({}),
                )
            }
            AppError::ValidationError(ref e) => {
                tracing::warn!(
                    validation_errors = ?e.field_errors(),
                    error_type = "ValidationError",
                    "Request validation failed"
                );
                (
                    StatusCode::BAD_REQUEST,
                    "Validation failed".to_string(),
                    "VALIDATION_ERROR".to_string(),
                    json!({ "details": e.field_errors() }),
                )
            }
            AppError::FlightNotFound => {
                tracing::warn!(
                    error_type = "FlightNotFound",
                    "Attempted to access non-existent flight"
                );
                (
                    StatusCode::NOT_FOUND,
                    "Flight with given ID not found".to_string(),
                    "FLIGHT_NOT_FOUND".to_string(),
                    json!({}),
                )
            }
            AppError::DuplicateFlight => {
                tracing::warn!(
                    error_type = "DuplicateFlight",
                    "Attempted to create duplicate flight"
                );
                (
                    StatusCode::CONFLICT,
                    "Flight number already exists for that date".to_string(),
                    "DUPLICATE_FLIGHT".to_string(),
                    json!({}),
                )
            }
            AppError::InvalidDepartureTime => {
                tracing::warn!(
                    error_type = "InvalidDepartureTime",
                    "Invalid departure time provided"
                );
                (
                    StatusCode::BAD_REQUEST,
                    "Departure time cannot be in the past".to_string(),
                    "INVALID_DEPARTURE_TIME".to_string(),
                    json!({}),
                )
            }
            AppError::InvalidBarcodeFormat => {
                tracing::warn!(
                    error_type = "InvalidBarcodeFormat",
                    "Invalid barcode format received"
                );
                (
                    StatusCode::BAD_REQUEST,
                    "Invalid barcode format for IATA decoding".to_string(),
                    "INVALID_BARCODE_FORMAT".to_string(),
                    json!({}),
                )
            }
        };

        let body = Json(json!({
            "status": "error",
            "message": error_message,
            "code": error_code,
            "details": details
        }));

        (status, body).into_response()
    }
}
