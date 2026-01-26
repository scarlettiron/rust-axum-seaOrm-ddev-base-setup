mod auth;
mod admin;
mod config;
mod middleware;
mod openapi;
mod routes;
mod security;

use redis::aio::ConnectionManager;
use sea_orm::DatabaseConnection;
use tower_http::trace::TraceLayer;
use tracing_subscriber;

///application state shared across all routes
#[derive(Clone)]
pub struct AppState {
    pub db: DatabaseConnection,
    pub redis: ConnectionManager,
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

    //connect to Redis
    let redis = config::redis_connect()
        .await
        .expect("Failed to connect to Redis");

    let state = AppState { db, redis };

    //create application router with middleware
    //API token auth middleware is applied globally but skips public routes
    let app = routes::create_router(state.clone())
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::api_token_auth_middleware,
        ))
        .layer(axum::middleware::from_fn(config::allowed_hosts_middleware))
        .layer(config::cors_layer())
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(middleware::request_logging_middleware));

    //get port from environment or use default
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let addr = format!("0.0.0.0:{}", port);

    tracing::info!("Server starting on {}", addr);

    //start server
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app.into_make_service())
        .await
        .expect("Server failed to start");
}
