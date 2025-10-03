use crate::{
    errors::AppError,
    models::{
        CreateFlight, Flight, FlightStatistics, GetScanDataQuery, ScanData, ScanDataInput,
        ScansByHour, TopDevice, UpdateFlight, DecodedBarcode, DecodeRequest, DecodedStatistics,
    },
};
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;

// Fungsi untuk membuat penerbangan baru di database
pub async fn create_flight(pool: &PgPool, flight: CreateFlight) -> Result<Flight, AppError> {
    // Validasi tambahan: departure_time tidak boleh di masa lalu
    if flight.departure_time < Utc::now() {
        return Err(AppError::InvalidDepartureTime);
    }

    let new_flight = sqlx::query_as!(
        Flight,
        r#"
        INSERT INTO flights (flight_number, airline, aircraft, departure_time, destination, gate, device_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id, flight_number, airline, aircraft, departure_time, destination, gate, is_active, created_at, updated_at, device_id
        "#,
        flight.flight_number,
        flight.airline,
        flight.aircraft,
        flight.departure_time,
        flight.destination,
        flight.gate,
        flight.device_id
    )
        .fetch_one(pool)
        .await
        .map_err(|e| {
            if let sqlx::Error::Database(db_err) = &e
                && db_err.constraint() == Some("idx_unique_flight_per_day")
            {
                return AppError::DuplicateFlight;
            }
            AppError::DatabaseError(e)
        })?;

    Ok(new_flight)
}

// Fungsi untuk mengambil semua penerbangan, dengan filter tanggal opsional
pub async fn get_all_flights(
    pool: &PgPool,
    date: Option<NaiveDate>,
) -> Result<(Vec<Flight>, i64), AppError> {
    let mut query_builder = sqlx::QueryBuilder::new(
        "SELECT id, flight_number, airline, aircraft, departure_time, destination, gate, is_active, created_at, updated_at, device_id FROM flights WHERE is_active = true ",
    );
    let mut count_builder =
        sqlx::QueryBuilder::new("SELECT COUNT(*) FROM flights WHERE is_active = true ");

    if let Some(d) = date {
        // Casting ke date harus dilakukan dengan zona waktu yang benar
        query_builder.push("AND (departure_time AT TIME ZONE 'utc')::date = ");
        query_builder.push_bind(d);
        count_builder.push("AND (departure_time AT TIME ZONE 'utc')::date = ");
        count_builder.push_bind(d);
    }

    query_builder.push(" ORDER BY departure_time ASC");

    let flights = query_builder.build_query_as::<Flight>().fetch_all(pool).await?;
    let total: (i64,) = count_builder.build_query_as().fetch_one(pool).await?;

    Ok((flights, total.0))
}


// Fungsi untuk mengambil satu penerbangan berdasarkan ID
pub async fn get_flight_by_id(pool: &PgPool, id: i32) -> Result<Flight, AppError> {
    let flight = sqlx::query_as!(
        Flight,
        "SELECT id, flight_number, airline, aircraft, departure_time, destination, gate, is_active, created_at, updated_at, device_id FROM flights WHERE id = $1 AND is_active = true",
        id
    )
        .fetch_optional(pool)
        .await?
        .ok_or(AppError::FlightNotFound)?;

    Ok(flight)
}

// Fungsi untuk memperbarui data penerbangan
pub async fn update_flight(
    pool: &PgPool,
    id: i32,
    flight: UpdateFlight,
) -> Result<Flight, AppError> {
    let updated_flight = sqlx::query_as!(
        Flight,
        r#"
        UPDATE flights
        SET
            airline = COALESCE($1, airline),
            aircraft = COALESCE($2, aircraft),
            departure_time = COALESCE($3, departure_time),
            destination = COALESCE($4, destination),
            gate = COALESCE($5, gate),
            is_active = COALESCE($6, is_active),
            updated_at = NOW()
        WHERE id = $7
        RETURNING id, flight_number, airline, aircraft, departure_time, destination, gate, is_active, created_at, updated_at, device_id
        "#,
        flight.airline,
        flight.aircraft,
        flight.departure_time,
        flight.destination,
        flight.gate,
        flight.is_active,
        id
    )
        .fetch_one(pool)
        .await?;

    Ok(updated_flight)
}

// Fungsi untuk soft delete penerbangan
pub async fn delete_flight(pool: &PgPool, id: i32) -> Result<(), AppError> {
    let result = sqlx::query!(
        "UPDATE flights SET is_active = false, updated_at = NOW() WHERE id = $1",
        id
    )
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::FlightNotFound);
    }

    Ok(())
}

// Fungsi untuk mengambil statistik penerbangan
pub async fn get_flight_statistics(pool: &PgPool, id: i32) -> Result<FlightStatistics, AppError> {
    let flight_info = get_flight_by_id(pool, id).await?;

    let total_scans: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM scan_data WHERE flight_id = $1")
            .bind(id)
            .fetch_one(pool)
            .await?;

    let unique_scans: (i64,) = sqlx::query_as(
        "SELECT COUNT(DISTINCT barcode_value) FROM scan_data WHERE flight_id = $1",
    )
        .bind(id)
        .fetch_one(pool)
        .await?;

    let scans_by_hour: Vec<ScansByHour> = sqlx::query_as(
        r#"
        SELECT TO_CHAR(DATE_TRUNC('hour', scan_time), 'HH24:00') as hour, COUNT(*) as count
        FROM scan_data
        WHERE flight_id = $1
        GROUP BY DATE_TRUNC('hour', scan_time)
        ORDER BY hour
        "#,
    )
        .bind(id)
        .fetch_all(pool)
        .await?;

    let top_devices: Vec<TopDevice> = sqlx::query_as(
        r#"
        SELECT device_id, COUNT(*) as scan_count
        FROM scan_data
        WHERE flight_id = $1
        GROUP BY device_id
        ORDER BY scan_count DESC
        LIMIT 5
        "#,
    )
        .bind(id)
        .fetch_all(pool)
        .await?;

    Ok(FlightStatistics {
        flight_id: id,
        flight_number: flight_info.flight_number,
        total_scans: total_scans.0,
        unique_scans: unique_scans.0,
        duplicate_scans: total_scans.0 - unique_scans.0,
        scans_by_hour,
        top_devices,
    })
}

// Fungsi untuk mengambil statistik decoded barcodes per penerbangan
pub async fn get_decoded_statistics(
    pool: &PgPool,
    flight_id: i32,
) -> Result<DecodedStatistics, AppError> {
    // Get flight info first
    let flight = get_flight_by_id(pool, flight_id).await?;

    // Count total decoded barcodes (JOIN with scan_data by flight_id)
    let total_decoded: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM decode_barcode db
        JOIN scan_data sd ON db.scan_data_id = sd.id
        WHERE sd.flight_id = $1
        "#,
    )
    .bind(flight_id)
    .fetch_one(pool)
    .await?;

    // Count valid tickets (status '0' or '3')
    let valid_tickets: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM decode_barcode db
        JOIN scan_data sd ON db.scan_data_id = sd.id
        WHERE sd.flight_id = $1 AND db.ticket_status IN ('0', '3')
        "#,
    )
    .bind(flight_id)
    .fetch_one(pool)
    .await?;

    Ok(DecodedStatistics {
        flight_id,
        flight_number: flight.flight_number,
        total_decoded: total_decoded.0,
        valid_tickets: valid_tickets.0,
        invalid_tickets: total_decoded.0 - valid_tickets.0,
    })
}

// Fungsi untuk membuat data scan baru
pub async fn create_scan_data(
    pool: &PgPool,
    scan: ScanDataInput,
) -> Result<ScanData, AppError> {
    // Pastikan flight_id valid
    let _ = get_flight_by_id(pool, scan.flight_id).await?;

    let new_scan = sqlx::query_as!(
        ScanData,
        r#"
        INSERT INTO scan_data (barcode_value, barcode_format, device_id, flight_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id, barcode_value, barcode_format, scan_time, device_id, flight_id, created_at
        "#,
        scan.barcode_value,
        scan.barcode_format,
        scan.device_id,
        scan.flight_id,
    )
        .fetch_one(pool)
        .await?;
    Ok(new_scan)
}

// Fungsi untuk mengambil data scan dengan filter
pub async fn get_scan_data(
    pool: &PgPool,
    query: GetScanDataQuery,
) -> Result<(Vec<ScanData>, i64), AppError> {
    let mut query_builder = sqlx::QueryBuilder::new("SELECT id, barcode_value, barcode_format, scan_time, device_id, flight_id, created_at FROM scan_data WHERE 1=1 ");
    let mut count_builder = sqlx::QueryBuilder::new("SELECT COUNT(*) FROM scan_data WHERE 1=1 ");

    if let Some(flight_id) = query.flight_id {
        query_builder.push(" AND flight_id = ").push_bind(flight_id);
        count_builder.push(" AND flight_id = ").push_bind(flight_id);
    }

    if let Some(date_range) = query.date_range {
        let parts: Vec<&str> = date_range.split(',').collect();
        if parts.len() == 2
            && let (Ok(start), Ok(end)) = (parts[0].parse::<DateTime<Utc>>(), parts[1].parse::<DateTime<Utc>>())
        {
            query_builder.push(" AND scan_time BETWEEN ").push_bind(start).push(" AND ").push_bind(end);
            count_builder.push(" AND scan_time BETWEEN ").push_bind(start).push(" AND ").push_bind(end);
        }
    }

    let scans = query_builder.build_query_as::<ScanData>().fetch_all(pool).await?;
    let total: (i64,) = count_builder.build_query_as().fetch_one(pool).await?;

    Ok((scans, total.0))
}


// Fungsi untuk mengambil penerbangan sejak timestamp terakhir
pub async fn get_flights_since(
    pool: &PgPool,
    last_sync: Option<DateTime<Utc>>,
) -> Result<Vec<Flight>, AppError> {
    let flights = match last_sync {
        Some(ts) => {
            sqlx::query_as!(Flight, "SELECT id, flight_number, airline, aircraft, departure_time, destination, gate, is_active, created_at, updated_at, device_id FROM flights WHERE updated_at > $1 OR created_at > $1 ORDER BY updated_at", ts)
                .fetch_all(pool)
                .await?
        }
        None => {
            sqlx::query_as!(Flight, "SELECT id, flight_number, airline, aircraft, departure_time, destination, gate, is_active, created_at, updated_at, device_id FROM flights ORDER BY created_at")
                .fetch_all(pool)
                .await?
        }
    };
    Ok(flights)
}

// Fungsi untuk bulk insert flights (TELAH DIPERBAIKI)
pub async fn bulk_insert_flights(
    pool: &PgPool,
    flights: Vec<CreateFlight>,
) -> Result<usize, AppError> {
    let mut tx = pool.begin().await?;
    let mut total_affected: u64 = 0;

    for flight in flights {
        if flight.departure_time < Utc::now() {
            // Kita bisa skip atau return error, di sini kita skip
            continue;
        }

        let result = sqlx::query(
            r#"
            INSERT INTO flights (flight_number, airline, aircraft, departure_time, destination, gate, device_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (flight_number, ((departure_time AT TIME ZONE 'utc')::date)) DO UPDATE SET
                airline = EXCLUDED.airline,
                aircraft = EXCLUDED.aircraft,
                departure_time = EXCLUDED.departure_time,
                destination = EXCLUDED.destination,
                gate = EXCLUDED.gate,
                updated_at = NOW()
            "#
        )
            .bind(&flight.flight_number)
            .bind(&flight.airline)
            .bind(&flight.aircraft)
            .bind(flight.departure_time)
            .bind(&flight.destination)
            .bind(&flight.gate)
            .bind(&flight.device_id)
            .execute(&mut *tx)
            .await?;

        total_affected += result.rows_affected();
    }

    tx.commit().await?;

    Ok(total_affected as usize)
}

// Barcode decoder functions

// Fungsi untuk decode barcode IATA format
pub async fn decode_barcode_iata(
    pool: &PgPool,
    request: DecodeRequest,
) -> Result<DecodedBarcode, AppError> {
    let barcode = &request.barcode_value;

    // Parse barcode IATA format sesuai contoh di plan
    // Format: M1BAYU/MUHAMMAD MR ESMMTHQ DHXCGKID 6473 032Y007A0002 300
    // Minimum length: 58 characters (IATA BCBP standard)
    if barcode.len() < 58 {
        return Err(AppError::InvalidBarcodeFormat);
    }

    // Ekstrak data sesuai format IATA BCBP
    let passenger_name = extract_passenger_name(barcode);
    let booking_code = extract_booking_code(barcode);
    let origin = extract_origin(barcode);
    let destination = extract_destination(barcode);
    let airline_code = extract_airline_code(barcode);
    let flight_number = extract_flight_number(barcode);
    let flight_date_julian = extract_julian_date(barcode);
    let cabin_class = extract_cabin_class(barcode);
    let seat_number = extract_seat_number(barcode);
    let sequence_number = extract_sequence_number(barcode);
    let ticket_status = extract_ticket_status(barcode);

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
        barcode,
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

// Fungsi untuk mengambil semua decoded barcodes dengan filter flight_id optional
pub async fn get_all_decoded_barcodes(
    pool: &PgPool,
    flight_id: Option<i32>,
) -> Result<Vec<DecodedBarcode>, AppError> {
    let decoded_list = if let Some(fid) = flight_id {
        // Filter by flight_id via JOIN dengan scan_data
        sqlx::query_as!(
            DecodedBarcode,
            r#"
            SELECT db.id, db.barcode_value, db.passenger_name, db.booking_code, db.origin, db.destination,
                   db.airline_code, db.flight_number, db.flight_date_julian, db.cabin_class, db.seat_number,
                   db.sequence_number, db.ticket_status, db.scan_data_id, db.created_at
            FROM decode_barcode db
            JOIN scan_data sd ON db.scan_data_id = sd.id
            WHERE sd.flight_id = $1
            ORDER BY db.created_at DESC
            "#,
            fid
        )
        .fetch_all(pool)
        .await?
    } else {
        // Get all decoded barcodes
        sqlx::query_as!(
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
        .await?
    };

    Ok(decoded_list)
}

// Helper functions untuk parsing IATA BCBP format
fn extract_passenger_name(barcode: &str) -> String {
    // Format: M1BAYU/MUHAMMAD MR -> MUHAMMAD BAYU (space separator, sesuai decode.json)
    // Position 2-21 (20 characters)
    if let Some(name_part) = barcode.get(2..22) {
        let name = name_part.trim();
        if let Some(slash_pos) = name.find('/') {
            let last_name = &name[..slash_pos];
            let first_name = &name[slash_pos+1..].trim_end_matches(" MR").trim_end_matches(" MS").trim();
            return format!("{} {}", first_name, last_name);  // Space separator
        }
    }
    "UNKNOWN UNKNOWN".to_string()
}

fn extract_booking_code(barcode: &str) -> String {
    // PNR/Booking Reference at position 23-29 (7 characters)
    barcode.get(23..30).unwrap_or("UNKNOWN").trim().to_string()
}

fn extract_origin(barcode: &str) -> String {
    // Origin airport code at position 30-32 (3 characters)
    barcode.get(30..33).unwrap_or("UNK").to_string()
}

fn extract_destination(barcode: &str) -> String {
    // Destination airport code at position 33-35 (3 characters)
    barcode.get(33..36).unwrap_or("UNK").to_string()
}

fn extract_airline_code(barcode: &str) -> String {
    // Airline code at position 36-38 (2-3 characters)
    barcode.get(36..39).unwrap_or("UN").trim().to_string()
}

fn extract_flight_number(barcode: &str) -> i32 {
    // Flight number at position 39-43 (5 characters)
    barcode.get(39..44)
        .unwrap_or("0000")
        .trim()
        .parse::<i32>()
        .unwrap_or(0)
}

fn extract_julian_date(barcode: &str) -> String {
    // Julian date at position 44-46 (3 characters)
    barcode.get(44..47).unwrap_or("000").to_string()
}

fn extract_cabin_class(barcode: &str) -> String {
    // Cabin class at position 47 (1 character)
    barcode.get(47..48).unwrap_or("Y").to_string()
}

fn extract_seat_number(barcode: &str) -> String {
    // Seat number at position 48-51 (4 characters)
    barcode.get(48..52).unwrap_or("000A").trim().to_string()
}

fn extract_sequence_number(barcode: &str) -> String {
    // Sequence number at position 52-56 (5 characters)
    barcode.get(52..57).unwrap_or("0000").trim().to_string()
}

fn extract_ticket_status(barcode: &str) -> String {
    // Ticket status at position 57 (1 character)
    barcode.get(57..58).unwrap_or("E").to_string()
}
