pub mod routes;

use axum::{routing::get, Router};
use crate::AppState;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(routes::health_check))
}
