use axum::{http::StatusCode, Json};
use serde_json::{json, Value};
use utoipa::ToSchema;

#[derive(ToSchema)]
pub struct AdminHealthResponse {
    status: String,
    module: String,
}

#[utoipa::path(
    get,
    path = "/admin/health",
    tag = "Admin",
    responses(
        (status = 200, description = "Admin module is healthy", body = AdminHealthResponse)
    )
)]
pub async fn health_check() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "module": "admin"
        }))
    )
}
