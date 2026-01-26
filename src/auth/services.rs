use axum::{http::StatusCode, Json};
use serde_json::{json, Value};
use utoipa::ToSchema;

#[derive(ToSchema)]
pub struct AuthHealthResponse {
    status: String,
    module: String,
}


#[utoipa::path(
    get,
    path = "/auth/health",
    tag = "Auth",
    responses(
        (status = 200, description = "Auth module is healthy", body = AuthHealthResponse)
    )
)]
pub async fn health_check() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "module": "auth"
        }))
    )
}
