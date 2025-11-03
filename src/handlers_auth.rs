use crate::{
    database_auth,
    errors::AppError,
    models::{
        ApiResponse, LoginRequest, LoginResponse, CreateUserRequest, UpdateUserRequest,
        ChangePasswordRequest, User, UserWithRole, Role, RoleWithPermissions, ListUsersQuery,
    },
};
use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, HeaderMap},
    Json,
    Extension,
};
use sqlx::PgPool;
use validator::Validate;

// ==================== AUTHENTICATION HANDLERS ====================

/// Login handler
#[utoipa::path(
    post,
    path = "/api/auth/login",
    tag = "Authentication",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = LoginResponse),
        (status = 401, description = "Invalid credentials"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn login(
    State(pool): State<PgPool>,
    headers: HeaderMap,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<ApiResponse<LoginResponse>>, AppError> {
    tracing::info!(
        username = %payload.username,
        "Login attempt"
    );

    payload.validate()?;

    // Extract client IP for session tracking
    let ip_address = headers
        .get("X-Forwarded-For")
        .and_then(|v| v.to_str().ok())
        .or_else(|| headers.get("X-Real-IP").and_then(|v| v.to_str().ok()))
        .map(|s| s.to_string());

    let login_response = database_auth::authenticate_user(
        &pool,
        &payload.username,
        &payload.password,
        payload.device_info.clone(),
        ip_address,
    )
    .await?;

    tracing::info!(
        username = %payload.username,
        user_id = login_response.user.id,
        role = %login_response.user.role.name,
        "Login successful"
    );

    let response = ApiResponse {
        status: "success".to_string(),
        message: Some("Login successful".to_string()),
        data: Some(login_response),
        total: None,
    };

    Ok(Json(response))
}

/// Logout handler
#[utoipa::path(
    post,
    path = "/api/auth/logout",
    tag = "Authentication",
    responses(
        (status = 200, description = "Logout successful"),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn logout(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<i32>,
    headers: HeaderMap,
) -> Result<Json<ApiResponse<()>>, AppError> {
    tracing::info!(user_id = user_id, "Logout request");

    // Extract token from Authorization header
    let token = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| s.strip_prefix("Bearer "))
        .ok_or(AppError::Unauthorized("Missing or invalid token".to_string()))?;

    database_auth::revoke_session(&pool, token).await?;

    tracing::info!(user_id = user_id, "Logout successful");

    let response = ApiResponse {
        status: "success".to_string(),
        message: Some("Logout successful".to_string()),
        data: None,
        total: None,
    };

    Ok(Json(response))
}

/// Get current user profile
#[utoipa::path(
    get,
    path = "/api/auth/me",
    tag = "Authentication",
    responses(
        (status = 200, description = "User profile", body = UserWithRole),
        (status = 401, description = "Unauthorized"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_current_user(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<i32>,
) -> Result<Json<ApiResponse<UserWithRole>>, AppError> {
    let user = database_auth::get_user_with_role(&pool, user_id).await?;

    let response = ApiResponse {
        status: "success".to_string(),
        message: None,
        data: Some(user),
        total: None,
    };

    Ok(Json(response))
}

/// Change password
#[utoipa::path(
    post,
    path = "/api/auth/change-password",
    tag = "Authentication",
    request_body = ChangePasswordRequest,
    responses(
        (status = 200, description = "Password changed successfully"),
        (status = 400, description = "Validation error"),
        (status = 401, description = "Invalid old password"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn change_password(
    State(pool): State<PgPool>,
    Extension(user_id): Extension<i32>,
    Json(payload): Json<ChangePasswordRequest>,
) -> Result<Json<ApiResponse<()>>, AppError> {
    tracing::info!(user_id = user_id, "Password change request");

    payload.validate()?;

    database_auth::change_password(
        &pool,
        user_id,
        &payload.old_password,
        &payload.new_password,
    )
    .await?;

    tracing::info!(user_id = user_id, "Password changed successfully");

    let response = ApiResponse {
        status: "success".to_string(),
        message: Some("Password changed successfully".to_string()),
        data: None,
        total: None,
    };

    Ok(Json(response))
}

// ==================== USER MANAGEMENT HANDLERS ====================

/// Create new user (admin only)
#[utoipa::path(
    post,
    path = "/api/users",
    tag = "Users",
    request_body = CreateUserRequest,
    responses(
        (status = 201, description = "User created successfully", body = UserWithRole),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Insufficient permissions"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn create_user(
    State(pool): State<PgPool>,
    Extension(creator_id): Extension<i32>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<(StatusCode, Json<ApiResponse<UserWithRole>>), AppError> {
    tracing::info!(
        username = %payload.username,
        email = %payload.email,
        role_id = payload.role_id,
        "Creating new user"
    );

    payload.validate()?;

    let user = database_auth::create_user(&pool, payload, creator_id).await?;

    tracing::info!(
        user_id = user.id,
        username = %user.username,
        "User created successfully"
    );

    let response = ApiResponse {
        status: "success".to_string(),
        message: Some("User created successfully".to_string()),
        data: Some(user),
        total: None,
    };

    Ok((StatusCode::CREATED, Json(response)))
}

/// Get all users with filters
#[utoipa::path(
    get,
    path = "/api/users",
    tag = "Users",
    params(
        ("role_id" = Option<i32>, Query, description = "Filter by role ID"),
        ("is_active" = Option<bool>, Query, description = "Filter by active status"),
        ("limit" = Option<i64>, Query, description = "Limit results"),
        ("offset" = Option<i64>, Query, description = "Offset for pagination")
    ),
    responses(
        (status = 200, description = "List of users", body = Vec<UserWithRole>),
        (status = 403, description = "Insufficient permissions"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_users(
    State(pool): State<PgPool>,
    Query(query): Query<ListUsersQuery>,
) -> Result<Json<ApiResponse<Vec<UserWithRole>>>, AppError> {
    let (users, total) = database_auth::list_users(&pool, query).await?;

    let response = ApiResponse {
        status: "success".to_string(),
        message: None,
        data: Some(users),
        total: Some(total as u64),
    };

    Ok(Json(response))
}

/// Get user by ID
#[utoipa::path(
    get,
    path = "/api/users/{id}",
    tag = "Users",
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 200, description = "User details", body = UserWithRole),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_user_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<UserWithRole>>, AppError> {
    let user = database_auth::get_user_with_role(&pool, id).await?;

    let response = ApiResponse {
        status: "success".to_string(),
        message: None,
        data: Some(user),
        total: None,
    };

    Ok(Json(response))
}

/// Update user
#[utoipa::path(
    put,
    path = "/api/users/{id}",
    tag = "Users",
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    request_body = UpdateUserRequest,
    responses(
        (status = 200, description = "User updated successfully", body = UserWithRole),
        (status = 400, description = "Validation error"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn update_user(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Json<ApiResponse<UserWithRole>>, AppError> {
    tracing::info!(user_id = id, "Updating user");

    payload.validate()?;

    let user = database_auth::update_user(&pool, id, payload).await?;

    tracing::info!(user_id = id, "User updated successfully");

    let response = ApiResponse {
        status: "success".to_string(),
        message: Some("User updated successfully".to_string()),
        data: Some(user),
        total: None,
    };

    Ok(Json(response))
}

/// Delete user (soft delete by deactivating)
#[utoipa::path(
    delete,
    path = "/api/users/{id}",
    tag = "Users",
    params(
        ("id" = i32, Path, description = "User ID")
    ),
    responses(
        (status = 204, description = "User deleted successfully"),
        (status = 403, description = "Insufficient permissions"),
        (status = 404, description = "User not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn delete_user(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<StatusCode, AppError> {
    tracing::info!(user_id = id, "Deleting user");

    database_auth::delete_user(&pool, id).await?;

    tracing::info!(user_id = id, "User deleted successfully");

    Ok(StatusCode::NO_CONTENT)
}

// ==================== ROLE MANAGEMENT HANDLERS ====================

/// Get all roles
#[utoipa::path(
    get,
    path = "/api/roles",
    tag = "Roles",
    responses(
        (status = 200, description = "List of roles", body = Vec<Role>),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn list_roles(
    State(pool): State<PgPool>,
) -> Result<Json<ApiResponse<Vec<Role>>>, AppError> {
    let roles = database_auth::list_roles(&pool).await?;

    let response = ApiResponse {
        status: "success".to_string(),
        message: None,
        data: Some(roles),
        total: None,
    };

    Ok(Json(response))
}

/// Get role by ID with permissions
#[utoipa::path(
    get,
    path = "/api/roles/{id}",
    tag = "Roles",
    params(
        ("id" = i32, Path, description = "Role ID")
    ),
    responses(
        (status = 200, description = "Role details with permissions", body = RoleWithPermissions),
        (status = 404, description = "Role not found"),
        (status = 500, description = "Internal server error")
    ),
    security(
        ("bearer_auth" = [])
    )
)]
pub async fn get_role_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<Json<ApiResponse<RoleWithPermissions>>, AppError> {
    let role = database_auth::get_role_with_permissions(&pool, id).await?;

    let response = ApiResponse {
        status: "success".to_string(),
        message: None,
        data: Some(role),
        total: None,
    };

    Ok(Json(response))
}
