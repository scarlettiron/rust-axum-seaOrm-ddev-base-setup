use utoipa::OpenApi;

use crate::routes::HealthCheckResponse;
use crate::auth::services::{health_check as auth_health_check, AuthHealthResponse};
use crate::admin::services::{health_check as admin_health_check, AdminHealthResponse};
use crate::tenant::routes::{
    TenantResponse, PaginatedTenantsResponse, ErrorResponse, DeleteResponse,
    CreateTenantRequest, UpdateTenantRequest,
};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::healthcheck,
        crate::auth::services::health_check,
        crate::admin::services::health_check,
        crate::tenant::routes::list_tenants,
        crate::tenant::routes::get_tenant,
        crate::tenant::routes::get_tenant_by_uuid,
        crate::tenant::routes::create_tenant,
        crate::tenant::routes::update_tenant,
        crate::tenant::routes::update_tenant_by_uuid,
        crate::tenant::routes::delete_tenant,
        crate::tenant::routes::delete_tenant_by_uuid,
    ),
    components(schemas(
        HealthCheckResponse,
        AuthHealthResponse,
        AdminHealthResponse,
        TenantResponse,
        PaginatedTenantsResponse,
        ErrorResponse,
        DeleteResponse,
        CreateTenantRequest,
        UpdateTenantRequest,
    )),
    tags(
        (name = "Health", description = "Health check endpoints"),
        (name = "Auth", description = "Authentication module endpoints"),
        (name = "Admin", description = "Admin module endpoints"),
        (name = "Tenant", description = "Tenant management endpoints"),
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
