use crate::{
    errors::AppError,
    models::{
        CreateFlight, Flight, FlightStatistics, GetScanDataQuery, ScanData, ScanDataInput,
        ScansByHour, TopDevice, UpdateFlight, DecodedBarcode, DecodeRequest,
    },
};
use chrono::{DateTime, NaiveDate, Utc};
use sqlx::PgPool;

// NOTE: Barcode decoder functions - Uncomment setelah running migration decode_barcode

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