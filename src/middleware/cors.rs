use axum::http::HeaderName;
use tower_http::cors::{AllowOrigin, CorsLayer};

use crate::config::cors::{get_allow_credentials, get_allowed_headers, get_allowed_methods, get_allowed_origins};

///creates a configured CORS layer
pub fn cors_layer() -> CorsLayer {
    let origins = get_allowed_origins();

    let headers: Vec<HeaderName> = get_allowed_headers()
        .iter()
        .filter_map(|header| header.parse().ok())
        .collect();

    CorsLayer::new()
        .allow_origin(AllowOrigin::list(origins))
        .allow_methods(get_allowed_methods())
        .allow_headers(headers)
        .allow_credentials(get_allow_credentials())
}
