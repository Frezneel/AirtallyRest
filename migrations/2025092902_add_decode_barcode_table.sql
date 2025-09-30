-- Migration untuk menambah tabel decode_barcode
-- Sesuai dengan fitur tambahan di plan_restapi.txt

CREATE TABLE decode_barcode (
    id SERIAL PRIMARY KEY,
    barcode_value TEXT NOT NULL,
    passenger_name VARCHAR(100) NOT NULL,
    booking_code VARCHAR(10) NOT NULL,
    origin VARCHAR(3) NOT NULL,
    destination VARCHAR(3) NOT NULL,
    airline_code VARCHAR(3) NOT NULL,
    flight_number INTEGER NOT NULL,
    flight_date_julian VARCHAR(3) NOT NULL,
    cabin_class VARCHAR(1) NOT NULL,
    seat_number VARCHAR(5) NOT NULL,
    sequence_number VARCHAR(4) NOT NULL,
    ticket_status VARCHAR(1) NOT NULL,
    scan_data_id INTEGER,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Menambahkan foreign key ke scan_data (optional)
ALTER TABLE decode_barcode
    ADD CONSTRAINT fk_scan_data
        FOREIGN KEY (scan_data_id)
            REFERENCES scan_data(id)
            ON DELETE SET NULL;

-- Index untuk performa
CREATE INDEX idx_decode_barcode_value ON decode_barcode(barcode_value);
CREATE INDEX idx_decode_passenger_name ON decode_barcode(passenger_name);
CREATE INDEX idx_decode_booking_code ON decode_barcode(booking_code);
CREATE INDEX idx_decode_created_at ON decode_barcode(created_at);