# Dokumentasi AirTally REST API
**Versi: 0.1.0**
**Tanggal: 29 September 2025**
**Stack: Rust + Axum + PostgreSQL**

## Daftar Isi
1. [Ringkasan Proyek](#ringkasan-proyek)
2. [Arsitektur Sistem](#arsitektur-sistem)
3. [Setup dan Instalasi](#setup-dan-instalasi)
4. [Struktur Database](#struktur-database)
5. [API Endpoints](#api-endpoints)
6. [Fitur Barcode Decoder](#fitur-barcode-decoder)
7. [Error Handling](#error-handling)
8. [Testing](#testing)
9. [Deployment](#deployment)
10. [Maintenance](#maintenance)

---

## 1. Ringkasan Proyek

AirTally REST API adalah backend system untuk manajemen penerbangan dan scanning barcode boarding pass. Sistem ini dirancang untuk mendukung aplikasi mobile yang melakukan tracking penumpang pesawat secara real-time.

### Fitur Utama:
- **Manajemen Penerbangan**: CRUD operations untuk data penerbangan
- **Scan Data Management**: Penyimpanan dan analisis data scan barcode
- **Barcode Decoder**: Decode boarding pass IATA format ke data terstruktur
- **Statistik Real-time**: Analytics untuk scan activity per penerbangan
- **Sync Capabilities**: Incremental dan bulk synchronization
- **RESTful API**: Standar HTTP methods dengan JSON response

### Teknologi Stack:
- **Backend**: Rust 2024 Edition dengan Axum web framework
- **Database**: PostgreSQL dengan SQLX untuk database operations
- **Validation**: Validator crate untuk input validation
- **Logging**: Tracing untuk structured logging
- **CORS**: Configured untuk cross-origin requests

---

## 2. Arsitektur Sistem

### Struktur Kode (src/)
```
src/
├── main.rs          # Entry point dan server configuration
├── router.rs        # Route definitions dan middleware setup
├── handlers.rs      # HTTP request handlers
├── database.rs      # Database operations dan business logic
├── models.rs        # Data structures dan validation rules
└── errors.rs        # Error handling dan HTTP response mapping
```

### Database Schema
```
flights ←→ scan_data ←→ decode_barcode
```

### Flow Aplikasi:
1. **Request** → Router → Handler
2. **Handler** → Database Operations
3. **Database** → Business Logic Processing
4. **Response** → JSON API Response

---

## 3. Setup dan Instalasi

### Prasyarat:
- Rust 1.70+ (Edition 2024)
- PostgreSQL 13+
- Git

### Langkah Instalasi:

1. **Clone Repository**
```bash
git clone <repository-url>
cd airtally-restapi
```

2. **Setup Environment**
```bash
cp .env.example .env
# Edit .env dengan konfigurasi database Anda
```

3. **Install Dependencies**
```bash
cargo check  # Verify dependencies
```

4. **Setup Database**
```bash
# Pastikan PostgreSQL running
createdb airtally_db
# Migrations akan otomatis dijalankan saat startup
```

5. **Run Application**
```bash
cargo run
# Server akan berjalan di http://127.0.0.1:3000
```

### Environment Variables (.env):
```
DATABASE_URL=postgresql://username:password@localhost/airtally_db
RUST_LOG=airtally_api=debug,tower_http=debug
```

---

## 4. Struktur Database

### Tabel `flights`
Primary table untuk data penerbangan.

**Kolom:**
- `id`: SERIAL PRIMARY KEY
- `flight_number`: VARCHAR(10) NOT NULL
- `airline`: VARCHAR(100) NOT NULL
- `aircraft`: VARCHAR(50) NOT NULL
- `departure_time`: TIMESTAMPTZ NOT NULL
- `destination`: VARCHAR(10) NOT NULL
- `gate`: VARCHAR(10) NOT NULL
- `is_active`: BOOLEAN DEFAULT true
- `created_at`: TIMESTAMPTZ DEFAULT NOW()
- `updated_at`: TIMESTAMPTZ
- `device_id`: VARCHAR(50)

**Constraints:**
- Unique constraint: flight_number + departure_date
- Indexes: flight_number, departure_time, is_active

### Tabel `scan_data`
Menyimpan data hasil scan barcode.

**Kolom:**
- `id`: SERIAL PRIMARY KEY
- `barcode_value`: TEXT NOT NULL
- `barcode_format`: VARCHAR(50) NOT NULL
- `scan_time`: TIMESTAMPTZ DEFAULT NOW()
- `device_id`: VARCHAR(50) NOT NULL
- `flight_id`: INTEGER (FK to flights.id)
- `created_at`: TIMESTAMPTZ DEFAULT NOW()

**Foreign Keys:**
- `flight_id` → `flights.id` (ON DELETE SET NULL)

### Tabel `decode_barcode` (Fitur Tambahan)
Menyimpan hasil decode boarding pass IATA format.

**Kolom:**
- `id`: SERIAL PRIMARY KEY
- `barcode_value`: TEXT NOT NULL
- `passenger_name`: VARCHAR(100) NOT NULL
- `booking_code`: VARCHAR(10) NOT NULL (PNR)
- `origin`: VARCHAR(3) NOT NULL (Airport code)
- `destination`: VARCHAR(3) NOT NULL (Airport code)
- `airline_code`: VARCHAR(3) NOT NULL
- `flight_number`: VARCHAR(10) NOT NULL
- `flight_date_julian`: VARCHAR(3) NOT NULL
- `cabin_class`: VARCHAR(1) NOT NULL
- `seat_number`: VARCHAR(5) NOT NULL
- `sequence_number`: VARCHAR(4) NOT NULL
- `ticket_status`: VARCHAR(1) NOT NULL
- `scan_data_id`: INTEGER (FK to scan_data.id)
- `created_at`: TIMESTAMPTZ DEFAULT NOW()

---

## 5. API Endpoints

### Flight Management

#### GET /api/flights
Mendapatkan daftar semua penerbangan dengan optional filter tanggal.

**Query Parameters:**
- `date` (optional): Format YYYY-MM-DD

**Response:**
```json
{
  "status": "success",
  "data": [
    {
      "id": 1,
      "flightNumber": "GA123",
      "airline": "Garuda Indonesia",
      "aircraft": "Boeing 737-800",
      "departureTime": "2024-01-15T14:30:00Z",
      "destination": "CGK",
      "gate": "A5",
      "isActive": true,
      "createdAt": "2024-01-15T08:00:00Z",
      "updatedAt": null
    }
  ],
  "total": 1
}
```

#### POST /api/flights
Membuat penerbangan baru.

**Request Body:**
```json
{
  "flightNumber": "JT456",
  "airline": "Lion Air",
  "aircraft": "Airbus A320",
  "departureTime": "2024-01-15T16:00:00Z",
  "destination": "DPS",
  "gate": "B3",
  "deviceId": "device_123456"
}
```

**Validation Rules:**
- `flightNumber`: 3-10 karakter
- `airline`: 2-100 karakter
- `aircraft`: 2-50 karakter
- `destination`: Exactly 3 karakter (airport code)
- `gate`: Format A1-Z99
- `departureTime`: Tidak boleh di masa lalu

#### GET /api/flights/:id
Mendapatkan detail penerbangan berdasarkan ID.

#### PUT /api/flights/:id
Update data penerbangan. Semua field optional kecuali ID.

#### DELETE /api/flights/:id
Soft delete penerbangan (set is_active = false).

#### GET /api/flights/:id/statistics
Mendapatkan statistik scan untuk penerbangan tertentu.

**Response:**
```json
{
  "status": "success",
  "data": {
    "flightId": 1,
    "flightNumber": "GA123",
    "totalScans": 25,
    "uniqueScans": 23,
    "duplicateScans": 2,
    "scansByHour": [
      {"hour": "14:00", "count": 5}
    ],
    "topDevices": [
      {"deviceId": "device_123456", "scanCount": 15}
    ]
  }
}
```

#### GET /api/flights_decoder
Alias endpoint untuk GET /api/flights (sesuai requirement plan).

### Scan Data Management

#### POST /api/scan-data
Menyimpan data scan barcode baru.

**Request Body:**
```json
{
  "barcodeValue": "ABC123DEF456GHI789",
  "barcodeFormat": "PDF417",
  "deviceId": "device_123456",
  "flightId": 1
}
```

#### GET /api/scan-data
Mendapatkan data scan dengan filter.

**Query Parameters:**
- `flight_id` (optional): Filter by flight ID
- `date_range` (optional): Format "start,end" (ISO 8601)

### Synchronization

#### GET /api/sync/flights
Incremental sync berdasarkan timestamp.

**Query Parameters:**
- `last_sync` (optional): ISO 8601 timestamp

#### POST /api/sync/flights/bulk
Bulk insert/update flights.

**Request Body:** Array of flight objects

---

## 6. Fitur Barcode Decoder

### Konsep
Fitur ini mengubah barcode boarding pass format IATA BCBP menjadi data terstruktur yang mudah dibaca.

### Contoh Input/Output:

**Input Barcode:**
```
M1BAYU/MUHAMMAD MR ESMMTHQ DHXCGKID 6473 032Y007A0002 300
```

**Output Decoded:**
```json
{
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
```

### API Endpoints:

#### POST /api/decode-barcode
Decode barcode boarding pass.

**Request Body:**
```json
{
  "barcodeValue": "M1BAYU/MUHAMMAD MR ESMMTHQ...",
  "scanDataId": 1  // optional link to scan_data
}
```

#### GET /api/decoded-barcodes
Mendapatkan semua hasil decode yang tersimpan.

### Parsing Logic
Sistem menggunakan algoritma parsing berdasarkan standar IATA BCBP:

1. **Passenger Name** (posisi 2-20): Format "LAST/FIRST TITLE" → "FIRST LAST" (space separator)
2. **Booking Code** (posisi 21-26): PNR code
3. **Origin/Destination** (posisi 28-33): Airport codes
4. **Airline** (posisi 34-35): IATA airline code
5. **Flight Number** (posisi 37-40): Numeric flight number (integer)
6. **Julian Date** (posisi 42-44): Day of year (001-366)
7. **Cabin Class** (posisi 45): Y=Economy, C=Business, F=First
8. **Seat Number** (posisi 46-49): Seat assignment
9. **Sequence** (posisi 50-53): Check-in sequence
10. **Status** (posisi 54): E=Electronic ticket

---

## 7. Error Handling

### Standard Error Response Format:
```json
{
  "status": "error",
  "message": "Error description",
  "code": "ERROR_CODE",
  "details": {
    "field": "validation error details"
  }
}
```

### Error Codes:
- `FLIGHT_NOT_FOUND`: Flight dengan ID tertentu tidak ditemukan
- `DUPLICATE_FLIGHT`: Nomor penerbangan sudah ada untuk tanggal tersebut
- `INVALID_GATE_FORMAT`: Format gate harus A1-Z99
- `INVALID_DEPARTURE_TIME`: Waktu keberangkatan tidak boleh di masa lalu
- `INVALID_BARCODE_FORMAT`: Format barcode tidak valid untuk decoding
- `VALIDATION_ERROR`: Input validation gagal
- `INTERNAL_ERROR`: Database atau server error

### HTTP Status Codes:
- `200`: Success
- `201`: Created
- `204`: No Content (untuk DELETE)
- `400`: Bad Request (validation errors)
- `404`: Not Found
- `409`: Conflict (duplicate data)
- `500`: Internal Server Error

---

## 8. Testing

### Unit Testing
```bash
# Run all tests
cargo test

# Run specific test module
cargo test database::tests

# Run with output
cargo test -- --nocapture
```

### Integration Testing
Buat tests di `tests/` directory:

```bash
# API endpoint testing
cargo test --test api_tests

# Database operations testing
cargo test --test db_tests
```

### Load Testing
Gunakan tools seperti:
- **wrk**: `wrk -t12 -c400 -d30s http://127.0.0.1:3000/api/flights`
- **Apache Bench**: `ab -n 1000 -c 10 http://127.0.0.1:3000/api/flights`

### Test Cases yang Harus Dicakup:
1. **Flight CRUD Operations**
   - Create flight dengan data valid
   - Create flight dengan data invalid (validation)
   - Get flights dengan dan tanpa filter
   - Update flight partial/complete
   - Delete flight (soft delete)

2. **Scan Data Operations**
   - Create scan dengan flight_id valid
   - Create scan dengan flight_id invalid
   - Get scans dengan filter

3. **Barcode Decoder**
   - Decode valid IATA barcode
   - Decode invalid/malformed barcode
   - Edge cases (short barcode, special characters)

4. **Error Scenarios**
   - Network timeouts
   - Database connection failures
   - Invalid JSON payloads
   - Missing required fields

---

## 9. Deployment

### Production Setup

#### Docker Deployment
1. **Dockerfile**:
```dockerfile
FROM rust:1.70-slim as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y libpq5 ca-certificates
COPY --from=builder /app/target/release/airtally-restapi /usr/local/bin/
EXPOSE 3000
CMD ["airtally-restapi"]
```

2. **docker-compose.yml**:
```yaml
version: '3.8'
services:
  api:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgresql://user:pass@db:5432/airtally
    depends_on:
      - db

  db:
    image: postgres:13
    environment:
      POSTGRES_DB: airtally
      POSTGRES_USER: user
      POSTGRES_PASSWORD: pass
    volumes:
      - postgres_data:/var/lib/postgresql/data

volumes:
  postgres_data:
```

#### Environment Configuration
**Production .env:**
```
DATABASE_URL=postgresql://prod_user:secure_password@prod_host:5432/airtally_prod
RUST_LOG=airtally_api=info,tower_http=warn
PORT=3000
DATABASE_MAX_CONNECTIONS=20
```

#### Security Considerations:
1. **Database**: Gunakan user dengan limited privileges
2. **Networking**: Firewall rules untuk port 3000
3. **Secrets**: Gunakan secret management (AWS Secrets Manager, HashiCorp Vault)
4. **SSL/TLS**: Terminate SSL di load balancer
5. **Rate Limiting**: Implement di API Gateway atau reverse proxy

### CI/CD Pipeline
**GitHub Actions example:**
```yaml
name: Deploy
on:
  push:
    branches: [main]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
      - run: cargo test

  deploy:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - name: Deploy to production
        run: |
          # Your deployment script
```

---

## 10. Maintenance

### Monitoring
Implementasikan monitoring untuk:

1. **Application Metrics**:
   - Request/response times
   - Error rates
   - Throughput (requests/second)
   - Active connections

2. **Database Metrics**:
   - Query performance
   - Connection pool utilization
   - Slow query log
   - Database size growth

3. **System Metrics**:
   - CPU/Memory usage
   - Disk I/O
   - Network bandwidth

### Backup Strategy
1. **Database Backup**:
   ```bash
   # Daily backup
   pg_dump airtally_prod > backup_$(date +%Y%m%d).sql

   # Point-in-time recovery setup
   # Enable WAL archiving di postgresql.conf
   ```

2. **Application Backup**:
   - Source code: Git repository
   - Configuration: Environment files
   - Logs: Centralized logging system

### Performance Optimization
1. **Database Optimization**:
   - Analyze query plans: `EXPLAIN ANALYZE`
   - Add indexes untuk frequently queried columns
   - Vacuum dan analyze tables regularly
   - Connection pooling tuning

2. **Application Optimization**:
   - Profile dengan tools seperti `perf` atau `flamegraph`
   - Optimize hot paths
   - Memory usage analysis
   - Async/await optimization

3. **Infrastructure Optimization**:
   - Load balancing
   - Caching layer (Redis)
   - CDN untuk static assets
   - Database read replicas

### Troubleshooting Common Issues

#### Database Connection Issues
```bash
# Check connection limit
SELECT count(*) FROM pg_stat_activity;

# Check long running queries
SELECT pid, now() - pg_stat_activity.query_start AS duration, query
FROM pg_stat_activity
WHERE (now() - pg_stat_activity.query_start) > interval '5 minutes';
```

#### Memory Issues
```bash
# Check memory usage
free -h
ps aux | sort -k 4 -nr | head

# Check for memory leaks
valgrind --tool=memcheck ./target/release/airtally-restapi
```

#### Performance Issues
```bash
# Check CPU usage
top -p $(pgrep airtally-restapi)

# Profile application
perf record -g ./target/release/airtally-restapi
perf report
```

### Upgrade Procedures
1. **Database Migrations**:
   - Test migrations di staging environment
   - Backup database sebelum migration
   - Run migrations dengan rollback plan

2. **Application Updates**:
   - Blue-green deployment strategy
   - Health check endpoints
   - Gradual rollout dengan monitoring

3. **Dependency Updates**:
   ```bash
   # Check for updates
   cargo outdated

   # Update dependencies
   cargo update

   # Security audit
   cargo audit
   ```

### Logs dan Debugging
1. **Log Levels**:
   - `ERROR`: Critical errors yang perlu immediate attention
   - `WARN`: Issues yang perlu monitoring
   - `INFO`: Normal operation info
   - `DEBUG`: Detailed debugging information

2. **Log Analysis**:
   ```bash
   # Filter error logs
   grep "ERROR" /var/log/airtally.log

   # Monitor real-time logs
   tail -f /var/log/airtally.log | grep "ERROR\|WARN"
   ```

3. **Debugging Techniques**:
   - Enable debug logging untuk specific modules
   - Use database query logging
   - HTTP request/response logging
   - Application metrics dashboard

---

## Kesimpulan

Dokumentasi ini memberikan panduan lengkap untuk development, deployment, dan maintenance AirTally REST API. Sistem ini dibangun dengan fokus pada:

- **Reliability**: Error handling yang robust dan database consistency
- **Performance**: Optimized queries dan efficient data structures
- **Scalability**: Modular architecture dan stateless design
- **Maintainability**: Clear code structure dan comprehensive logging
- **Security**: Input validation dan secure database operations

Untuk pertanyaan atau issues, silakan buat ticket di repository atau hubungi tim development.

**Last Updated**: 29 September 2025
**Version**: 1.0
**Authors**: Development Team