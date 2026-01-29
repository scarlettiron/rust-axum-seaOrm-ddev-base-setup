use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::AppState;
use super::services::{CreateTenant, TenantFilter, TenantService, UpdateTenant};
use entity::sea_orm_active_enums::Enum as TenantStatus;


/// RESPONSE SCHEMAS ///
#[derive(Serialize, ToSchema)]
pub struct TenantResponse {
    pub id: i64,
    pub uuid: String,
    pub tenant_id: String,
    pub display_name: Option<String>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Serialize, ToSchema)]
pub struct PaginatedTenantsResponse {
    pub items: Vec<TenantResponse>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

#[derive(Serialize, ToSchema)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Serialize, ToSchema)]
pub struct DeleteResponse {
    pub message: String,
}


/// REQUEST SCHEMAS ///
#[derive(Deserialize, ToSchema)]
pub struct CreateTenantRequest {
    pub display_name: Option<String>,
}

#[derive(Deserialize, ToSchema)]
pub struct UpdateTenantRequest {
    pub display_name: Option<String>,
    pub status: Option<String>,
}

#[derive(Deserialize, IntoParams)]
pub struct ListTenantsQuery {
    #[param(default = 1)]
    pub page: Option<u64>,
    #[param(default = 20)]
    pub per_page: Option<u64>,
    pub status: Option<String>,
    pub display_name: Option<String>,
    pub tenant_id: Option<String>,
}


/// HELPER FUNCTIONS ///
fn model_to_response(model: entity::tenant::Model) -> TenantResponse {
    TenantResponse {
        id: model.id,
        uuid: model.uuid.to_string(),
        tenant_id: model.tenant_id,
        display_name: model.display_name,
        status: format!("{:?}", model.status).to_lowercase(),
        created_at: model.created_at.to_rfc3339(),
        updated_at: model.updated_at.to_rfc3339(),
    }
}

fn parse_status(status: &str) -> Option<TenantStatus> {
    match status.to_lowercase().as_str() {
        "active" => Some(TenantStatus::Active),
        "removed" => Some(TenantStatus::Removed),
        _ => None,
    }
}


/// ROUTE HANDLERS ///

#[utoipa::path(
    get,
    path = "/all",
    tag = "Tenant",
    params(ListTenantsQuery),
    responses(
        (status = 200, description = "List of tenants", body = PaginatedTenantsResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    )
)]
pub async fn list_tenants(
    State(state): State<AppState>,
    Query(query): Query<ListTenantsQuery>,
) -> Result<Json<PaginatedTenantsResponse>, (StatusCode, Json<ErrorResponse>)> {
    let service = TenantService::new(state.db);

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    let filter = if query.status.is_some() || query.display_name.is_some() || query.tenant_id.is_some() {
        Some(TenantFilter {
            status: query.status.and_then(|s| parse_status(&s)),
            display_name: query.display_name,
            tenant_id: query.tenant_id,
        })
    } else {
        None
    };

    match service.get_all(page, per_page, filter, None).await {
        Ok(result) => Ok(Json(PaginatedTenantsResponse {
            items: result.items.into_iter().map(model_to_response).collect(),
            total: result.total,
            page: result.page,
            per_page: result.per_page,
            total_pages: result.total_pages,
        })),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {}", e),
            }),
        )),
    }
}

#[utoipa::path(
    get,
    path = "/get/{tenant_id}",
    tag = "Tenant",
    params(
        ("tenant_id" = String, Path, description = "Tenant ID (TN_xxx format)")
    ),
    responses(
        (status = 200, description = "Tenant found", body = TenantResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Tenant not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ))]
pub async fn get_tenant(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
) -> Result<Json<TenantResponse>, (StatusCode, Json<ErrorResponse>)> {
    let service = TenantService::new(state.db);

    match service.get_by_tenant_id(&tenant_id, None).await {
        Ok(Some(tenant)) => Ok(Json(model_to_response(tenant))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Tenant not found".to_string(),
            }),
        )),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {}", e),
            }),
        )),
    }
}


#[utoipa::path(
    post,
    path = "/create",
    tag = "Tenant",
    request_body = CreateTenantRequest,
    responses(
        (status = 201, description = "Tenant created", body = TenantResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ))]
pub async fn create_tenant(
    State(state): State<AppState>,
    Json(body): Json<CreateTenantRequest>,
) -> Result<(StatusCode, Json<TenantResponse>), (StatusCode, Json<ErrorResponse>)> {
    let service = TenantService::new(state.db);

    let data = CreateTenant {
        display_name: body.display_name,
    };

    match service.create(data, None).await {
        Ok(tenant) => Ok((StatusCode::CREATED, Json(model_to_response(tenant)))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {}", e),
            }),
        )),
    }
}

#[utoipa::path(
    put,
    path = "/update/{tenant_id}",
    tag = "Tenant",
    params(
        ("tenant_id" = String, Path, description = "Tenant ID (TN_xxx format)")
    ),
    request_body = UpdateTenantRequest,
    responses(
        (status = 200, description = "Tenant updated", body = TenantResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Tenant not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ))]
pub async fn update_tenant(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
    Json(body): Json<UpdateTenantRequest>,
) -> Result<Json<TenantResponse>, (StatusCode, Json<ErrorResponse>)> {
    let service = TenantService::new(state.db);

    let patch = UpdateTenant {
        display_name: body.display_name,
        status: body.status.and_then(|s| parse_status(&s)),
    };

    match service.update_by_tenant_id(&tenant_id, patch, None).await {
        Ok(Some(tenant)) => Ok(Json(model_to_response(tenant))),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Tenant not found".to_string(),
            }),
        )),
        Err(super::services::TenantError::NotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Tenant not found".to_string(),
            }),
        )),
        Err(super::services::TenantError::Db(e)) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {}", e),
            }),
        )),
    }
}


#[utoipa::path(
    delete,
    path = "/remove/{tenant_id}",
    tag = "Tenant",
    params(
        ("tenant_id" = String, Path, description = "Tenant ID (TN_xxx format)")
    ),
    responses(
        (status = 200, description = "Tenant removed (soft delete)", body = DeleteResponse),
        (status = 401, description = "Unauthorized", body = ErrorResponse),
        (status = 404, description = "Tenant not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ))]
pub async fn delete_tenant(
    State(state): State<AppState>,
    Path(tenant_id): Path<String>,
) -> Result<Json<DeleteResponse>, (StatusCode, Json<ErrorResponse>)> {
    let service = TenantService::new(state.db);

    match service.delete_by_tenant_id(&tenant_id, None).await {
        Ok(Some(_)) => Ok(Json(DeleteResponse {
            message: "Tenant removed successfully".to_string(),
        })),
        Ok(None) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Tenant not found".to_string(),
            }),
        )),
        Err(super::services::TenantError::NotFound) => Err((
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Tenant not found".to_string(),
            }),
        )),
        Err(super::services::TenantError::Db(e)) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {}", e),
            }),
        )),
    }
}




/// ROUTER ///
pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/", get(list_tenants).post(create_tenant))
        .route("/{tenant_id}", get(get_tenant).put(update_tenant).delete(delete_tenant))
}
