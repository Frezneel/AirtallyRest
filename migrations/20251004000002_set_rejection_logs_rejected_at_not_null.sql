-- Set rejection_logs.rejected_at to NOT NULL since it has a DEFAULT value
ALTER TABLE rejection_logs ALTER COLUMN rejected_at SET NOT NULL;
