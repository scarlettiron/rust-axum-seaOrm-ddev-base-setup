use prometheus::{HistogramOpts, HistogramVec, IntCounterVec, IntGauge, Opts, Registry};
use std::sync::OnceLock;

pub static REGISTRY: OnceLock<Registry> = OnceLock::new();
pub static HTTP_REQUESTS_TOTAL: OnceLock<IntCounterVec> = OnceLock::new();
pub static HTTP_REQUEST_DURATION: OnceLock<HistogramVec> = OnceLock::new();
pub static HTTP_REQUESTS_IN_FLIGHT: OnceLock<IntGauge> = OnceLock::new();

///initializes prometheus metrics registry and registers all metrics
pub fn init_metrics() {
    let registry = Registry::new();

    //total HTTP requests counter
    let http_requests_total = IntCounterVec::new(
        Opts::new("http_requests_total", "Total number of HTTP requests"),
        &["method", "path", "status"],
    )
    .expect("Failed to create http_requests_total metric");

    //HTTP request duration histogram
    let http_request_duration = HistogramVec::new(
        HistogramOpts::new("http_request_duration_seconds", "HTTP request duration in seconds")
            .buckets(vec![0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0]),
        &["method", "path"],
    )
    .expect("Failed to create http_request_duration metric");

    //requests currently being processed
    let http_requests_in_flight = IntGauge::new(
        "http_requests_in_flight",
        "Number of HTTP requests currently being processed",
    )
    .expect("Failed to create http_requests_in_flight metric");

    //register all metrics
    registry
        .register(Box::new(http_requests_total.clone()))
        .expect("Failed to register http_requests_total");
    registry
        .register(Box::new(http_request_duration.clone()))
        .expect("Failed to register http_request_duration");
    registry
        .register(Box::new(http_requests_in_flight.clone()))
        .expect("Failed to register http_requests_in_flight");

    //store in static variables
    REGISTRY.set(registry).expect("Failed to set registry");
    HTTP_REQUESTS_TOTAL
        .set(http_requests_total)
        .expect("Failed to set http_requests_total");
    HTTP_REQUEST_DURATION
        .set(http_request_duration)
        .expect("Failed to set http_request_duration");
    HTTP_REQUESTS_IN_FLIGHT
        .set(http_requests_in_flight)
        .expect("Failed to set http_requests_in_flight");

    tracing::info!("Prometheus metrics initialized");
}
