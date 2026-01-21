use axum::{
    body::Body,
    http::{Request, HeaderMap},
    middleware::Next,
    response::Response,
};
use std::env;
use std::time::Instant;

///headers that should never be logged for security reasons
const SENSITIVE_HEADERS: &[&str] = &[
    "authorization",
    "cookie",
    "set-cookie",
    "x-api-key",
    "x-auth-token",
    "x-access-token",
    "x-refresh-token",
    "proxy-authorization",
];

///checks if request logging is enabled via REQUEST_LOGGING env var
///defaults to true if not set
pub fn is_logging_enabled() -> bool {
    env::var("REQUEST_LOGGING")
        .map(|v| v.to_lowercase() != "false" && v != "0")
        .unwrap_or(true)
}

///filters sensitive headers from being logged
fn filter_headers(headers: &HeaderMap) -> Vec<(String, String)> {
    headers
        .iter()
        .filter(|(name, _)| {
            let name_lower = name.as_str().to_lowercase();
            !SENSITIVE_HEADERS.contains(&name_lower.as_str())
        })
        .map(|(name, value)| {
            (
                name.to_string(),
                value.to_str().unwrap_or("[binary]").to_string(),
            )
        })
        .collect()
}

///extracts client IP address from request headers or connection info
fn get_client_ip(request: &Request<Body>) -> String {
    //check x-forwarded-for header first (for proxied requests)
    if let Some(forwarded) = request.headers().get("x-forwarded-for") {
        if let Ok(value) = forwarded.to_str() {
            //take the first IP in the chain
            if let Some(ip) = value.split(',').next() {
                return ip.trim().to_string();
            }
        }
    }

    //check x-real-ip header
    if let Some(real_ip) = request.headers().get("x-real-ip") {
        if let Ok(value) = real_ip.to_str() {
            return value.to_string();
        }
    }

    //fallback to unknown
    "unknown".to_string()
}

///logging middleware that logs request details
///logs: method, path, timestamp, headers (filtered), direction, client IP
pub async fn request_logging_middleware(
    request: Request<Body>,
    next: Next,
) -> Response {
    if !is_logging_enabled() {
        return next.run(request).await;
    }

    let start_time = Instant::now();
    let timestamp = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();

    //extract request info before passing to handler
    let method = request.method().to_string();
    let path = request.uri().path().to_string();
    let client_ip = get_client_ip(&request);
    let request_headers = filter_headers(request.headers());

    //log incoming request
    tracing::info!(
        direction = "incoming",
        method = %method,
        path = %path,
        client_ip = %client_ip,
        timestamp = %timestamp,
        headers = ?request_headers,
        "Request received"
    );

    //process the request
    let response = next.run(request).await;

    //log outgoing response
    let duration = start_time.elapsed();
    let status = response.status().as_u16();
    let response_headers = filter_headers(response.headers());

    tracing::info!(
        direction = "outgoing",
        method = %method,
        path = %path,
        client_ip = %client_ip,
        status = %status,
        duration_ms = %duration.as_millis(),
        headers = ?response_headers,
        "Response sent"
    );

    response
}
