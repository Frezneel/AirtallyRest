-- Migration: Drop passenger_status_codes table
-- This table is no longer needed as we use infant_status boolean instead

DROP TABLE IF EXISTS passenger_status_codes CASCADE;
