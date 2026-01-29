use axum::{routing::get, Router, http::StatusCode, Json};
use serde_json::{json, Value};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use crate::AppState;
use crate::openapi::ApiDoc;

#[derive(utoipa::ToSchema)]
pub struct HealthCheckResponse {
    message: String,
}

#[utoipa::path(
    get,
    path = "/healthcheck",
    tag = "Health",
    responses(
        (status = 200, description = "Application is running", body = HealthCheckResponse)
    )
)]
pub async fn healthcheck() -> (StatusCode, Json<Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "message": "Application Up And Running"
        }))
    )
}

pub fn create_router(state: AppState) -> Router {
    let swagger_ui = SwaggerUi::new("/local/swagger-ui")
        .url("/api-doc/openapi.json", ApiDoc::openapi());
    
    Router::new()
        .merge(swagger_ui)
        .route("/healthcheck", get(healthcheck))
        .route("/metrics", get(crate::middleware::metrics_handler))
        .nest("/auth", crate::auth::create_router())
        .nest("/admin", crate::admin::create_router())
        .nest("/tenant", crate::tenant::create_router())
        .route("/", get(healthcheck))
        .with_state(state)
}
