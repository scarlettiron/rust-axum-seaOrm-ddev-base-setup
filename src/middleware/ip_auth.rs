use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode, HeaderMap},
    middleware::Next,
    response::Response,
};
use axum::body::to_bytes;
use crate::AppState;
use crate::security::AllowedIpAddressService;

//extracts client IP address from request headers
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

//collects all headers as a string representation
fn format_headers(headers: &HeaderMap) -> String {
    headers
        .iter()
        .map(|(name, value)| {
            format!(
                "{}: {}",
                name,
                value.to_str().unwrap_or("[binary]")
            )
        })
        .collect::<Vec<_>>()
        .join(", ")
}

//extracts request body for logging
//note: This consumes the body, so we need to reconstruct it for the next handler
async fn extract_body(body: Body) -> (String, Body) {
    let body_bytes = match to_bytes(body, usize::MAX).await {
        Ok(bytes) => bytes,
        Err(_) => {
            return ("[Error reading body]".to_string(), Body::empty());
        }
    };

    let body_str = String::from_utf8_lossy(&body_bytes).to_string();
    let new_body = Body::from(body_bytes);

    (body_str, new_body)
}

//list of public routes that don't require IP address validation
fn is_public_route(path: &str) -> bool {
    let public_routes = [
        "/",
        "/healthcheck",
        "/metrics",
        "/local/swagger-ui",
        "/api-doc/openapi.json"
    ];

    public_routes.iter().any(|route| {
        path == *route || path.starts_with(&format!("{}/", route))
    })
}

//IP address validation middleware
//validates IP addresses on every request and logs unauthorized attempts critically
//skips validation for public routes
pub async fn ip_address_auth_middleware(
    State(state): State<AppState>,
    mut request: Request<Body>,
    next: Next,
) -> Response {
    let path = request.uri().path();
    
    //skip validation for public routes
    if is_public_route(path) {
        return next.run(request).await;
    }
    
    //extract client IP address
    let client_ip = get_client_ip(&request);
    
    //validate IP address
    let service = AllowedIpAddressService::new(state.db.clone());
    let is_allowed = match service.ip_address_allowed(&client_ip, None).await {
        Ok(allowed) => allowed,
        Err(e) => {
            //database error - log and reject
            tracing::error!(
                error = %e,
                "Database error while validating IP address"
            );
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal server error"))
                .unwrap();
        }
    };

    if !is_allowed {
        //IP address is not allowed - critically log all details
        let route = request.uri().path().to_string();
        let method = request.method().to_string();
        let headers = format_headers(request.headers());
        let query = request.uri().query().unwrap_or("").to_string();
        let full_path = if query.is_empty() {
            route.clone()
        } else {
            format!("{}?{}", route, query)
        };

        //extract body for logging (this consumes it, but we'll return error anyway so it's fine)
        let body = std::mem::replace(request.body_mut(), Body::empty());
        let (body_content, _) = extract_body(body).await;

        //critical log with all security-relevant information
        tracing::error!(
            severity = "CRITICAL",
            event = "unauthorized_ip_address_attempt",
            client_ip = %client_ip,
            route = %full_path,
            method = %method,
            headers = %headers,
            body = %body_content,
            "Unauthorized IP address attempt detected"
        );

        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from("Forbidden: IP address not allowed"))
            .unwrap();
    }

    //IP address is allowed - proceed with request (body is still intact since we didn't extract it)
    next.run(request).await
}
