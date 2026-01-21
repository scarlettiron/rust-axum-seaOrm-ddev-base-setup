pub mod routes;

use axum::{routing::get, Router};

pub fn create_router() -> Router {
    Router::new()
        .route("/health", get(routes::health_check))
}
