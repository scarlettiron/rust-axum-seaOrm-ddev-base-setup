mod auth;
mod admin;
mod config;
mod routes;

use axum::middleware;
use sea_orm::DatabaseConnection;
use tower_http::trace::TraceLayer;
use tracing_subscriber;

///application state shared across all routes
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
}

#[tokio::main]
async fn main() {
    //load environment variables from .env file
    dotenvy::dotenv().ok();

    //initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    //connect to database
    let db = config::db_connect()
        .await
        .expect("Failed to connect to database");

    let state = AppState { db };

    //create application router with middleware
    let app = routes::create_router(state)
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
