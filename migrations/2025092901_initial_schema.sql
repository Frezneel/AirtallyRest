-- Inisialisasi skema database untuk AirTally

-- Membuat tabel flights
-- Menggunakan SERIAL PRIMARY KEY untuk auto-increment yang aman di Postgres
-- Menggunakan TIMESTAMPTZ untuk menyimpan waktu dengan informasi zona waktu
CREATE TABLE flights (
                         id SERIAL PRIMARY KEY,
                         flight_number VARCHAR(10) NOT NULL,
                         airline VARCHAR(100) NOT NULL,
                         aircraft VARCHAR(50) NOT NULL,
                         departure_time TIMESTAMPTZ NOT NULL,
                         destination VARCHAR(10) NOT NULL,
                         gate VARCHAR(10) NOT NULL,
                         is_active BOOLEAN DEFAULT true,
                         created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                         updated_at TIMESTAMPTZ,
                         device_id VARCHAR(50)
);

-- Membuat tabel scan_data
CREATE TABLE scan_data (
                           id SERIAL PRIMARY KEY,
                           barcode_value TEXT NOT NULL,
                           barcode_format VARCHAR(50) NOT NULL,
                           scan_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                           device_id VARCHAR(50) NOT NULL,
                           flight_id INTEGER,
                           created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Menambahkan Foreign Key constraint ke scan_data
ALTER TABLE scan_data
    ADD CONSTRAINT fk_flight
        FOREIGN KEY (flight_id)
            REFERENCES flights(id)
            ON DELETE SET NULL; -- Atau ON DELETE CASCADE jika data scan harus dihapus saat flight dihapus

-- Membuat index untuk mempercepat query
CREATE INDEX idx_flight_number ON flights(flight_number);
CREATE INDEX idx_departure_time ON flights(departure_time);
CREATE INDEX idx_is_active ON flights(is_active);
CREATE INDEX idx_scan_data_flight_id ON scan_data(flight_id);
CREATE INDEX idx_scan_data_scan_time ON scan_data(scan_time);

-- Menambahkan constraint unik untuk nomor penerbangan per hari (TELAH DIPERBAIKI)
-- Menggunakan (departure_time AT TIME ZONE 'utc')::date untuk memastikan fungsi IMMUTABLE
CREATE UNIQUE INDEX idx_unique_flight_per_day
    ON flights (flight_number, ((departure_time AT TIME ZONE 'utc')::date));

-- Komentar: Tabel `flights` kini siap digunakan dengan relasi ke `scan_data`.
-- Timestamp disimpan dalam UTC (TIMESTAMPTZ) untuk konsistensi.

