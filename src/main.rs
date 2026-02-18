mod admin;
mod auth;
mod config;
mod connection_identity;
mod connection_run;
mod erp_connection_credentials;
mod erp_connection_sync_state;
mod inventory_records;
mod middleware;
mod openapi;
mod sync_event;
mod routes;
mod security;
mod tenant;

#[path = "client-systems/mod.rs"]
mod client_systems;

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

    //initialize central config from environment variables
    config::env::init();

    //initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    //initialize prometheus metrics
    config::init_metrics();

    //connect to database
    let db = config::db_connect()
        .await
        .expect("Failed to connect to database");

    //run pending migrations (idempotent; safe on every startup)
    migration::Migrator::up(&db, None)
        .await
        .expect("Failed to run migrations");

    //connect to Redis
    let redis = config::redis_connect()
        .await
        .expect("Failed to connect to Redis");

    let state = AppState { db, redis };

    //create application router with middleware
    let mut app = routes::create_router(state.clone());

    //apply API token authentication middleware if enabled
    if config::is_api_token_auth_enabled() {
        tracing::info!("API token authentication middleware enabled");
        app = app.layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::api_token_auth_middleware,
        ));
    } else {
        tracing::info!("API token authentication middleware disabled");
    }

    //apply IP address authentication middleware if enabled
    if config::is_ip_address_auth_enabled() {
        tracing::info!("IP address authentication middleware enabled");
        app = app.layer(axum::middleware::from_fn_with_state(
            state.clone(),
            middleware::ip_address_auth_middleware,
        ));
    } else {
        tracing::info!("IP address authentication middleware disabled");
    }

    //apply other middleware
    app = app
        .layer(axum::middleware::from_fn(middleware::allowed_hosts_middleware))
        .layer(middleware::cors_layer())
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn(middleware::request_logging_middleware))
        .layer(axum::middleware::from_fn(middleware::metrics_middleware));

    //get port from central config
    let port = &config::env::get().server.port;
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
