use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize, Deserializer};
use validator::Validate;

// Custom deserializer untuk DateTime yang lebih fleksibel
fn deserialize_flexible_datetime<'de, D>(deserializer: D) -> Result<DateTime<Utc>, D::Error>
where
    D: Deserializer<'de>,
{
    let s: String = Deserialize::deserialize(deserializer)?;

    // Try parsing dengan berbagai format
    // Format 1: ISO 8601 dengan milliseconds (2025-09-30T07:58:00.000)
    if let Ok(dt) = chrono::DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.3f") {
        return Ok(dt.with_timezone(&Utc));
    }

    // Format 2: ISO 8601 dengan timezone (2025-09-30T07:58:00.000Z)
    if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&s) {
        return Ok(dt.with_timezone(&Utc));
    }

    // Format 3: ISO 8601 tanpa milliseconds (2025-09-30T07:58:00)
    if let Ok(dt) = chrono::DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(dt.with_timezone(&Utc));
    }

    // Format 4: Naive datetime tanpa timezone, assume UTC
    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.3f") {
        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc));
    }

    if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S") {
        return Ok(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc));
    }

    Err(serde::de::Error::custom(format!(
        "Failed to parse datetime: {}. Expected format: '2025-09-30T07:58:00.000' or '2025-09-30T07:58:00.000Z'",
        s
    )))
}

// Custom deserializer untuk optional DateTime
fn deserialize_optional_flexible_datetime<'de, D>(
    deserializer: D,
) -> Result<Option<DateTime<Utc>>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(s) => {
            // Reuse logic from deserialize_flexible_datetime
            if let Ok(dt) = chrono::DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.3f") {
                return Ok(Some(dt.with_timezone(&Utc)));
            }
            if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&s) {
                return Ok(Some(dt.with_timezone(&Utc)));
            }
            if let Ok(dt) = chrono::DateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S") {
                return Ok(Some(dt.with_timezone(&Utc)));
            }
            if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S%.3f")
            {
                return Ok(Some(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)));
            }
            if let Ok(naive) = chrono::NaiveDateTime::parse_from_str(&s, "%Y-%m-%dT%H:%M:%S") {
                return Ok(Some(DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc)));
            }
            Err(serde::de::Error::custom(format!(
                "Failed to parse datetime: {}",
                s
            )))
        }
        None => Ok(None),
    }
}

// Model utama untuk tabel `flights` yang sesuai dengan skema database
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Flight {
    pub id: i32,
    pub flight_number: String,
    pub airline: String,
    pub aircraft: String,
    pub departure_time: DateTime<Utc>,
    pub destination: String,
    pub gate: String,
    pub is_active: Option<bool>, // Make nullable for SQLX compatibility
    pub created_at: DateTime<Utc>,
    pub updated_at: Option<DateTime<Utc>>,
    pub device_id: Option<String>, // Ditambahkan karena ada di database dan bisa NULL
}

// Model untuk membuat penerbangan baru (Request Body)
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct CreateFlight {
    #[validate(length(min = 3, max = 10))]
    pub flight_number: String,
    #[validate(length(min = 2, max = 100))]
    pub airline: String,
    #[validate(length(min = 2, max = 50))]
    pub aircraft: String,
    #[serde(deserialize_with = "deserialize_flexible_datetime")]
    pub departure_time: DateTime<Utc>,
    #[validate(length(equal = 3))]
    pub destination: String,
    #[validate(regex(
        path = "*crate::models::GATE_REGEX", // Dereferensi untuk validator
        message = "Gate format must be A1-Z99"
    ))]
    pub gate: String,
    pub device_id: Option<String>,
}

// Model untuk memperbarui penerbangan (Request Body)
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct UpdateFlight {
    #[validate(length(min = 2, max = 100))]
    pub airline: Option<String>,
    #[validate(length(min = 2, max = 50))]
    pub aircraft: Option<String>,
    #[serde(
        default,
        deserialize_with = "deserialize_optional_flexible_datetime"
    )]
    pub departure_time: Option<DateTime<Utc>>,
    #[validate(length(equal = 3))]
    pub destination: Option<String>,
    #[validate(regex(
        path = "*crate::models::GATE_REGEX", // Dereferensi untuk validator
        message = "Gate format must be A1-Z99"
    ))]
    pub gate: Option<String>,
    pub is_active: Option<bool>,
}

// Custom deserializer untuk i32 yang fleksibel (menerima string atau number)
fn deserialize_flexible_i32<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::{self, Deserialize};
    use serde_json::Value;

    let value = Value::deserialize(deserializer)?;
    match value {
        Value::Number(num) => num
            .as_i64()
            .and_then(|n| i32::try_from(n).ok())
            .ok_or_else(|| de::Error::custom("Invalid i32 number")),
        Value::String(s) => s
            .parse::<i32>()
            .map_err(|_| de::Error::custom(format!("Cannot parse '{}' as i32", s))),
        _ => Err(de::Error::custom("Expected number or string for i32")),
    }
}

// Struct DIPISAH: Satu untuk input dari user (ScanDataInput)...
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ScanDataInput {
    #[validate(length(min = 1))]
    pub barcode_value: String,
    #[validate(length(min = 1))]
    pub barcode_format: String,
    #[validate(length(min = 1))]
    pub device_id: String,
    #[serde(deserialize_with = "deserialize_flexible_i32")]
    pub flight_id: i32,
    // Note: confidenceScore dari request akan diabaikan karena tidak ada di struct
}

// ...dan satu lagi untuk representasi data di database (ScanData)
#[derive(Debug, Serialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ScanData {
    pub id: i32,
    pub barcode_value: String,
    pub barcode_format: String,
    pub scan_time: DateTime<Utc>,
    pub device_id: String,
    pub flight_id: Option<i32>, // Sesuai skema ON DELETE SET NULL
    pub created_at: DateTime<Utc>,
}


// Struktur untuk parameter query di GET /api/flights
#[derive(Debug, Deserialize)]
pub struct GetFlightsQuery {
    pub date: Option<chrono::NaiveDate>,
}

// Struktur untuk parameter query di GET /api/scan-data
#[derive(Debug, Deserialize)]
pub struct GetScanDataQuery {
    pub flight_id: Option<i32>,
    pub date_range: Option<String>, // "start,end" format
}

// Struktur untuk parameter query di GET /api/decoded-barcodes
#[derive(Debug, Deserialize)]
pub struct GetDecodedBarcodesQuery {
    pub flight_id: Option<i32>,
}

// Struktur untuk parameter query di GET /api/sync/flights
#[derive(Debug, Deserialize)]
pub struct SyncFlightsQuery {
    pub last_sync: Option<DateTime<Utc>>,
}

// Struktur untuk response statistik
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlightStatistics {
    pub flight_id: i32,
    pub flight_number: String,
    pub total_scans: i64,
    pub unique_scans: i64,
    pub duplicate_scans: i64,
    pub scans_by_hour: Vec<ScansByHour>,
    pub top_devices: Vec<TopDevice>,
}

// Struktur untuk response decoded barcode statistics
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecodedStatistics {
    pub flight_id: i32,
    pub flight_number: String,
    pub total_decoded: i64,
    pub valid_tickets: i64,
    pub invalid_tickets: i64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct ScansByHour {
    pub hour: String,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TopDevice {
    pub device_id: String,
    pub scan_count: i64,
}

// Format response API standar
#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total: Option<u64>,
}

// Model untuk tabel decode_barcode (sesuai dengan decode.json)
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub struct DecodedBarcode {
    pub id: i32,
    pub barcode_value: String,
    pub passenger_name: String,
    pub booking_code: String,
    pub origin: String,
    pub destination: String,
    pub airline_code: String,
    pub flight_number: i32,  // Integer sesuai decode.json
    pub flight_date_julian: String,
    pub cabin_class: String,
    pub seat_number: String,
    pub sequence_number: String,
    pub ticket_status: String,
    pub scan_data_id: Option<i32>,
    pub created_at: DateTime<Utc>,
}

// Model untuk input decode barcode
#[derive(Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct DecodeRequest {
    #[validate(length(min = 1))]
    pub barcode_value: String,
    pub scan_data_id: Option<i32>,
}

// Regex untuk validasi format gate
lazy_static::lazy_static! {
    pub static ref GATE_REGEX: regex::Regex = regex::Regex::new(r"^[A-Z]\d{1,2}$").unwrap();
}