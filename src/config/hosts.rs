use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use std::env;

///default allowed host if ALLOWED_HOSTS env var is not set
const DEFAULT_HOST: &str = "erp-proxy-server.ddev.site";

///gets allowed hosts from ALLOWED_HOSTS env var (comma-separated)
///falls back to default DDEV project host if not set
pub fn get_allowed_hosts() -> Vec<String> {
    match env::var("ALLOWED_HOSTS") {
        Ok(hosts) => hosts
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        Err(_) => vec![DEFAULT_HOST.to_string()],
    }
}

///checks if a host is allowed
fn is_host_allowed(host: &str) -> bool {
    let allowed = get_allowed_hosts();

    //strip port if present
    let host_without_port = host.split(':').next().unwrap_or(host);

    allowed.iter().any(|allowed_host| {
        //exact match
        host_without_port == allowed_host
        //wildcard subdomain match (e.g., ".ddev.site" matches "anything.ddev.site")
        || (allowed_host.starts_with('.') && host_without_port.ends_with(allowed_host))
        //match subdomains
        || host_without_port.ends_with(&format!(".{}", allowed_host))
    })
}

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
