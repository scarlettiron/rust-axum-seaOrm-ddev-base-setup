//! QuickBooks Desktop Web Connector routes.
//!
//! Credential / .qwc generation:
//!   POST /client-systems/quickbooks/desktop/qwc
//!
//! Poll cycle (mounted at /poll/v1 in the main router):
//!   POST /poll/v1/qbwc         — request phase: returns QBXML for QBD to execute
//!   POST /poll/v1/qbwc/receive — response phase: processes QBD response, upserts records

use axum::{
    extract::State,
    http::{header, StatusCode},
    response::IntoResponse,
    routing::post,
    Json, Router,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::client_systems::quickbooks::desktop::poll_services::{
    PollResponseInput, PollResponseOutput, QbdPollError, QbdPollService,
};
use crate::client_systems::quickbooks::desktop::services::{generate_qwc, QbdDesktopError};
use crate::AppState;

// ── .qwc generation ───────────────────────────────────────────────────────────

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

// ── Poll: request phase ───────────────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct QbdPollRequestBody {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QbdPollRequestResponse {
    pub has_work: bool,
    /// QBXML to execute against QuickBooks Desktop. Null when has_work is false.
    pub xml: Option<String>,
}

/// POST /poll/v1/qbwc
///
/// Called by the QBWC adapter on each poll cycle.
/// Returns credentials-validated QBXML to execute against QuickBooks Desktop,
/// along with UUIDs that must be echoed back in the /receive call.
pub async fn qbwc_request_handler(
    State(state): State<AppState>,
    Json(body): Json<QbdPollRequestBody>,
) -> impl IntoResponse {
    let svc = QbdPollService::new(state.db.clone());
    match svc.handle_request(&body.username, &body.password).await {
        Ok(out) => Json(QbdPollRequestResponse {
            has_work: out.has_work,
            xml: out.xml,
        })
        .into_response(),
        Err(QbdPollError::Unauthorized) => StatusCode::FORBIDDEN.into_response(),
        Err(QbdPollError::Db(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {e}"),
        )
            .into_response(),
        Err(QbdPollError::XmlParse(e)) => {
            (StatusCode::INTERNAL_SERVER_ERROR, e).into_response()
        }
    }
}

// ── Poll: response phase ──────────────────────────────────────────────────────

#[derive(Debug, Deserialize, ToSchema)]
pub struct QbdPollReceiveBody {
    pub username: String,
    pub password: String,
    /// Full QBXML response string from QuickBooks Desktop. Mutually exclusive with qbd_error.
    pub qbd_response_xml: Option<String>,
    /// Error message from QBD (when QBD returned an error instead of XML).
    pub qbd_error: Option<String>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct QbdPollReceiveResponse {
    pub success: bool,
    /// True when QBWC should call sendRequestXML again immediately (more pages).
    /// Maps to QBWC's receiveResponseXML integer: 100 = keep going, 0 = done.
    pub has_more: bool,
    pub message: Option<String>,
}

/// POST /poll/v1/qbwc/receive
///
/// Called after QuickBooks Desktop executes the query and returns data.
/// Processes the response: upserts inventory records, updates the cursor,
/// and marks the sync event back to Pending (list) or Success (other).
pub async fn qbwc_receive_handler(
    State(state): State<AppState>,
    Json(body): Json<QbdPollReceiveBody>,
) -> impl IntoResponse {
    let svc = QbdPollService::new(state.db.clone());
    // Extract credentials before moving other fields into PollResponseInput.
    let username = body.username;
    let password = body.password;
    let input = PollResponseInput {
        qbd_response_xml: body.qbd_response_xml,
        qbd_error: body.qbd_error,
    };

    match svc.handle_response(&username, &password, input).await {
        Ok(PollResponseOutput { has_more }) => Json(QbdPollReceiveResponse {
            success: true,
            has_more,
            message: None,
        })
        .into_response(),
        Err(QbdPollError::Unauthorized) => StatusCode::FORBIDDEN.into_response(),
        Err(QbdPollError::Db(e)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(QbdPollReceiveResponse {
                success: false,
                has_more: false,
                message: Some(format!("Database error: {e}")),
            }),
        )
            .into_response(),
        Err(QbdPollError::XmlParse(e)) => (
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(QbdPollReceiveResponse {
                success: false,
                has_more: false,
                message: Some(e),
            }),
        )
            .into_response(),
    }
}

// ── Poll router (mounted at /poll/v1 in main routes) ──────────────────────────

pub fn create_poll_router() -> Router<AppState> {
    Router::new()
        .route("/qbwc", post(qbwc_request_handler))
        .route("/qbwc/receive", post(qbwc_receive_handler))
}
