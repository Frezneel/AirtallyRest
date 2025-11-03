use axum::{
    extract::{Request, State},
    http::{StatusCode, HeaderValue},
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;

/// JWT Authentication Middleware
///
/// Extracts and validates JWT token from Authorization header.
/// Adds user_id to request extensions for use in handlers.
pub async fn jwt_auth_middleware(
    State(pool): State<PgPool>,
    mut req: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract token from Authorization header
    let auth_header = req
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Check Bearer prefix
    let token = auth_header
        .strip_prefix("Bearer ")
        .ok_or(StatusCode::UNAUTHORIZED)?;

    // Verify token and get user_id
    let user_id = crate::database_auth::verify_token(&pool, token)
        .await
        .map_err(|e| {
            tracing::warn!("JWT verification failed: {:?}", e);
            StatusCode::UNAUTHORIZED
        })?;

    // Add user_id to request extensions
    req.extensions_mut().insert(user_id);

    // Add security headers to response
    let mut response = next.run(req).await;
    let headers = response.headers_mut();

    headers.insert(
        "X-Content-Type-Options",
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        "X-Frame-Options",
        HeaderValue::from_static("SAMEORIGIN"),
    );
    headers.insert(
        "X-XSS-Protection",
        HeaderValue::from_static("1; mode=block"),
    );

    Ok(response)
}

/// Permission check middleware
///
/// Verifies that the authenticated user has the required permission.
pub async fn require_permission(
    permission: String,
) -> impl Fn(State<PgPool>, Request, Next) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Response, StatusCode>> + Send>> + Clone {
    move |State(pool): State<PgPool>, req: Request, next: Next| {
        let perm = permission.clone();
        Box::pin(async move {
            // Get user_id from extensions (set by jwt_auth_middleware)
            let user_id = req
                .extensions()
                .get::<i32>()
                .copied()
                .ok_or(StatusCode::UNAUTHORIZED)?;

            // Get user permissions
            let permissions = crate::database_auth::get_user_permissions(&pool, user_id)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to get user permissions: {:?}", e);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;

            // Check if user has required permission or is superuser
            let has_permission = permissions.contains(&perm)
                || permissions.iter().any(|p| p.starts_with("system."));

            if !has_permission {
                tracing::warn!(
                    user_id = user_id,
                    required_permission = %perm,
                    "Insufficient permissions"
                );
                return Err(StatusCode::FORBIDDEN);
            }

            Ok(next.run(req).await)
        })
    }
}
