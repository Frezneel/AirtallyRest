-- Migration: Add rejection_logs table for server-side rejection tracking
-- This allows developers to monitor rejected barcodes across all devices

CREATE TABLE IF NOT EXISTS rejection_logs (
    id SERIAL PRIMARY KEY,
    barcode_value TEXT NOT NULL,
    barcode_format VARCHAR(50) NOT NULL,
    reason TEXT NOT NULL,
    expected_date VARCHAR(20),
    actual_date VARCHAR(20),
    flight_number VARCHAR(20),
    airline VARCHAR(10),
    device_id VARCHAR(100),
    rejected_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Indexes for efficient querying
CREATE INDEX idx_rejection_logs_rejected_at ON rejection_logs(rejected_at);
CREATE INDEX idx_rejection_logs_airline ON rejection_logs(airline);
CREATE INDEX idx_rejection_logs_reason ON rejection_logs(reason);
CREATE INDEX idx_rejection_logs_device_id ON rejection_logs(device_id);

-- Comment
COMMENT ON TABLE rejection_logs IS 'Server-side log of rejected barcodes for developer monitoring';
COMMENT ON COLUMN rejection_logs.barcode_value IS 'Full barcode string that was rejected';
COMMENT ON COLUMN rejection_logs.reason IS 'Rejection reason (date_mismatch, invalid_format, error)';
COMMENT ON COLUMN rejection_logs.expected_date IS 'Expected date (usually today)';
COMMENT ON COLUMN rejection_logs.actual_date IS 'Actual date from barcode';
COMMENT ON COLUMN rejection_logs.device_id IS 'Device fingerprint for tracking';
