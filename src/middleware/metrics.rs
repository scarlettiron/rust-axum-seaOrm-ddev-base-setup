use axum::{
    body::Body,
    extract::MatchedPath,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
};
use prometheus::{Encoder, TextEncoder};
use std::time::Instant;

use crate::config::metrics::{
    HTTP_REQUESTS_IN_FLIGHT, HTTP_REQUESTS_TOTAL, HTTP_REQUEST_DURATION, REGISTRY,
};

///handler for /metrics endpoint - returns prometheus metrics in text format
pub async fn metrics_handler() -> impl IntoResponse {
    let registry = REGISTRY.get().expect("Metrics not initialized");
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();

    let mut buffer = Vec::new();
    encoder
        .encode(&metric_families, &mut buffer)
        .expect("Failed to encode metrics");

    (
        StatusCode::OK,
        [("content-type", "text/plain; charset=utf-8")],
        buffer,
    )
}

///middleware to track HTTP request metrics
pub async fn metrics_middleware(request: Request<Body>, next: Next) -> Response {
    let method = request.method().to_string();
    let path = request
        .extensions()
        .get::<MatchedPath>()
        .map(|p| p.as_str().to_string())
        .unwrap_or_else(|| request.uri().path().to_string());

    //increment in-flight requests
    if let Some(gauge) = HTTP_REQUESTS_IN_FLIGHT.get() {
        gauge.inc();
    }

    let start = Instant::now();
    let response = next.run(request).await;
    let duration = start.elapsed().as_secs_f64();

    let status = response.status().as_u16().to_string();

    //record metrics
    if let Some(counter) = HTTP_REQUESTS_TOTAL.get() {
        counter.with_label_values(&[&method, &path, &status]).inc();
    }

    if let Some(histogram) = HTTP_REQUEST_DURATION.get() {
        histogram
            .with_label_values(&[&method, &path])
            .observe(duration);
    }

    //decrement in-flight requests
    if let Some(gauge) = HTTP_REQUESTS_IN_FLIGHT.get() {
        gauge.dec();
    }

    response
}
