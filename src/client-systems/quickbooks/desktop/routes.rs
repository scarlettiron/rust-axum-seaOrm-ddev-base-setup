//! QuickBooks Desktop Web Connector .qwc generation endpoint.

use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::client_systems::quickbooks::desktop::services::{generate_qwc, QbdDesktopError};
use crate::AppState;

#[derive(Debug, Deserialize, ToSchema)]
pub struct GenerateQwcRequest {
    /// If omitted, a new tenant is created and used.
    pub tenant_id: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GenerateQwcResponse {
    pub tenant_id: String,
    pub password: String,
    /// Base64-encoded .qwc file content. Decode and save as e.g. `Pro_Portals_ERP_Connector.qwc`.
    pub qwc_file_base64: String,
    /// Web Connector username (for reference; also embedded in the .qwc file).
    pub username: Option<String>,
    /// File ID (for reference; also embedded in the .qwc file).
    pub file_id: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct GenerateQwcErrorResponse {
    pub error: String,
}

#[utoipa::path(
    post,
    path = "/qwc",
    tag = "QuickBooks Desktop",
    request_body = GenerateQwcRequest,
    responses(
        (status = 200, description = "QWC file and credentials", body = GenerateQwcResponse),
        (status = 404, description = "Tenant not found", body = GenerateQwcErrorResponse),
        (status = 500, description = "Internal server error", body = GenerateQwcErrorResponse)
    )
)]
pub async fn generate_qwc_handler(
    State(state): State<AppState>,
    Json(body): Json<GenerateQwcRequest>,
) -> Result<Json<GenerateQwcResponse>, (StatusCode, Json<GenerateQwcErrorResponse>)> {
    let out = generate_qwc(&state.db, body.tenant_id.as_deref())
        .await
        .map_err(|e| {
            (
                e.status_code(),
                Json(GenerateQwcErrorResponse {
                    error: e.message(),
                }),
            )
        })?;
    Ok(Json(GenerateQwcResponse {
        tenant_id: out.tenant_id,
        password: out.password,
        qwc_file_base64: out.qwc_file_base64,
        username: out.username,
        file_id: out.file_id,
    }))
}

pub fn create_router() -> Router<AppState> {
    Router::new().route("/qwc", post(generate_qwc_handler))
}
