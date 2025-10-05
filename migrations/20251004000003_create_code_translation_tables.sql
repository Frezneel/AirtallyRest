-- Migration: Create code translation tables for starter data
-- This allows admin to update translations without app deployment

-- Airport Codes Table
CREATE TABLE IF NOT EXISTS airport_codes (
    id SERIAL PRIMARY KEY,
    code VARCHAR(3) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    city VARCHAR(100) NOT NULL,
    country VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Airline Codes Table
CREATE TABLE IF NOT EXISTS airline_codes (
    id SERIAL PRIMARY KEY,
    code VARCHAR(3) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    country VARCHAR(100) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Cabin Class Codes Table
CREATE TABLE IF NOT EXISTS cabin_class_codes (
    id SERIAL PRIMARY KEY,
    code VARCHAR(1) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    description VARCHAR(255),
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Passenger Status Codes Table
CREATE TABLE IF NOT EXISTS passenger_status_codes (
    id SERIAL PRIMARY KEY,
    code VARCHAR(1) NOT NULL UNIQUE,
    description VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Starter Data Version Table (for incremental sync)
CREATE TABLE IF NOT EXISTS starter_data_version (
    id SERIAL PRIMARY KEY,
    version INTEGER NOT NULL DEFAULT 1,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP
);

-- Insert initial version
INSERT INTO starter_data_version (version) VALUES (1);

-- Indexes
CREATE INDEX idx_airport_codes_code ON airport_codes(code);
CREATE INDEX idx_airline_codes_code ON airline_codes(code);
CREATE INDEX idx_cabin_class_codes_code ON cabin_class_codes(code);
CREATE INDEX idx_passenger_status_codes_code ON passenger_status_codes(code);

-- Comments
COMMENT ON TABLE airport_codes IS 'Airport code translations (IATA codes)';
COMMENT ON TABLE airline_codes IS 'Airline code translations (IATA codes)';
COMMENT ON TABLE cabin_class_codes IS 'Cabin class code translations';
COMMENT ON TABLE passenger_status_codes IS 'Passenger status code translations';
COMMENT ON TABLE starter_data_version IS 'Version tracking for starter data sync';

-- Insert sample data for Indonesian airports
INSERT INTO airport_codes (code, name, city, country) VALUES
('CGK', 'Soekarno-Hatta International Airport', 'Jakarta', 'Indonesia'),
('DPS', 'Ngurah Rai International Airport', 'Denpasar', 'Indonesia'),
('SUB', 'Juanda International Airport', 'Surabaya', 'Indonesia'),
('KNO', 'Kualanamu International Airport', 'Medan', 'Indonesia'),
('BDO', 'Husein Sastranegara International Airport', 'Bandung', 'Indonesia'),
('JOG', 'Adisucipto International Airport', 'Yogyakarta', 'Indonesia'),
('SRG', 'Ahmad Yani International Airport', 'Semarang', 'Indonesia'),
('SOC', 'Adi Sumarmo International Airport', 'Solo', 'Indonesia'),
('UPG', 'Sultan Hasanuddin International Airport', 'Makassar', 'Indonesia'),
('BPN', 'Sultan Aji Muhammad Sulaiman Airport', 'Balikpapan', 'Indonesia');

-- Insert sample data for Indonesian airlines
INSERT INTO airline_codes (code, name, country) VALUES
('GA', 'Garuda Indonesia', 'Indonesia'),
('JT', 'Lion Air', 'Indonesia'),
('QG', 'Citilink', 'Indonesia'),
('ID', 'Batik Air', 'Indonesia'),
('QZ', 'AirAsia Indonesia', 'Indonesia'),
('IW', 'Wings Air', 'Indonesia'),
('IN', 'NAM Air', 'Indonesia'),
('SJ', 'Sriwijaya Air', 'Indonesia'),
('QD', 'JC International Airlines', 'Indonesia');

-- Insert cabin class codes
INSERT INTO cabin_class_codes (code, name, description) VALUES
('F', 'First Class', 'Premium first class seating'),
('C', 'Business Class', 'Business class seating'),
('W', 'Premium Economy', 'Premium economy seating'),
('Y', 'Economy Class', 'Standard economy seating'),
('J', 'Business (alternate)', 'Business class (alternate code)');

-- Insert passenger status codes
INSERT INTO passenger_status_codes (code, description) VALUES
('0', 'Ticket issuance/passenger checked in'),
('1', 'Ticket issuance without passenger record'),
('2', 'Baggage checked'),
('3', 'Passenger boarded'),
('4', 'Passenger not boarded'),
('5', 'Checked baggage for standby passenger');
