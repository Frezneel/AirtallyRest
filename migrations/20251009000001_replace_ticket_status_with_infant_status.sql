-- Migration: Replace ticket_status with infant_status
-- Changes ticket_status VARCHAR(1) to infant_status BOOLEAN

-- Drop the old ticket_status column
ALTER TABLE decode_barcode
DROP COLUMN IF EXISTS ticket_status;

-- Add new infant_status column
ALTER TABLE decode_barcode
ADD COLUMN infant_status BOOLEAN NOT NULL DEFAULT FALSE;

-- Add index for filtering infant tickets
CREATE INDEX idx_decode_infant_status ON decode_barcode(infant_status);

COMMENT ON COLUMN decode_barcode.infant_status IS 'TRUE if passenger is an infant (lap infant, no seat assigned), FALSE otherwise';
