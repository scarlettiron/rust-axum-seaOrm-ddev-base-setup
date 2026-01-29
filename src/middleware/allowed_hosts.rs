use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use crate::config::hosts::is_host_allowed;

///middleware that validates the Host header against allowed hosts
///returns 400 Bad Request if the host is not allowed
pub async fn allowed_hosts_middleware(
    request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    //get the Host header
    let host = request
        .headers()
        .get("host")
        .and_then(|h| h.to_str().ok())
        .unwrap_or("");

    if host.is_empty() || !is_host_allowed(host) {
        tracing::warn!("Blocked request from disallowed host: {}", host);
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(next.run(request).await)
}
