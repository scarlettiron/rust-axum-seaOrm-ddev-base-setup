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

    let routes = Router::new()
        .merge(swagger_ui)
        .route("/healthcheck", get(healthcheck))
        .route("/metrics", get(crate::middleware::metrics_handler))
        .nest("/auth", crate::auth::create_router())
        .nest("/admin", crate::admin::create_router())
        .nest("/tenant", crate::tenant::create_router())
        .nest(
            "/client-systems/quickbooks/desktop",
            crate::client_systems::quickbooks::desktop::create_router(),
        )
        .nest(
            "/poll/v1",
            crate::client_systems::quickbooks::desktop::create_poll_router(),
        )
        .route("/", get(healthcheck))
        .with_state(state);

    //if base_url is set, mount routes under both the prefix and root
    //so /api/healthcheck and /healthcheck both work
    match &crate::config::env::get().server.base_url {
        Some(base) if !base.is_empty() => {
            Router::new()
                .nest(base, routes)
        }
        _ => routes,
    }
}
