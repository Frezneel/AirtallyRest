use crate::{
    errors::AppError,
    models::{
        User, UserWithRole, Role, Permission, RoleWithPermissions,
        LoginResponse, CreateUserRequest, UpdateUserRequest, ListUsersQuery,
    },
};
use sqlx::PgPool;
use chrono::{Utc, Duration};
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{encode, decode, Header, Validation, EncodingKey, DecodingKey};
use sha2::{Sha256, Digest};

/// JWT secret key (should be loaded from environment variable)
fn get_jwt_secret() -> String {
    std::env::var("JWT_SECRET").unwrap_or_else(|_| {
        tracing::warn!("JWT_SECRET not set, using default (NOT SECURE FOR PRODUCTION)");
        "airtally_jwt_secret_change_in_production_2025".to_string()
    })
}

/// Token expiration duration (7 days)
const TOKEN_EXPIRATION_DAYS: i64 = 7;

// ==================== AUTHENTICATION FUNCTIONS ====================

/// Authenticate user with username and password
pub async fn authenticate_user(
    pool: &PgPool,
    username: &str,
    password: &str,
    device_info: Option<String>,
    ip_address: Option<String>,
) -> Result<LoginResponse, AppError> {
    // Get user from database
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, email, password_hash, full_name, role_id, is_active,
               last_login_at, created_at, updated_at, created_by
        FROM users
        WHERE username = $1
        "#,
    )
    .bind(username)
    .fetch_optional(pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid username or password".to_string()))?;

    // Check if user is active
    if !user.is_active {
        return Err(AppError::Unauthorized("User account is disabled".to_string()));
    }

    // Verify password
    let password_valid = verify(password, &user.password_hash)
        .map_err(|e| AppError::InternalError(format!("Password verification failed: {}", e)))?;

    if !password_valid {
        return Err(AppError::Unauthorized("Invalid username or password".to_string()));
    }

    // Get role
    let role = sqlx::query_as::<_, Role>(
        r#"
        SELECT id, name, description, created_at, updated_at
        FROM roles
        WHERE id = $1
        "#,
    )
    .bind(user.role_id)
    .fetch_one(pool)
    .await?;

    // Get permissions
    let permissions = sqlx::query_as::<_, Permission>(
        r#"
        SELECT p.id, p.name, p.description, p.resource, p.action, p.created_at
        FROM permissions p
        JOIN role_permissions rp ON p.id = rp.permission_id
        WHERE rp.role_id = $1
        ORDER BY p.name
        "#,
    )
    .bind(user.role_id)
    .fetch_all(pool)
    .await?;

    let permission_names: Vec<String> = permissions.iter().map(|p| p.name.clone()).collect();

    // Create JWT token
    let now = Utc::now();
    let expires_at = now + Duration::days(TOKEN_EXPIRATION_DAYS);

    let claims = crate::models::Claims {
        sub: user.id,
        username: user.username.clone(),
        role: role.name.clone(),
        permissions: permission_names.clone(),
        exp: expires_at.timestamp(),
        iat: now.timestamp(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(get_jwt_secret().as_bytes()),
    )
    .map_err(|e| AppError::InternalError(format!("Failed to generate token: {}", e)))?;

    // Hash token for storage
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    // Save session to database
    sqlx::query(
        r#"
        INSERT INTO user_sessions (user_id, token_hash, device_info, ip_address, expires_at)
        VALUES ($1, $2, $3, $4, $5)
        "#,
    )
    .bind(user.id)
    .bind(&token_hash)
    .bind(&device_info)
    .bind(&ip_address)
    .bind(expires_at)
    .execute(pool)
    .await?;

    // Update last login timestamp
    sqlx::query(
        r#"
        UPDATE users
        SET last_login_at = $1
        WHERE id = $2
        "#,
    )
    .bind(now)
    .bind(user.id)
    .execute(pool)
    .await?;

    // Build response
    let user_with_role = UserWithRole {
        id: user.id,
        username: user.username,
        email: user.email,
        full_name: user.full_name,
        role,
        is_active: user.is_active,
        last_login_at: Some(now),
        created_at: user.created_at,
        updated_at: user.updated_at,
    };

    Ok(LoginResponse {
        token,
        user: user_with_role,
        permissions: permission_names,
        expires_at,
    })
}

/// Verify JWT token and return user_id
pub async fn verify_token(pool: &PgPool, token: &str) -> Result<i32, AppError> {
    // Decode JWT
    let token_data = decode::<crate::models::Claims>(
        token,
        &DecodingKey::from_secret(get_jwt_secret().as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AppError::Unauthorized(format!("Invalid token: {}", e)))?;

    let user_id = token_data.claims.sub;

    // Hash token for lookup
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    // Check if session exists and is not revoked
    let session = sqlx::query(
        r#"
        SELECT id FROM user_sessions
        WHERE token_hash = $1
        AND user_id = $2
        AND expires_at > NOW()
        AND revoked_at IS NULL
        "#,
    )
    .bind(&token_hash)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if session.is_none() {
        return Err(AppError::Unauthorized("Session expired or revoked".to_string()));
    }

    Ok(user_id)
}

/// Revoke session (logout)
pub async fn revoke_session(pool: &PgPool, token: &str) -> Result<(), AppError> {
    // Hash token for lookup
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let token_hash = format!("{:x}", hasher.finalize());

    sqlx::query(
        r#"
        UPDATE user_sessions
        SET revoked_at = NOW()
        WHERE token_hash = $1
        "#,
    )
    .bind(&token_hash)
    .execute(pool)
    .await?;

    Ok(())
}

/// Change user password
pub async fn change_password(
    pool: &PgPool,
    user_id: i32,
    old_password: &str,
    new_password: &str,
) -> Result<(), AppError> {
    // Get current password hash
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, email, password_hash, full_name, role_id, is_active,
               last_login_at, created_at, updated_at, created_by
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    // Verify old password
    let password_valid = verify(old_password, &user.password_hash)
        .map_err(|e| AppError::InternalError(format!("Password verification failed: {}", e)))?;

    if !password_valid {
        return Err(AppError::Unauthorized("Invalid old password".to_string()));
    }

    // Hash new password
    let new_hash = hash(new_password, DEFAULT_COST)
        .map_err(|e| AppError::InternalError(format!("Password hashing failed: {}", e)))?;

    // Update password
    sqlx::query(
        r#"
        UPDATE users
        SET password_hash = $1, updated_at = NOW()
        WHERE id = $2
        "#,
    )
    .bind(&new_hash)
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

// ==================== USER MANAGEMENT FUNCTIONS ====================

/// Create new user
pub async fn create_user(
    pool: &PgPool,
    data: CreateUserRequest,
    creator_id: i32,
) -> Result<UserWithRole, AppError> {
    // Check if username already exists
    let existing = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE username = $1"
    )
    .bind(&data.username)
    .fetch_one(pool)
    .await?;

    if existing > 0 {
        return Err(AppError::ValidationError(
            validator::ValidationErrors::new()
        ));
    }

    // Check if email already exists
    let existing_email = sqlx::query_scalar::<_, i64>(
        "SELECT COUNT(*) FROM users WHERE email = $1"
    )
    .bind(&data.email)
    .fetch_one(pool)
    .await?;

    if existing_email > 0 {
        return Err(AppError::ValidationError(
            validator::ValidationErrors::new()
        ));
    }

    // Hash password
    let password_hash = hash(&data.password, DEFAULT_COST)
        .map_err(|e| AppError::InternalError(format!("Password hashing failed: {}", e)))?;

    // Insert user
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (username, email, password_hash, full_name, role_id, created_by)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, username, email, password_hash, full_name, role_id, is_active,
                  last_login_at, created_at, updated_at, created_by
        "#,
    )
    .bind(&data.username)
    .bind(&data.email)
    .bind(&password_hash)
    .bind(&data.full_name)
    .bind(data.role_id)
    .bind(creator_id)
    .fetch_one(pool)
    .await?;

    get_user_with_role(pool, user.id).await
}

/// Get user with role information
pub async fn get_user_with_role(pool: &PgPool, user_id: i32) -> Result<UserWithRole, AppError> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT id, username, email, password_hash, full_name, role_id, is_active,
               last_login_at, created_at, updated_at, created_by
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound(format!("User with id {} not found", user_id)))?;

    let role = sqlx::query_as::<_, Role>(
        r#"
        SELECT id, name, description, created_at, updated_at
        FROM roles
        WHERE id = $1
        "#,
    )
    .bind(user.role_id)
    .fetch_one(pool)
    .await?;

    Ok(UserWithRole {
        id: user.id,
        username: user.username,
        email: user.email,
        full_name: user.full_name,
        role,
        is_active: user.is_active,
        last_login_at: user.last_login_at,
        created_at: user.created_at,
        updated_at: user.updated_at,
    })
}

/// List users with filters
pub async fn list_users(
    pool: &PgPool,
    query: ListUsersQuery,
) -> Result<(Vec<UserWithRole>, i64), AppError> {
    let limit = query.limit.unwrap_or(100).min(500);
    let offset = query.offset.unwrap_or(0);

    let mut conditions = Vec::new();
    let mut count_conditions = Vec::new();

    if let Some(role_id) = query.role_id {
        conditions.push(format!("u.role_id = {}", role_id));
        count_conditions.push(format!("role_id = {}", role_id));
    }

    if let Some(is_active) = query.is_active {
        conditions.push(format!("u.is_active = {}", is_active));
        count_conditions.push(format!("is_active = {}", is_active));
    }

    let where_clause = if conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", conditions.join(" AND "))
    };

    let count_where_clause = if count_conditions.is_empty() {
        String::new()
    } else {
        format!("WHERE {}", count_conditions.join(" AND "))
    };

    // Get total count
    let total: i64 = sqlx::query_scalar(&format!(
        "SELECT COUNT(*) FROM users {}",
        count_where_clause
    ))
    .fetch_one(pool)
    .await?;

    // Get users with roles
    let query_str = format!(
        r#"
        SELECT u.id, u.username, u.email, u.full_name, u.is_active,
               u.last_login_at, u.created_at, u.updated_at,
               r.id as role_id, r.name as role_name, r.description as role_description,
               r.created_at as role_created_at, r.updated_at as role_updated_at
        FROM users u
        JOIN roles r ON u.role_id = r.id
        {}
        ORDER BY u.created_at DESC
        LIMIT {} OFFSET {}
        "#,
        where_clause, limit, offset
    );

    let rows = sqlx::query(&query_str).fetch_all(pool).await?;

    let users: Vec<UserWithRole> = rows
        .iter()
        .map(|row| {
            use sqlx::Row;
            UserWithRole {
                id: row.get("id"),
                username: row.get("username"),
                email: row.get("email"),
                full_name: row.get("full_name"),
                role: Role {
                    id: row.get("role_id"),
                    name: row.get("role_name"),
                    description: row.get("role_description"),
                    created_at: row.get("role_created_at"),
                    updated_at: row.get("role_updated_at"),
                },
                is_active: row.get("is_active"),
                last_login_at: row.get("last_login_at"),
                created_at: row.get("created_at"),
                updated_at: row.get("updated_at"),
            }
        })
        .collect();

    Ok((users, total))
}

/// Update user
pub async fn update_user(
    pool: &PgPool,
    user_id: i32,
    data: UpdateUserRequest,
) -> Result<UserWithRole, AppError> {
    let mut updates = Vec::new();

    if let Some(email) = &data.email {
        updates.push(format!("email = '{}'", email));
    }
    if let Some(full_name) = &data.full_name {
        updates.push(format!("full_name = '{}'", full_name));
    }
    if let Some(role_id) = data.role_id {
        updates.push(format!("role_id = {}", role_id));
    }
    if let Some(is_active) = data.is_active {
        updates.push(format!("is_active = {}", is_active));
    }

    if updates.is_empty() {
        return get_user_with_role(pool, user_id).await;
    }

    updates.push("updated_at = NOW()".to_string());

    let query_str = format!(
        "UPDATE users SET {} WHERE id = {}",
        updates.join(", "),
        user_id
    );

    sqlx::query(&query_str).execute(pool).await?;

    get_user_with_role(pool, user_id).await
}

/// Delete user (deactivate)
pub async fn delete_user(pool: &PgPool, user_id: i32) -> Result<(), AppError> {
    sqlx::query(
        r#"
        UPDATE users
        SET is_active = FALSE, updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    Ok(())
}

// ==================== ROLE MANAGEMENT FUNCTIONS ====================

/// List all roles
pub async fn list_roles(pool: &PgPool) -> Result<Vec<Role>, AppError> {
    let roles = sqlx::query_as::<_, Role>(
        r#"
        SELECT id, name, description, created_at, updated_at
        FROM roles
        ORDER BY name
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(roles)
}

/// Get role with permissions
pub async fn get_role_with_permissions(
    pool: &PgPool,
    role_id: i32,
) -> Result<RoleWithPermissions, AppError> {
    let role = sqlx::query_as::<_, Role>(
        r#"
        SELECT id, name, description, created_at, updated_at
        FROM roles
        WHERE id = $1
        "#,
    )
    .bind(role_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::NotFound(format!("Role with id {} not found", role_id)))?;

    let permissions = sqlx::query_as::<_, Permission>(
        r#"
        SELECT p.id, p.name, p.description, p.resource, p.action, p.created_at
        FROM permissions p
        JOIN role_permissions rp ON p.id = rp.permission_id
        WHERE rp.role_id = $1
        ORDER BY p.name
        "#,
    )
    .bind(role_id)
    .fetch_all(pool)
    .await?;

    Ok(RoleWithPermissions {
        id: role.id,
        name: role.name,
        description: role.description,
        permissions,
        created_at: role.created_at,
        updated_at: role.updated_at,
    })
}

/// Get user permissions
pub async fn get_user_permissions(pool: &PgPool, user_id: i32) -> Result<Vec<String>, AppError> {
    let user = sqlx::query_as::<_, User>(
        "SELECT id, username, email, password_hash, full_name, role_id, is_active, last_login_at, created_at, updated_at, created_by FROM users WHERE id = $1"
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    let permissions = sqlx::query_scalar::<_, String>(
        r#"
        SELECT p.name
        FROM permissions p
        JOIN role_permissions rp ON p.id = rp.permission_id
        WHERE rp.role_id = $1
        ORDER BY p.name
        "#,
    )
    .bind(user.role_id)
    .fetch_all(pool)
    .await?;

    Ok(permissions)
}
