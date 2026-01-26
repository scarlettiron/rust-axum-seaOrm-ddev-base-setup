use utoipa::OpenApi;

use crate::routes::HealthCheckResponse;
use crate::auth::services::{health_check as auth_health_check, AuthHealthResponse};
use crate::admin::services::{health_check as admin_health_check, AdminHealthResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::healthcheck,
        crate::auth::services::health_check,
        crate::admin::services::health_check,
    ),
    components(schemas(
        HealthCheckResponse,
        AuthHealthResponse,
        AdminHealthResponse,
    )),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Auth", description = "Authentication module endpoints"),
        (name = "Admin", description = "Admin module endpoints"),
    ),
    info(
        title = "ERP Proxy Server API",
        description = "API documentation for the ERP Proxy Server",
        version = "1.0.0"
    ),
    servers(
        (url = "http://localhost:3000", description = "Local development server"),
        (url = "https://erp-proxy-server.ddev.site", description = "DDEV development server"),
    )
)]
pub struct ApiDoc;
