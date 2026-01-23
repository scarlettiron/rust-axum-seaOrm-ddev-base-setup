use axum::{routing::get, Router};
use crate::AppState;
use super::services;

pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/health", get(services::health_check))
}
