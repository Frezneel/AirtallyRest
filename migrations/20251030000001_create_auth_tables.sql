-- Migration: Create authentication tables
-- Description: Tables for user authentication and role-based access control

-- Create roles table
CREATE TABLE IF NOT EXISTS roles (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    description TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ
);

-- Create permissions table
CREATE TABLE IF NOT EXISTS permissions (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL,
    description TEXT,
    resource VARCHAR(50) NOT NULL,  -- e.g., 'flights', 'scans', 'users'
    action VARCHAR(50) NOT NULL,    -- e.g., 'create', 'read', 'update', 'delete'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Create users table
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    full_name VARCHAR(255) NOT NULL,
    role_id INTEGER NOT NULL REFERENCES roles(id) ON DELETE RESTRICT,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_login_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    created_by INTEGER REFERENCES users(id) ON DELETE SET NULL
);

-- Create role_permissions junction table
CREATE TABLE IF NOT EXISTS role_permissions (
    role_id INTEGER NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission_id INTEGER NOT NULL REFERENCES permissions(id) ON DELETE CASCADE,
    PRIMARY KEY (role_id, permission_id)
);

-- Create user_sessions table for JWT token management
CREATE TABLE IF NOT EXISTS user_sessions (
    id SERIAL PRIMARY KEY,
    user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    device_info TEXT,
    ip_address VARCHAR(45),
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    revoked_at TIMESTAMPTZ
);

-- Create indexes for performance
CREATE INDEX idx_users_username ON users(username);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_role_id ON users(role_id);
CREATE INDEX idx_users_is_active ON users(is_active);
CREATE INDEX idx_user_sessions_user_id ON user_sessions(user_id);
CREATE INDEX idx_user_sessions_token_hash ON user_sessions(token_hash);
CREATE INDEX idx_user_sessions_expires_at ON user_sessions(expires_at);
CREATE INDEX idx_role_permissions_role_id ON role_permissions(role_id);
CREATE INDEX idx_role_permissions_permission_id ON role_permissions(permission_id);

-- Insert default roles
INSERT INTO roles (name, description) VALUES
    ('superuser', 'Full system access with all permissions'),
    ('admin', 'Administrative access with user management'),
    ('operator', 'Flight and scan operations'),
    ('viewer', 'Read-only access to data')
ON CONFLICT (name) DO NOTHING;

-- Insert default permissions
INSERT INTO permissions (name, description, resource, action) VALUES
    -- Flight permissions
    ('flights.create', 'Create new flights', 'flights', 'create'),
    ('flights.read', 'View flight data', 'flights', 'read'),
    ('flights.update', 'Update flight information', 'flights', 'update'),
    ('flights.delete', 'Delete flights', 'flights', 'delete'),

    -- Scan permissions
    ('scans.create', 'Create scan data', 'scans', 'create'),
    ('scans.read', 'View scan data', 'scans', 'read'),
    ('scans.delete', 'Delete scan data', 'scans', 'delete'),

    -- Decoded barcode permissions
    ('decoded.read', 'View decoded barcodes', 'decoded', 'read'),
    ('decoded.delete', 'Delete decoded barcodes', 'decoded', 'delete'),

    -- User management permissions
    ('users.create', 'Create new users', 'users', 'create'),
    ('users.read', 'View user data', 'users', 'read'),
    ('users.update', 'Update user information', 'users', 'update'),
    ('users.delete', 'Delete users', 'users', 'delete'),

    -- Role management permissions
    ('roles.create', 'Create new roles', 'roles', 'create'),
    ('roles.read', 'View roles', 'roles', 'read'),
    ('roles.update', 'Update roles', 'roles', 'update'),
    ('roles.delete', 'Delete roles', 'roles', 'delete'),

    -- System permissions
    ('system.logs', 'View system logs', 'system', 'read'),
    ('system.settings', 'Manage system settings', 'system', 'update')
ON CONFLICT (name) DO NOTHING;

-- Assign all permissions to superuser role
INSERT INTO role_permissions (role_id, permission_id)
SELECT
    (SELECT id FROM roles WHERE name = 'superuser'),
    id
FROM permissions
ON CONFLICT DO NOTHING;

-- Assign permissions to admin role (all except system settings)
INSERT INTO role_permissions (role_id, permission_id)
SELECT
    (SELECT id FROM roles WHERE name = 'admin'),
    id
FROM permissions
WHERE resource IN ('flights', 'scans', 'decoded', 'users', 'roles')
ON CONFLICT DO NOTHING;

-- Assign permissions to operator role (flight and scan operations)
INSERT INTO role_permissions (role_id, permission_id)
SELECT
    (SELECT id FROM roles WHERE name = 'operator'),
    id
FROM permissions
WHERE resource IN ('flights', 'scans', 'decoded') AND action IN ('create', 'read', 'update')
ON CONFLICT DO NOTHING;

-- Assign permissions to viewer role (read-only)
INSERT INTO role_permissions (role_id, permission_id)
SELECT
    (SELECT id FROM roles WHERE name = 'viewer'),
    id
FROM permissions
WHERE action = 'read'
ON CONFLICT DO NOTHING;

-- Create default superuser account
-- Username: superuser
-- Password: AirTally2025! (hashed with bcrypt cost 12)
-- Note: Change this password immediately after first login
INSERT INTO users (username, email, password_hash, full_name, role_id, is_active)
VALUES (
    'superuser',
    'superuser@airtally.local',
    '$2b$12$SvU6dXk58vp7Y0XC8q3cNOh1gFDjPOa7Bq7jx8M2PIR2AQyeEkqB2',  -- AirTally2025!
    'Super User',
    (SELECT id FROM roles WHERE name = 'superuser'),
    TRUE
)
ON CONFLICT (username) DO UPDATE SET password_hash = EXCLUDED.password_hash;

-- Add comment to remind password change
COMMENT ON TABLE users IS 'User accounts. Default superuser password must be changed after first login.';
