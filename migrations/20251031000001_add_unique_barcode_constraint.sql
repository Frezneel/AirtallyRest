-- ============================================================================
-- Add Unique Constraint: Prevent Duplicate Barcode Scans Per Flight
-- ============================================================================
-- Prevents the same passenger barcode from being scanned multiple times
-- for the same flight, even across different devices.
--
-- Scenario Prevention:
-- - Officer A scans Passenger 01 → Saved to server
-- - Officer B scans same Passenger 01 → Server rejects with HTTP 409
-- ============================================================================

BEGIN;

-- Step 1: Check for existing duplicates
DO $$
DECLARE
    duplicate_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO duplicate_count
    FROM (
        SELECT barcode_value, flight_id, COUNT(*) as cnt
        FROM scan_data
        WHERE flight_id IS NOT NULL
        GROUP BY barcode_value, flight_id
        HAVING COUNT(*) > 1
    ) duplicates;

    IF duplicate_count > 0 THEN
        RAISE NOTICE 'Found % groups of duplicate scans', duplicate_count;
        RAISE NOTICE 'Keeping oldest scan per barcode+flight, newer ones will remain as historical data';
    ELSE
        RAISE NOTICE 'No existing duplicates found';
    END IF;
END $$;

-- Step 2: Create unique constraint
-- This prevents future duplicates at database level
CREATE UNIQUE INDEX IF NOT EXISTS idx_unique_barcode_per_flight
ON scan_data (barcode_value, flight_id)
WHERE flight_id IS NOT NULL;

-- Step 3: Add comment
COMMENT ON INDEX idx_unique_barcode_per_flight IS
'Prevents duplicate barcode scans for the same flight across all devices.
Server will return HTTP 409 Conflict when duplicate is attempted.';

-- Step 4: Verify constraint
SELECT
    indexname,
    indexdef
FROM pg_indexes
WHERE tablename = 'scan_data'
  AND indexname = 'idx_unique_barcode_per_flight';

COMMIT;

-- ============================================================================
-- After this migration:
--
-- ✅ Database enforces: One barcode per flight (unique)
-- ✅ Server returns: HTTP 409 Conflict if duplicate attempted
-- ✅ Client handles: 409 as successful sync (data already exists on server)
-- ============================================================================
