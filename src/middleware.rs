use axum::{
    body::{Body, Bytes},
    extract::Request,
    middleware::Next,
    response::Response,
};
use http_body_util::BodyExt;
use std::time::Instant;

/// Middleware untuk logging request dan response, khususnya 4xx errors
pub async fn logging_middleware(
    req: Request,
    next: Next,
) -> Response {
    let method = req.method().clone();
    let uri = req.uri().clone();
    let headers = req.headers().clone();
    let start = Instant::now();

    // Extract dan log request body untuk POST/PUT/PATCH
    let (parts, body) = req.into_parts();
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            tracing::error!("Failed to read request body: {:?}", e);
            Bytes::new()
        }
    };

    // Log incoming request dengan body
    let request_body_preview = if !bytes.is_empty() {
        String::from_utf8_lossy(&bytes[..bytes.len().min(1000)]).to_string()
    } else {
        "empty".to_string()
    };

    tracing::info!(
        method = %method,
        uri = %uri,
        content_type = ?headers.get("content-type"),
        body_size = bytes.len(),
        body_preview = %request_body_preview,
        "Incoming request"
    );

    // Reconstruct request with body
    let req = Request::from_parts(parts, Body::from(bytes));

    // Process request
    let response = next.run(req).await;

    let status = response.status();
    let duration = start.elapsed();

    // Extract response body untuk logging
    let (parts, body) = response.into_parts();
    let bytes = match body.collect().await {
        Ok(collected) => collected.to_bytes(),
        Err(e) => {
            tracing::error!("Failed to read response body: {:?}", e);
            Bytes::new()
        }
    };

    let response_body = if !bytes.is_empty() {
        String::from_utf8_lossy(&bytes).to_string()
    } else {
        "empty".to_string()
    };

    // Log berdasarkan status code
    match status.as_u16() {
        // 4xx Client Errors - Log dengan detail lengkap
        400..=499 => {
            tracing::warn!(
                method = %method,
                uri = %uri,
                status = %status.as_u16(),
                status_text = %status.canonical_reason().unwrap_or("Unknown"),
                duration_ms = ?duration.as_millis(),
                error_category = "CLIENT_ERROR",
                request_body = %request_body_preview,
                response_body = %response_body,
                "Request failed with client error (4xx)"
            );
        }
        // 5xx Server Errors - Log sebagai error dengan detail
        500..=599 => {
            tracing::error!(
                method = %method,
                uri = %uri,
                status = %status.as_u16(),
                status_text = %status.canonical_reason().unwrap_or("Unknown"),
                duration_ms = ?duration.as_millis(),
                error_category = "SERVER_ERROR",
                request_body = %request_body_preview,
                response_body = %response_body,
                "Request failed with server error (5xx)"
            );
        }
        // 2xx Success - Log dengan response body untuk POST
        200..=299 if method == "POST" || method == "PUT" => {
            tracing::info!(
                method = %method,
                uri = %uri,
                status = %status.as_u16(),
                duration_ms = ?duration.as_millis(),
                response_body = %response_body,
                "Request completed successfully"
            );
        }
        // 2xx Success - tanpa body
        200..=299 => {
            tracing::info!(
                method = %method,
                uri = %uri,
                status = %status.as_u16(),
                duration_ms = ?duration.as_millis(),
                "Request completed successfully"
            );
        }
        // Other status codes
        _ => {
            tracing::debug!(
                method = %method,
                uri = %uri,
                status = %status.as_u16(),
                duration_ms = ?duration.as_millis(),
                response_body = %response_body,
                "Request completed"
            );
        }
    }

    // Reconstruct response
    Response::from_parts(parts, Body::from(bytes))
}