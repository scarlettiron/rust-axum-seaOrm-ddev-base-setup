use axum::http::{HeaderValue, Method};
use super::env;

///gets allowed origins from central config as HeaderValues
pub fn get_allowed_origins() -> Vec<HeaderValue> {
    env::get().cors.allowed_origins
        .iter()
        .filter_map(|origin| origin.parse().ok())
        .collect()
}

///gets allowed methods from central config
pub fn get_allowed_methods() -> Vec<Method> {
    env::get().cors.allowed_methods.clone()
}

///gets allowed headers from central config
pub fn get_allowed_headers() -> Vec<String> {
    env::get().cors.allowed_headers.clone()
}

///gets whether credentials are allowed from central config
pub fn get_allow_credentials() -> bool {
    env::get().cors.allow_credentials
}
