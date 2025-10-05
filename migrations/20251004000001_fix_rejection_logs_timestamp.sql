-- Fix rejection_logs.rejected_at to use TIMESTAMPTZ instead of TIMESTAMP
ALTER TABLE rejection_logs ALTER COLUMN rejected_at TYPE TIMESTAMPTZ;
