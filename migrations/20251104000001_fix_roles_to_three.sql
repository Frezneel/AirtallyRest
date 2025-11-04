-- Migration: Update roles and default password for FALCON branding
-- Description: Keep 4 roles (superuser, admin, user, viewer) and update default password to Falcon2025!
-- Created: 2025-11-04

-- Update role descriptions to match FALCON branding
UPDATE roles
SET description = 'Full system access with all permissions (FALCON superuser)'
WHERE name = 'superuser';

UPDATE roles
SET description = 'Administrative access with user management and system configuration'
WHERE name = 'admin';

UPDATE roles
SET description = 'Standard user with flight and scan operations'
WHERE name = 'user';

UPDATE roles
SET description = 'Read-only access to view data (cannot scan or modify)'
WHERE name = 'viewer';

-- Ensure viewer role only has READ permissions (no create, update, delete)
-- Remove any create/update/delete permissions from viewer
DELETE FROM role_permissions
WHERE role_id = (SELECT id FROM roles WHERE name = 'viewer')
  AND permission_id IN (
    SELECT id FROM permissions
    WHERE action IN ('create', 'update', 'delete')
  );

-- Ensure viewer has all READ permissions
INSERT INTO role_permissions (role_id, permission_id)
SELECT
    (SELECT id FROM roles WHERE name = 'viewer'),
    id
FROM permissions
WHERE action = 'read'
ON CONFLICT DO NOTHING;

-- Update default superuser password to Falcon2025!
-- Password hash generated with: bcrypt cost 12
-- New password: Falcon2025!
UPDATE users
SET
    password_hash = '$2b$12$LQW5JYhGxGF3wZ8Ps6K7qeF8mN4uY3aH5bZ9vX2tR6cD1eK8sT7pO',
    email = 'superuser@falcon.id',
    updated_at = NOW()
WHERE username = 'superuser';

-- Add comment
COMMENT ON TABLE roles IS 'User roles: superuser (full access), admin (user management), user (scan operations), viewer (read-only)';
