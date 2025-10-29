use crate::{
    database,
    database_config::health_check,
    errors::AppError,
    models::{
        ApiResponse, CreateFlight, ScanDataInput, ScanData, Flight, FlightStatistics, GetFlightsQuery,
        GetScanDataQuery, SyncFlightsQuery, UpdateFlight, DecodedBarcode, DecodeRequest,
        GetDecodedBarcodesQuery, DecodedStatistics, CreateRejectionLog, RejectionLog, RejectionLogQuery,
        AirportCode, AirlineCode, CabinClassCode,
    },
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;
use validator::Validate;

// ==================== FLIGHT MANAGEMENT HANDLERS ====================

/// Create a new flight
#[utoipa::path(
    post,
    path = "/api/flights",
    tag = "Flights",
    request_body = CreateFlight,
    responses(
        (status = 201, description = "Flight created successfully", body = Flight),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal server error")
    )
)]
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

/// Get all flights with optional date filter
#[utoipa::path(
    get,
    path = "/api/flights",
    tag = "Flights",
    params(
        ("date" = Option<String>, Query, description = "Filter by date (YYYY-MM-DD)")
    ),
    responses(
        (status = 200, description = "List of flights", body = Vec<Flight>),
        (status = 500, description = "Internal server error")
    )
)]
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

/// Get flight by ID
#[utoipa::path(
    get,
    path = "/api/flights/{id}",
    tag = "Flights",
    params(
        ("id" = i32, Path, description = "Flight ID")
    ),
    responses(
        (status = 200, description = "Flight details", body = Flight),
        (status = 404, description = "Flight not found"),
        (status = 500, description = "Internal server error")
    )
)]
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

/// Update flight by ID
#[utoipa::path(
    put,
    path = "/api/flights/{id}",
    tag = "Flights",
    params(
        ("id" = i32, Path, description = "Flight ID")
    ),
    request_body = UpdateFlight,
    responses(
        (status = 200, description = "Flight updated successfully", body = Flight),
        (status = 400, description = "Validation error"),
        (status = 404, description = "Flight not found"),
        (status = 500, description = "Internal server error")
    )
)]
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

/// Delete flight by ID (soft delete)
#[utoipa::path(
    delete,
    path = "/api/flights/{id}",
    tag = "Flights",
    params(
        ("id" = i32, Path, description = "Flight ID")
    ),
    responses(
        (status = 204, description = "Flight deleted successfully"),
        (status = 404, description = "Flight not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_flight(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    database::delete_flight(&pool, id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// Get flight scan statistics
#[utoipa::path(
    get,
    path = "/api/flights/{id}/statistics",
    tag = "Flights",
    params(
        ("id" = i32, Path, description = "Flight ID")
    ),
    responses(
        (status = 200, description = "Flight scan statistics", body = FlightStatistics),
        (status = 404, description = "Flight not found"),
        (status = 500, description = "Internal server error")
    )
)]
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

/// Get decoded barcode statistics for flight
#[utoipa::path(
    get,
    path = "/api/flights/{id}/decoded-statistics",
    tag = "Flights",
    params(
        ("id" = i32, Path, description = "Flight ID")
    ),
    responses(
        (status = 200, description = "Decoded barcode statistics", body = DecodedStatistics),
        (status = 404, description = "Flight not found"),
        (status = 500, description = "Internal server error")
    )
)]
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

// ==================== SCANNING HANDLERS ====================

/// Create new scan data
#[utoipa::path(
    post,
    path = "/api/scan-data",
    tag = "Scanning",
    request_body = ScanDataInput,
    responses(
        (status = 201, description = "Scan data created successfully", body = ScanData),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal server error")
    )
)]
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

/// Get scan data with filters
#[utoipa::path(
    get,
    path = "/api/scan-data",
    tag = "Scanning",
    params(
        ("flight_id" = Option<i32>, Query, description = "Filter by flight ID"),
        ("date_range" = Option<String>, Query, description = "Date range filter (start,end)")
    ),
    responses(
        (status = 200, description = "List of scan data", body = Vec<ScanData>),
        (status = 500, description = "Internal server error")
    )
)]
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

/// Decode barcode (IATA BCBP format)
#[utoipa::path(
    post,
    path = "/api/decode-barcode",
    tag = "Scanning",
    request_body = DecodeRequest,
    responses(
        (status = 201, description = "Barcode decoded successfully", body = DecodedBarcode),
        (status = 400, description = "Invalid barcode format"),
        (status = 500, description = "Internal server error")
    )
)]
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

/// Get all decoded barcodes with optional flight filter
#[utoipa::path(
    get,
    path = "/api/decoded-barcodes",
    tag = "Scanning",
    params(
        ("flight_id" = Option<i32>, Query, description = "Filter by flight ID")
    ),
    responses(
        (status = 200, description = "List of decoded barcodes", body = Vec<DecodedBarcode>),
        (status = 500, description = "Internal server error")
    )
)]
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

// ==================== SYNC HANDLERS ====================

/// Incremental flight synchronization
#[utoipa::path(
    get,
    path = "/api/sync/flights",
    tag = "Sync",
    params(
        ("last_sync" = Option<String>, Query, description = "Last sync timestamp (ISO 8601)")
    ),
    responses(
        (status = 200, description = "Updated flights since last sync", body = Vec<Flight>),
        (status = 500, description = "Internal server error")
    )
)]
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

/// Bulk flight synchronization
#[utoipa::path(
    post,
    path = "/api/sync/flights/bulk",
    tag = "Sync",
    request_body = Vec<CreateFlight>,
    responses(
        (status = 201, description = "Flights synced successfully"),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal server error")
    )
)]
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

// ==================== REJECTION LOGGING HANDLERS ====================

/// Create rejection log
#[utoipa::path(
    post,
    path = "/api/rejection-logs",
    tag = "Logs",
    request_body = CreateRejectionLog,
    responses(
        (status = 201, description = "Rejection log created successfully", body = RejectionLog),
        (status = 400, description = "Validation error"),
        (status = 500, description = "Internal server error")
    )
)]
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

/// Get rejection logs with filtering
#[utoipa::path(
    get,
    path = "/api/rejection-logs",
    tag = "Logs",
    params(
        ("limit" = Option<i64>, Query, description = "Limit number of results"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination"),
        ("airline" = Option<String>, Query, description = "Filter by airline code"),
        ("reason" = Option<String>, Query, description = "Filter by rejection reason"),
        ("device_id" = Option<String>, Query, description = "Filter by device ID")
    ),
    responses(
        (status = 200, description = "List of rejection logs", body = Vec<RejectionLog>),
        (status = 500, description = "Internal server error")
    )
)]
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

/// Get rejection statistics
#[utoipa::path(
    get,
    path = "/api/rejection-logs/stats",
    tag = "Logs",
    responses(
        (status = 200, description = "Rejection statistics"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_rejection_stats(
    State(pool): State<PgPool>,
) -> Result<Json<serde_json::Value>, AppError> {
    let stats = database::get_rejection_stats(&pool).await?;
    Ok(Json(stats))
}

// ==================== CODE TRANSLATION HANDLERS ====================

/// Get airport codes
#[utoipa::path(
    get,
    path = "/api/codes/airports",
    tag = "Codes",
    responses(
        (status = 200, description = "List of airport codes", body = Vec<AirportCode>),
        (status = 500, description = "Internal server error")
    )
)]
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

/// Get airline codes
#[utoipa::path(
    get,
    path = "/api/codes/airlines",
    tag = "Codes",
    responses(
        (status = 200, description = "List of airline codes", body = Vec<AirlineCode>),
        (status = 500, description = "Internal server error")
    )
)]
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

/// Get cabin class codes
#[utoipa::path(
    get,
    path = "/api/codes/classes",
    tag = "Codes",
    responses(
        (status = 200, description = "List of cabin class codes", body = Vec<CabinClassCode>),
        (status = 500, description = "Internal server error")
    )
)]
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

/// Get starter data version
#[utoipa::path(
    get,
    path = "/api/starter-data/version",
    tag = "Codes",
    responses(
        (status = 200, description = "Starter data version"),
        (status = 500, description = "Internal server error")
    )
)]
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

// ==================== HEALTH CHECK HANDLER ====================

/// Health check endpoint
///
/// Returns system health status including:
/// - Database connectivity
/// - Response time
/// - Connection pool status
/// - System uptime
///
/// This endpoint does not require authentication
/// and can be used by monitoring systems.
pub async fn health_check(
    State(pool): State<PgPool>,
) -> Result<(StatusCode, Json<serde_json::Value>), AppError> {
    let health_info = health_check(&pool).await;
    let status_code = StatusCode::from_u16(health_info.status_code())
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);

    let response = serde_json::json!({
        "status": if health_info.is_healthy { "healthy" } else { "unhealthy" },
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "database": {
            "is_healthy": health_info.is_healthy,
            "response_time_ms": health_info.response_time.as_millis(),
            "active_connections": health_info.active_connections,
            "idle_connections": health_info.idle_connections
        },
        "api": {
            "version": env!("CARGO_PKG_VERSION", "unknown"),
            "environment": std::env::var("ENVIRONMENT").unwrap_or_else(|_| "unknown".to_string())
        },
        "error": health_info.error
    });

    Ok((status_code, Json(response)))
}
