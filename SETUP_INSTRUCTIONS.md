# INSTRUKSI SETUP DAN TESTING - AirTally REST API

## 🚀 Setup Langkah demi Langkah

### 1. Prerequisites
- PostgreSQL 13+ (sudah running)
- Rust 1.70+ dengan Edition 2024
- Database `airtally` sudah dibuat

### 2. Setup Database Connection
File `.env` sudah dikonfigurasi:
```
DATABASE_URL="postgres://postgres:1234@localhost:5432/airtally"
```

### 3. First Run (Basic Functionality)
```bash
# Step 1: Jalankan aplikasi pertama kali (migrations akan otomatis berjalan)
cargo run

# Jika berhasil, Anda akan melihat:
# - "Successfully connected to the database"
# - "Database migrations ran successfully"
# - "Server listening on 127.0.0.1:3000"
```

### 4. Test Basic API Endpoints

#### Test Flight Management:
```bash
# 1. Create Flight
curl -X POST http://127.0.0.1:3000/api/flights \
  -H "Content-Type: application/json" \
  -d '{
    "flightNumber": "GA123",
    "airline": "Garuda Indonesia",
    "aircraft": "Boeing 737-800",
    "departureTime": "2024-12-01T14:30:00Z",
    "destination": "CGK",
    "gate": "A5"
  }'

# 2. Get All Flights
curl -X GET http://127.0.0.1:3000/api/flights

# 3. Get Flight by ID
curl -X GET http://127.0.0.1:3000/api/flights/1

# 4. Get Flight Statistics
curl -X GET http://127.0.0.1:3000/api/flights/1/statistics
```

#### Test Scan Data:
```bash
# Create Scan Data
curl -X POST http://127.0.0.1:3000/api/scan-data \
  -H "Content-Type: application/json" \
  -d '{
    "barcodeValue": "ABC123DEF456GHI789",
    "barcodeFormat": "PDF417",
    "deviceId": "device_123456",
    "flightId": 1
  }'

# Get Scan Data
curl -X GET "http://127.0.0.1:3000/api/scan-data?flight_id=1"
```

### 5. Enable Barcode Decoder (Setelah Basic Test Berhasil)

Setelah aplikasi berjalan dan migrations selesai, enable barcode decoder:

#### A. Update database.rs:
```rust
// Uncomment dari file database_decode_functions.rs ke database.rs
// Copy semua content dari database_decode_functions.rs
// Paste di akhir database.rs (setelah line 318)
```

#### B. Update handlers.rs:
```rust
// Uncomment dari file handlers_decode_functions.rs ke handlers.rs
// Copy content dari handlers_decode_functions.rs
// Paste di akhir handlers.rs (setelah line 166)
```

#### C. Update router.rs:
```rust
// Uncomment baris di router.rs:
.route("/api/decode-barcode", post(handlers::decode_barcode))
.route("/api/decoded-barcodes", get(handlers::get_decoded_barcodes))
```

### 6. Test Barcode Decoder (Setelah Enable)

```bash
# Test Decode Barcode
curl -X POST http://127.0.0.1:3000/api/decode-barcode \
  -H "Content-Type: application/json" \
  -d '{
    "barcodeValue": "M1BAYU/MUHAMMAD MR ESMMTHQ DHXCGKID 6473 032Y007A0002 300"
  }'

# Expected Response:
{
  "status": "success",
  "message": "Barcode decoded successfully",
  "data": {
    "id": 1,
    "passenger_name": "MUHAMMAD BAYU",
    "booking_code": "SMMTHQ",
    "origin": "DHX",
    "destination": "CGK",
    "airline_code": "ID",
    "flight_number": 6473,
    "flight_date_julian": "032",
    "cabin_class": "Y",
    "seat_number": "007A",
    "sequence_number": "0002",
    "ticket_status": "E"
  }
}

# Get All Decoded Barcodes
curl -X GET http://127.0.0.1:3000/api/decoded-barcodes
```

## 🐛 Troubleshooting

### Error: "relation does not exist"
- Migration belum jalan atau database belum dibuat
- Pastikan PostgreSQL running dan database 'airtally' ada

### Error: Connection refused
- PostgreSQL service tidak berjalan
- Check DATABASE_URL di .env file

### Error: Authentication failed
- Username/password salah di DATABASE_URL
- Pastikan user 'postgres' dengan password '1234' ada

### Compile Errors tentang unused imports
- Normal jika barcode decoder belum di-enable
- Warnings bisa diabaikan

## 📁 File Structure Final

```
src/
├── main.rs                           # ✅ Entry point
├── router.rs                         # ✅ Routes (decode routes commented)
├── handlers.rs                       # ✅ Handlers (decode handlers commented)
├── database.rs                       # ✅ Database ops (decode functions commented)
├── models.rs                         # ✅ All models including DecodedBarcode
├── errors.rs                         # ✅ Error handling
├── database_decode_functions.rs      # 📄 Ready-to-copy decoder functions
└── handlers_decode_functions.rs      # 📄 Ready-to-copy decoder handlers

migrations/
├── 2025092901_initial_schema.sql     # ✅ Basic tables
└── 2025092902_add_decode_barcode_table.sql  # ✅ Decode table

docs/
└── AirTally_REST_API_Documentation.md  # 📖 Complete documentation
```

## 🎯 Next Steps untuk Production

1. **Enable Barcode Decoder** - Copy functions dari helper files
2. **Add Validation** - Enhance input validation
3. **Add Authentication** - JWT tokens
4. **Add Logging** - Structured logging
5. **Add Tests** - Unit dan integration tests
6. **Docker Setup** - Container deployment
7. **Monitoring** - Health checks dan metrics

## 🔍 Monitoring Commands

```bash
# Check server status
curl -I http://127.0.0.1:3000/api/flights

# Monitor logs
cargo run 2>&1 | grep -E "(ERROR|WARN|INFO)"

# Database check
psql -U postgres -d airtally -c "\dt"
```

---

**Status**: ✅ Basic API siap production
**Barcode Decoder**: 🟡 Siap enable (files tersedia)
**Documentation**: ✅ Lengkap
**Testing**: ✅ Manual testing ready