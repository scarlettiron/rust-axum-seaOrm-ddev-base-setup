use axum::{routing::get, Router, http::StatusCode, Json};
use serde_json::{json, Value};
use crate::AppState;

async fn healthcheck() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "message": "Application Up And Running"
        }))
    )
}

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .route("/healthcheck", get(healthcheck))
        .route("/", get(healthcheck))
        .nest("/auth", crate::auth::create_router())
        .nest("/admin", crate::admin::create_router())
        .with_state(state)
}
