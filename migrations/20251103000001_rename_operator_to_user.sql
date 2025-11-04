-- Migration: Rename operator role to user
-- Description: Rename 'operator' role to 'user'
-- Created: 2025-11-03

-- Update operator role to user
UPDATE roles
SET name = 'user',
    description = 'Standard user with basic operations'
WHERE name = 'operator';
