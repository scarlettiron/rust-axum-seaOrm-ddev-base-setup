//! QuickBooks Desktop (QBD) Web Connector .qwc generation and credential management.

use base64::Engine;
use entity::connection_identity;
use entity::erp_connection_credentials;
use entity::sea_orm_active_enums::{
    ErpEnvironment, ErpProvider, ErpProviderAuthType, ErpProviderType,
};
use sea_orm::{
    ActiveModelTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait, Set,
};
use uuid::Uuid;

use crate::connection_identity::services::{ConnectionIdentityService, CreateConnectionIdentity};
use crate::erp_connection_credentials::services::{
    CreateErpConnectionCredentials, ErpConnectionCredentialsService,
};
use crate::tenant::services::TenantService;

/// Template is read at compile time so we never overwrite it.
const QWC_TEMPLATE: &str = include_str!("./QBD_QBWC_TEMPLATE.qwc");

#[derive(Debug)]
pub enum QbdDesktopError {
    TenantNotFound,
    Db(DbErr),
}

impl From<DbErr> for QbdDesktopError {
    fn from(err: DbErr) -> Self {
        QbdDesktopError::Db(err)
    }
}

impl QbdDesktopError {
    /// HTTP status for this error.
    pub fn status_code(&self) -> axum::http::StatusCode {
        match self {
            QbdDesktopError::TenantNotFound => axum::http::StatusCode::NOT_FOUND,
            QbdDesktopError::Db(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// User-facing error message.
    pub fn message(&self) -> String {
        match self {
            QbdDesktopError::TenantNotFound => "Tenant not found".to_string(),
            QbdDesktopError::Db(e) => format!("Database error: {}", e),
        }
    }
}

/// Result of generating or retrieving QBD credentials and .qwc content.
pub struct QwcResult {
    pub tenant_id: String,
    pub username: String,
    pub password: String,
    pub file_id: String,
    pub qwc_xml: String,
}

/// Output for the generate-qwc API: all fields needed for the JSON response.
pub struct GenerateQwcOutput {
    pub tenant_id: String,
    pub password: String,
    pub qwc_file_base64: String,
    pub username: Option<String>,
    pub file_id: Option<String>,
}

/// Ensures a tenant exists. If `tenant_id` is None, creates a new tenant.
/// Returns the tenant's DB id and tenant_id string.
pub async fn ensure_tenant(
    db: &DatabaseConnection,
    tenant_id: Option<&str>,
    txn: Option<&DatabaseTransaction>,
) -> Result<(i64, String), QbdDesktopError> {
    let tenant_svc = TenantService::new(db.clone());
    match tenant_id {
        Some(id) => {
            let tenant = tenant_svc
                .get_by_tenant_id(id, txn)
                .await?
                .ok_or(QbdDesktopError::TenantNotFound)?;
            Ok((tenant.id, tenant.tenant_id))
        }
        None => {
            let tenant = tenant_svc
                .create(
                    crate::tenant::services::CreateTenant {
                        display_name: None,
                    },
                    txn,
                )
                .await?;
            Ok((tenant.id, tenant.tenant_id))
        }
    }
}

/// Generates a random username with prefix `pro_portals_`.
fn random_username() -> String {
    format!("pro_portals_{}", Uuid::new_v4().simple())
}

/// Generates a random password (alphanumeric-friendly for Web Connector).
fn random_password() -> String {
    Uuid::new_v4().simple().to_string()
}

/// Generates a random file id (GUID format for QBWC).
fn random_file_id() -> String {
    Uuid::new_v4().to_string()
}

/// Finds an existing QuickBooks Desktop connection for the tenant, if any.
async fn find_qbd_connection(
    db: &DatabaseConnection,
    tenant_db_id: i64,
    txn: Option<&DatabaseTransaction>,
) -> Result<
    Option<(connection_identity::Model, erp_connection_credentials::Model)>,
    DbErr,
> {
    let conn_svc = ConnectionIdentityService::new(db.clone());
    let connections = conn_svc
        .get_by_tenant_id(tenant_db_id, txn)
        .await?;
    let cred_svc = ErpConnectionCredentialsService::new(db.clone());
    for conn in connections {
        if conn.erp_provider != ErpProvider::Quickbooks || conn.erp_type != ErpProviderType::Desktop
        {
            continue;
        }
        if let Some(creds) = cred_svc.get_by_connection_id(conn.id, txn).await? {
            if creds.provider_user_id.is_some() && creds.provider_password.is_some() {
                return Ok(Some((conn, creds)));
            }
        }
    }
    Ok(None)
}

/// Gets or creates QuickBooks Desktop credentials for the tenant.
/// Returns (tenant_id_string, username, password, file_id, qwc_xml).
pub async fn get_or_create_qbd_credentials_and_qwc(
    db: &DatabaseConnection,
    tenant_db_id: i64,
    tenant_id_str: &str,
    txn: Option<&DatabaseTransaction>,
) -> Result<QwcResult, QbdDesktopError> {
    let conn_svc = ConnectionIdentityService::new(db.clone());
    let cred_svc = ErpConnectionCredentialsService::new(db.clone());

    if let Some((conn, creds)) = find_qbd_connection(db, tenant_db_id, txn).await? {
        let username = creds.provider_user_id.unwrap_or_default();
        let password = creds.provider_password.unwrap_or_default();
        let file_id = conn
            .company_file_id
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let qwc_xml = format_qwc_template(&username, &password, &file_id);
        return Ok(QwcResult {
            tenant_id: tenant_id_str.to_string(),
            username,
            password,
            file_id,
            qwc_xml,
        });
    }

    let username = random_username();
    let password = random_password();
    let file_id = random_file_id();

    let connection = conn_svc
        .create(
            CreateConnectionIdentity {
                tenant_id: tenant_db_id,
                erp_provider: ErpProvider::Quickbooks,
                erp_type: ErpProviderType::Desktop,
                erp_auth_type: ErpProviderAuthType::UsernamePassword,
                display_name: Some("QuickBooks Desktop Web Connector".to_string()),
                environment: Some(ErpEnvironment::Production),
                scopes: None,
                provider_realm_id: None,
                provider_tenant_id: None,
                company_file_identity: None,
                company_file_path: None,
                company_file_id: Some(file_id.clone()),
                system_version: None,
                web_connector_app_name: None,
                secret_storage_ref: None,
                secret_version: None,
                sync_enabled_push: Some(true),
                sync_enabled_pull: Some(true),
            },
            txn,
        )
        .await?;

    let _creds = cred_svc
        .create(
            CreateErpConnectionCredentials {
                connection_id: connection.id,
                client_id: None,
                issuer_base_url: None,
                token_type: None,
                reauth_required_reason: None,
                reauth_url: None,
                enc_scheme: Some("none".to_string()),
                enc_key_id: "qbd-webconnector".to_string(),
                enc_version: Some(1),
                enc_iv: None,
                enc_tag: None,
                access_token: None,
                refresh_token: None,
                access_token_expires_at: None,
                refresh_token_expires_at: None,
                id_token_enc: None,
                provider_user_id: Some(username.clone()),
                provider_password: Some(password.clone()),
                client_cert: None,
                private_key: None,
                cert_expires_at: None,
                session_token: None,
                session_expires_at: None,
                api_access_token: None,
                api_access_token_key: None,
            },
            txn,
        )
        .await?;

    let qwc_xml = format_qwc_template(&username, &password, &file_id);
    Ok(QwcResult {
        tenant_id: tenant_id_str.to_string(),
        username,
        password,
        file_id,
        qwc_xml,
    })
}

/// Fills the .qwc template with username, password, and file_id.
/// Template placeholders: {{username}}, {{fileid}}. Password is not in the .qwc file.
fn format_qwc_template(username: &str, _password: &str, file_id: &str) -> String {
    QWC_TEMPLATE
        .replace("{{username}}", username)
        .replace("{{fileid}}", file_id)
}

/// Full flow: ensure tenant (create if no tenant_id), get or create QBD credentials,
/// build .qwc XML, base64-encode it, and return the API output.
pub async fn generate_qwc(
    db: &DatabaseConnection,
    tenant_id: Option<&str>,
) -> Result<GenerateQwcOutput, QbdDesktopError> {
    let (tenant_db_id, tenant_id_str) = ensure_tenant(db, tenant_id, None).await?;
    let result =
        get_or_create_qbd_credentials_and_qwc(db, tenant_db_id, &tenant_id_str, None).await?;
    let qwc_file_base64 = base64::engine::general_purpose::STANDARD.encode(result.qwc_xml.as_bytes());
    Ok(GenerateQwcOutput {
        tenant_id: result.tenant_id,
        password: result.password,
        qwc_file_base64,
        username: Some(result.username),
        file_id: Some(result.file_id),
    })
}
