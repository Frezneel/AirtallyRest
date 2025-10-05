use crate::{
    errors::AppError,
    models::{
        CreateFlight, Flight, FlightStatistics, GetScanDataQuery, ScanData, ScanDataInput,
        ScansByHour, TopDevice, UpdateFlight, DecodedBarcode, DecodeRequest,
    },
    barcode_parser,
};
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;

// NOTE: Barcode decoder functions - Uncomment setelah running migration decode_barcode

// Fungsi untuk decode barcode IATA format
// Uses shared parser module synchronized with mobile app
pub async fn decode_barcode_iata(
    pool: &PgPool,
    request: DecodeRequest,
) -> Result<DecodedBarcode, AppError> {
    // Use shared parser (synchronized with mobile app)
    let parsed = barcode_parser::parse_iata_bcbp(&request.barcode_value)
        .ok_or(AppError::InvalidBarcodeFormat)?;

    // Extract data from parsed result
    let passenger_name = parsed.passenger_name;
    let booking_code = parsed.booking_code;
    let origin = parsed.origin;
    let destination = parsed.destination;
    let airline_code = parsed.airline_code;
    let flight_number = parsed.flight_number.parse::<i32>().unwrap_or(0);
    let flight_date_julian = parsed.flight_date_julian;
    let cabin_class = parsed.cabin_class;
    let seat_number = parsed.seat_number;
    let sequence_number = parsed.sequence_number;
    let ticket_status = parsed.passenger_status;

    let decoded = sqlx::query_as!(
        DecodedBarcode,
        r#"
        INSERT INTO decode_barcode
        (barcode_value, passenger_name, booking_code, origin, destination, airline_code,
         flight_number, flight_date_julian, cabin_class, seat_number, sequence_number,
         ticket_status, scan_data_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
        RETURNING id, barcode_value, passenger_name, booking_code, origin, destination,
                  airline_code, flight_number, flight_date_julian, cabin_class, seat_number,
                  sequence_number, ticket_status, scan_data_id, created_at
        "#,
        request.barcode_value,
        passenger_name,
        booking_code,
        origin,
        destination,
        airline_code,
        flight_number,
        flight_date_julian,
        cabin_class,
        seat_number,
        sequence_number,
        ticket_status,
        request.scan_data_id
    )
    .fetch_one(pool)
    .await?;

    Ok(decoded)
}

// Fungsi untuk mengambil semua decoded barcodes
pub async fn get_all_decoded_barcodes(pool: &PgPool) -> Result<Vec<DecodedBarcode>, AppError> {
    let decoded_list = sqlx::query_as!(
        DecodedBarcode,
        r#"
        SELECT id, barcode_value, passenger_name, booking_code, origin, destination,
               airline_code, flight_number, flight_date_julian, cabin_class, seat_number,
               sequence_number, ticket_status, scan_data_id, created_at
        FROM decode_barcode
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(decoded_list)
}

// NOTE: All parsing logic has been moved to shared barcode_parser module
// This ensures 100% synchronization between mobile app and server

// ==================== REJECTION LOGGING FUNCTIONS ====================

/// Create a rejection log entry in server database
pub async fn create_rejection_log(
    pool: &PgPool,
    log: crate::models::CreateRejectionLog,
) -> Result<crate::models::RejectionLog, AppError> {
    let rejection = sqlx::query_as!(
        crate::models::RejectionLog,
        r#"
        INSERT INTO rejection_logs
        (barcode_value, barcode_format, reason, expected_date, actual_date,
         flight_number, airline, device_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, barcode_value, barcode_format, reason, expected_date, actual_date,
                  flight_number, airline, device_id, rejected_at
        "#,
        log.barcode_value,
        log.barcode_format,
        log.reason,
        log.expected_date,
        log.actual_date,
        log.flight_number,
        log.airline,
        log.device_id
    )
    .fetch_one(pool)
    .await?;

    Ok(rejection)
}

/// Get rejection logs with optional filtering
pub async fn get_rejection_logs(
    pool: &PgPool,
    query: crate::models::RejectionLogQuery,
) -> Result<Vec<crate::models::RejectionLog>, AppError> {
    let limit = query.limit.unwrap_or(100);
    let offset = query.offset.unwrap_or(0);

    let mut query_builder = String::from(
        "SELECT id, barcode_value, barcode_format, reason, expected_date, actual_date,
                flight_number, airline, device_id, rejected_at
         FROM rejection_logs
         WHERE 1=1"
    );

    // Add filters
    if query.airline.is_some() {
        query_builder.push_str(" AND airline = $1");
    }
    if query.reason.is_some() {
        query_builder.push_str(" AND reason LIKE $2");
    }
    if query.device_id.is_some() {
        query_builder.push_str(" AND device_id = $3");
    }

    query_builder.push_str(" ORDER BY rejected_at DESC LIMIT $4 OFFSET $5");

    // Execute query with parameters
    let logs = if let (Some(airline), Some(reason), Some(device_id)) =
        (&query.airline, &query.reason, &query.device_id) {
        sqlx::query_as::<_, crate::models::RejectionLog>(&query_builder)
            .bind(airline)
            .bind(format!("%{}%", reason))
            .bind(device_id)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
    } else if let (Some(airline), Some(reason)) = (&query.airline, &query.reason) {
        sqlx::query_as::<_, crate::models::RejectionLog>(&query_builder.replace("$3", ""))
            .bind(airline)
            .bind(format!("%{}%", reason))
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
    } else if let Some(airline) = &query.airline {
        sqlx::query_as::<_, crate::models::RejectionLog>(&query_builder.replace("$2", "").replace("$3", ""))
            .bind(airline)
            .bind(limit)
            .bind(offset)
            .fetch_all(pool)
            .await?
    } else {
        sqlx::query_as::<_, crate::models::RejectionLog>(
            "SELECT id, barcode_value, barcode_format, reason, expected_date, actual_date,
                    flight_number, airline, device_id, rejected_at
             FROM rejection_logs
             ORDER BY rejected_at DESC
             LIMIT $1 OFFSET $2"
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(pool)
        .await?
    };

    Ok(logs)
}

/// Get rejection statistics
pub async fn get_rejection_stats(
    pool: &PgPool,
) -> Result<serde_json::Value, AppError> {
    let stats = sqlx::query!(
        r#"
        SELECT
            COUNT(*) as total_rejections,
            COUNT(DISTINCT airline) as airlines_count,
            COUNT(DISTINCT device_id) as devices_count,
            COUNT(CASE WHEN reason LIKE '%date_mismatch%' THEN 1 END) as date_mismatch_count,
            COUNT(CASE WHEN reason LIKE '%invalid_format%' THEN 1 END) as invalid_format_count
        FROM rejection_logs
        WHERE rejected_at >= NOW() - INTERVAL '30 days'
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok(serde_json::json!({
        "totalRejections": stats.total_rejections.unwrap_or(0),
        "airlinesCount": stats.airlines_count.unwrap_or(0),
        "devicesCount": stats.devices_count.unwrap_or(0),
        "dateMismatchCount": stats.date_mismatch_count.unwrap_or(0),
        "invalidFormatCount": stats.invalid_format_count.unwrap_or(0),
    }))
}