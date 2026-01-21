use axum::http::{HeaderName, HeaderValue, Method};
use std::env;
use tower_http::cors::{AllowOrigin, CorsLayer};

/// Default origin if CORS_ALLOWED_ORIGINS env var is not set
const DEFAULT_ORIGIN: &str = "https://erp-proxy-server.ddev.site";

/// Allowed HTTP methods
const ALLOWED_METHODS: &[Method] = &[
    Method::GET,
    Method::POST,
    Method::PUT,
    Method::DELETE,
    Method::OPTIONS,
    Method::PATCH,
];

/// Allowed headers
const ALLOWED_HEADERS: &[&str] = &[
    "authorization",
    "content-type",
    "x-requested-with",
    "x-custom-host",
    "accept",
    "origin",
];

/// Gets allowed origins from CORS_ALLOWED_ORIGINS env var (comma-separated)
/// Falls back to default DDEV project route if not set
fn get_allowed_origins() -> Vec<HeaderValue> {
    match env::var("CORS_ALLOWED_ORIGINS") {
        Ok(origins) => origins
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .filter_map(|origin| origin.parse().ok())
            .collect(),
        Err(_) => vec![DEFAULT_ORIGIN.parse().unwrap()],
    }
}

/// Creates a configured CORS layer
pub fn cors_layer() -> CorsLayer {
    let origins = get_allowed_origins();

    let headers: Vec<HeaderName> = ALLOWED_HEADERS
        .iter()
        .filter_map(|header| header.parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods(ALLOWED_METHODS.to_vec())
        .allow_headers(headers)
        .allow_credentials(true)
}
