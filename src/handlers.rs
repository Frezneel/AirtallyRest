use crate::{
    database,
    errors::AppError,
    models::{
        ApiResponse, CreateFlight, ScanDataInput, ScanData, Flight, FlightStatistics, GetFlightsQuery,
        GetScanDataQuery, SyncFlightsQuery, UpdateFlight, DecodedBarcode, DecodeRequest,
        GetDecodedBarcodesQuery, DecodedStatistics, CreateRejectionLog, RejectionLog, RejectionLogQuery,
    },
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use validator::Validate;

// Handler untuk membuat penerbangan baru
pub async fn create_flight(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateFlight>,
) -> Result<(StatusCode, Json<ApiResponse<Flight>>), AppError> {
    tracing::info!(
        flight_number = %payload.flight_number,
        airline = %payload.airline,
        destination = %payload.destination,
        "Creating new flight"
    );

    if let Err(validation_errors) = payload.validate() {
        tracing::error!(
            errors = ?validation_errors.field_errors(),
            payload = ?payload,
            "Flight validation failed"
        );
        return Err(AppError::ValidationError(validation_errors));
    }

    let new_flight = database::create_flight(&pool, payload).await?;

    tracing::info!(
        flight_id = new_flight.id,
        flight_number = %new_flight.flight_number,
        "Flight created successfully"
    );

    let response = ApiResponse {
        status: "success".to_string(),
        message: Some("Flight created successfully".to_string()),
        data: Some(new_flight),
        total: None,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

// Handler untuk mendapatkan daftar penerbangan (dengan filter tanggal opsional)
pub async fn get_flights(
    State(pool): State<PgPool>,
    Query(query): Query<GetFlightsQuery>,
) -> Result<Json<ApiResponse<Vec<Flight>>>, AppError> {
    let (flights, total) = database::get_all_flights(&pool, query.date).await?;
    let response = ApiResponse {
        status: "success".to_string(),
        message: None,
        data: Some(flights),
        total: Some(total as u64),
    };
    Ok(Json(response))
}

// Handler untuk mendapatkan penerbangan berdasarkan ID
pub async fn get_flight_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<Flight>>, AppError> {
    let flight = database::get_flight_by_id(&pool, id).await?;
    let response = ApiResponse {
        status: "success".to_string(),
        message: None,
        data: Some(flight),
        total: None,
    };
    Ok(Json(response))
}

// Handler untuk memperbarui penerbangan
pub async fn update_flight(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateFlight>,
) -> Result<Json<ApiResponse<Flight>>, AppError> {
    payload.validate()?;
    let updated_flight = database::update_flight(&pool, id, payload).await?;
    let response = ApiResponse {
        status: "success".to_string(),
        message: Some("Flight updated successfully".to_string()),
        data: Some(updated_flight),
        total: None,
    };
    Ok(Json(response))
}

// Handler untuk menghapus penerbangan (soft delete)
pub async fn delete_flight(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    database::delete_flight(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// Handler untuk mendapatkan statistik scan penerbangan
pub async fn get_flight_statistics(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<FlightStatistics>>, AppError> {
    let stats = database::get_flight_statistics(&pool, id).await?;
    let response = ApiResponse {
        status: "success".to_string(),
        message: None,
        data: Some(stats),
        total: None,
    };
    Ok(Json(response))
}

// Handler untuk mendapatkan statistik decoded barcodes per penerbangan
pub async fn get_decoded_statistics(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<DecodedStatistics>>, AppError> {
    let stats = database::get_decoded_statistics(&pool, id).await?;
    let response = ApiResponse {
        status: "success".to_string(),
        message: None,
        data: Some(stats),
        total: None,
    };
    Ok(Json(response))
}

// Handler untuk membuat data scan baru
pub async fn create_scan(
    State(pool): State<PgPool>,
    Json(payload): Json<ScanDataInput>,
) -> Result<(StatusCode, Json<ApiResponse<ScanData>>), AppError> {
    tracing::info!(
        flight_id = payload.flight_id,
        barcode_format = %payload.barcode_format,
        "Creating new scan data"
    );

    if let Err(validation_errors) = payload.validate() {
        tracing::error!(
            errors = ?validation_errors.field_errors(),
            payload = ?payload,
            "Scan data validation failed"
        );
        return Err(AppError::ValidationError(validation_errors));
    }

    let new_scan = database::create_scan_data(&pool, payload).await?;

    tracing::info!(
        scan_id = new_scan.id,
        flight_id = new_scan.flight_id,
        "Scan data created successfully"
    );

    let response = ApiResponse {
        status: "success".to_string(),
        message: Some("Scan data saved successfully".to_string()),
        data: Some(new_scan),
        total: None,
    };
    Ok((StatusCode::CREATED, Json(response)))
}

// Handler untuk mendapatkan data scan dengan filter
pub async fn get_scan_data(
    State(pool): State<PgPool>,
    Query(query): Query<GetScanDataQuery>,
) -> Result<Json<ApiResponse<Vec<ScanData>>>, AppError> {
    let (scans, total) = database::get_scan_data(&pool, query).await?;
    let response = ApiResponse {
        status: "success".to_string(),
        message: None,
        data: Some(scans),
        total: Some(total as u64),
    };
    Ok(Json(response))
}

// Handler untuk sinkronisasi incremental
pub async fn sync_flights(
    State(pool): State<PgPool>,
    Query(query): Query<SyncFlightsQuery>,
) -> Result<Json<ApiResponse<Vec<Flight>>>, AppError> {
    let flights = database::get_flights_since(&pool, query.last_sync).await?;
    let response = ApiResponse {
        status: "success".to_string(),
        message: None,
        data: Some(flights),
        total: None,
    };
    Ok(Json(response))
}

// Handler untuk sinkronisasi bulk
pub async fn sync_flights_bulk(
    State(pool): State<PgPool>,
    Json(payload): Json<Vec<CreateFlight>>,
) -> Result<(StatusCode, Json<ApiResponse<usize>>), AppError> {
    tracing::info!(
        flight_count = payload.len(),
        "Bulk sync flights request"
    );

    for (index, p) in payload.iter().enumerate() {
        if let Err(validation_errors) = p.validate() {
            tracing::error!(
                index = index,
                errors = ?validation_errors.field_errors(),
                flight = ?p,
                "Bulk sync validation failed"
            );
            return Err(AppError::ValidationError(validation_errors));
        }
    }

    let count = database::bulk_insert_flights(&pool, payload).await?;

    tracing::info!(
        synced_count = count,
        "Bulk flights synced successfully"
    );

    let response = ApiResponse {
        status: "success".to_string(),
        message: Some(format!("{} flights synced successfully", count)),
        data: Some(count),
        total: None,
    };
    Ok((StatusCode::CREATED, Json(response)))
}

// Handler untuk decode barcode IATA
pub async fn decode_barcode(
    State(pool): State<PgPool>,
    Json(payload): Json<DecodeRequest>,
) -> Result<(StatusCode, Json<ApiResponse<DecodedBarcode>>), AppError> {
    payload.validate()?;
    let decoded = database::decode_barcode_iata(&pool, payload).await?;
    let response = ApiResponse {
        status: "success".to_string(),
        message: Some("Barcode decoded successfully".to_string()),
        data: Some(decoded),
        total: None,
    };
    Ok((StatusCode::CREATED, Json(response)))
}

// Handler untuk mendapatkan semua decoded barcodes
pub async fn get_decoded_barcodes(
    State(pool): State<PgPool>,
    Query(query): Query<GetDecodedBarcodesQuery>,
) -> Result<Json<ApiResponse<Vec<DecodedBarcode>>>, AppError> {
    let decoded_list = database::get_all_decoded_barcodes(&pool, query.flight_id).await?;
    let response = ApiResponse {
        status: "success".to_string(),
        message: None,
        data: Some(decoded_list),
        total: None,
    };
    Ok(Json(response))
}

// ==================== REJECTION LOGGING HANDLERS ====================

// Handler untuk membuat rejection log baru
pub async fn create_rejection_log(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateRejectionLog>,
) -> Result<(StatusCode, Json<ApiResponse<RejectionLog>>), AppError> {
    tracing::info!(
        barcode_format = %payload.barcode_format,
        reason = %payload.reason,
        airline = ?payload.airline,
        "Creating rejection log"
    );

    if let Err(validation_errors) = payload.validate() {
        tracing::error!(
            errors = ?validation_errors.field_errors(),
            "Rejection log validation failed"
        );
        return Err(AppError::ValidationError(validation_errors));
    }

    let rejection = database::create_rejection_log(&pool, payload).await?;

    tracing::info!(
        rejection_id = rejection.id,
        "Rejection log created successfully"
    );

    let response = ApiResponse {
        status: "success".to_string(),
        message: Some("Rejection log saved successfully".to_string()),
        data: Some(rejection),
        total: None,
    };
    Ok((StatusCode::CREATED, Json(response)))
}

// Handler untuk mendapatkan rejection logs dengan filtering
pub async fn get_rejection_logs(
    State(pool): State<PgPool>,
    Query(query): Query<RejectionLogQuery>,
) -> Result<Json<ApiResponse<Vec<RejectionLog>>>, AppError> {
    let logs = database::get_rejection_logs(&pool, query).await?;
    let response = ApiResponse {
        status: "success".to_string(),
        message: None,
        data: Some(logs),
        total: None,
    };
    Ok(Json(response))
}

// Handler untuk mendapatkan rejection statistics
pub async fn get_rejection_stats(
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, AppError> {
    let stats = database::get_rejection_stats(&pool).await?;
    Ok(Json(stats))
}

// ============= Translation/Code Mapping Handlers =============

// Handler untuk mendapatkan airport codes
pub async fn get_airport_codes(
    State(pool): State<PgPool>,
) -> Result<Json<ApiResponse<Vec<crate::models::AirportCode>>>, AppError> {
    let codes = database::get_airport_codes(&pool).await?;
    let response = ApiResponse {
        status: "success".to_string(),
        message: None,
        data: Some(codes),
        total: None,
    };
    Ok(Json(response))
}

// Handler untuk mendapatkan airline codes
pub async fn get_airline_codes(
    State(pool): State<PgPool>,
) -> Result<Json<ApiResponse<Vec<crate::models::AirlineCode>>>, AppError> {
    let codes = database::get_airline_codes(&pool).await?;
    let response = ApiResponse {
        status: "success".to_string(),
        message: None,
        data: Some(codes),
        total: None,
    };
    Ok(Json(response))
}

// Handler untuk mendapatkan cabin class codes
pub async fn get_cabin_class_codes(
    State(pool): State<PgPool>,
) -> Result<Json<ApiResponse<Vec<crate::models::CabinClassCode>>>, AppError> {
    let codes = database::get_cabin_class_codes(&pool).await?;
    let response = ApiResponse {
        status: "success".to_string(),
        message: None,
        data: Some(codes),
        total: None,
    };
    Ok(Json(response))
}

// Handler untuk mendapatkan passenger status codes
pub async fn get_passenger_status_codes(
    State(pool): State<PgPool>,
) -> Result<Json<ApiResponse<Vec<crate::models::PassengerStatusCode>>>, AppError> {
    let codes = database::get_passenger_status_codes(&pool).await?;
    let response = ApiResponse {
        status: "success".to_string(),
        message: None,
        data: Some(codes),
        total: None,
    };
    Ok(Json(response))
}

// Handler untuk mendapatkan starter data version
pub async fn get_starter_data_version(
    State(pool): State<PgPool>,
) -> Result<Json<ApiResponse<crate::models::StarterDataVersion>>, AppError> {
    let version = database::get_starter_data_version(&pool).await?;
    let response = ApiResponse {
        status: "success".to_string(),
        message: None,
        data: Some(version),
        total: None,
    };
    Ok(Json(response))
}
