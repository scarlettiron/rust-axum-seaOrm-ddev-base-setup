mod auth;
mod admin;
mod config;
mod routes;

use axum::middleware;
use tower_http::trace::TraceLayer;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    //initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    //create application router with middleware
    let app = routes::create_router()
        .layer(middleware::from_fn(config::allowed_hosts_middleware))
        .layer(config::cors_layer())
        .layer(TraceLayer::new_for_http());

    //get port from environment or use default
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    tracing::info!("Server starting on {}", addr);

    //start server
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Server failed to start");
}
