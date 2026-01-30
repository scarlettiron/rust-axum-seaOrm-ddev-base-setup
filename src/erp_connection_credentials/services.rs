use entity::erp_connection_credentials;
use entity::sea_orm_active_enums::{ErpConnectionAuthTokenType, ErpConnectionReauthReason};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    QueryFilter, Set,
};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug)]
pub enum ErpConnectionCredentialsError {
    NotFound,
    Db(DbErr),
}

#[allow(dead_code)]
impl From<DbErr> for ErpConnectionCredentialsError {
    fn from(err: DbErr) -> Self {
        ErpConnectionCredentialsError::Db(err)
    }
}

#[allow(dead_code)]
pub struct ErpConnectionCredentialsService {
    db: DatabaseConnection,
}

#[allow(dead_code)]
pub struct CreateErpConnectionCredentials {
    pub connection_id: i64,
    pub client_id: Option<String>,
    pub issuer_base_url: Option<String>,
    pub token_type: Option<ErpConnectionAuthTokenType>,
    pub reauth_required_reason: Option<ErpConnectionReauthReason>,
    pub reauth_url: Option<String>,
    pub enc_scheme: Option<String>,
    pub enc_key_id: String,
    pub enc_version: Option<i32>,
    pub enc_iv: Option<Vec<u8>>,
    pub enc_tag: Option<Vec<u8>>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub access_token_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub refresh_token_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub id_token_enc: Option<String>,
    pub provider_user_id: Option<String>,
    pub provider_password: Option<String>,
    pub client_cert: Option<Vec<u8>>,
    pub private_key: Option<String>,
    pub cert_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub session_token: Option<String>,
    pub session_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub api_access_token: Option<String>,
    pub api_access_token_key: Option<String>,
}

#[allow(dead_code)]
pub struct UpdateErpConnectionCredentials {
    pub client_id: Option<String>,
    pub issuer_base_url: Option<String>,
    pub token_type: Option<ErpConnectionAuthTokenType>,
    pub reauth_required_reason: Option<ErpConnectionReauthReason>,
    pub reauth_url: Option<String>,
    pub enc_scheme: Option<String>,
    pub enc_key_id: Option<String>,
    pub enc_version: Option<i32>,
    pub enc_iv: Option<Vec<u8>>,
    pub enc_tag: Option<Vec<u8>>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub access_token_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub refresh_token_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub id_token_enc: Option<String>,
    pub provider_user_id: Option<String>,
    pub provider_password: Option<String>,
    pub client_cert: Option<Vec<u8>>,
    pub private_key: Option<String>,
    pub cert_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub session_token: Option<String>,
    pub session_expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub api_access_token: Option<String>,
    pub api_access_token_key: Option<String>,
}

#[allow(dead_code)]
impl ErpConnectionCredentialsService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get_by_id(
        &self,
        id: i64,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<erp_connection_credentials::Model>, DbErr> {
        match txn {
            Some(txn) => erp_connection_credentials::Entity::find_by_id(id).one(txn).await,
            None => erp_connection_credentials::Entity::find_by_id(id).one(&self.db).await,
        }
    }

    pub async fn get_by_uuid(
        &self,
        uuid: Uuid,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<erp_connection_credentials::Model>, DbErr> {
        match txn {
            Some(txn) => {
                erp_connection_credentials::Entity::find()
                    .filter(erp_connection_credentials::Column::Uuid.eq(uuid))
                    .one(txn)
                    .await
            }
            None => {
                erp_connection_credentials::Entity::find()
                    .filter(erp_connection_credentials::Column::Uuid.eq(uuid))
                    .one(&self.db)
                    .await
            }
        }
    }

    pub async fn get_by_connection_id(
        &self,
        connection_id: i64,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<erp_connection_credentials::Model>, DbErr> {
        match txn {
            Some(txn) => {
                erp_connection_credentials::Entity::find()
                    .filter(erp_connection_credentials::Column::ConnectionId.eq(connection_id))
                    .one(txn)
                    .await
            }
            None => {
                erp_connection_credentials::Entity::find()
                    .filter(erp_connection_credentials::Column::ConnectionId.eq(connection_id))
                    .one(&self.db)
                    .await
            }
        }
    }

    pub async fn create(
        &self,
        data: CreateErpConnectionCredentials,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<erp_connection_credentials::Model, DbErr> {
        let active = erp_connection_credentials::ActiveModel {
            connection_id: Set(data.connection_id),
            enc_scheme: Set(data.enc_scheme.unwrap_or_else(|| "kms-envelope-v1".to_string())),
            enc_key_id: Set(data.enc_key_id),
            enc_version: Set(data.enc_version.unwrap_or(1)),
            token_type: Set(data.token_type.unwrap_or(ErpConnectionAuthTokenType::Bearer)),
            client_id: Set(data.client_id),
            issuer_base_url: Set(data.issuer_base_url),
            reauth_required_reason: Set(data.reauth_required_reason),
            reauth_url: Set(data.reauth_url),
            enc_iv: Set(data.enc_iv),
            enc_tag: Set(data.enc_tag),
            access_token: Set(data.access_token),
            refresh_token: Set(data.refresh_token),
            access_token_expires_at: Set(data.access_token_expires_at.map(Into::into)),
            refresh_token_expires_at: Set(data.refresh_token_expires_at.map(Into::into)),
            id_token_enc: Set(data.id_token_enc),
            provider_user_id: Set(data.provider_user_id),
            provider_password: Set(data.provider_password),
            client_cert: Set(data.client_cert),
            private_key: Set(data.private_key),
            cert_expires_at: Set(data.cert_expires_at.map(Into::into)),
            session_token: Set(data.session_token),
            session_expires_at: Set(data.session_expires_at.map(Into::into)),
            api_access_token: Set(data.api_access_token),
            api_access_token_key: Set(data.api_access_token_key),
            ..Default::default()
        };

        match txn {
            Some(txn) => active.insert(txn).await,
            None => active.insert(&self.db).await,
        }
    }

    pub async fn update_by_uuid(
        &self,
        uuid: Uuid,
        patch: UpdateErpConnectionCredentials,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<erp_connection_credentials::Model>, ErpConnectionCredentialsError> {
        let model = match txn {
            Some(txn) => {
                erp_connection_credentials::Entity::find()
                    .filter(erp_connection_credentials::Column::Uuid.eq(uuid))
                    .one(txn)
                    .await?
            }
            None => {
                erp_connection_credentials::Entity::find()
                    .filter(erp_connection_credentials::Column::Uuid.eq(uuid))
                    .one(&self.db)
                    .await?
            }
        };

        let Some(model) = model else {
            return Err(ErpConnectionCredentialsError::NotFound);
        };

        let mut active: erp_connection_credentials::ActiveModel = model.into();
        apply_credentials_patch(&mut active, patch);
        active.updated_at = Set(chrono::Utc::now().into());

        match txn {
            Some(txn) => Ok(Some(active.update(txn).await?)),
            None => Ok(Some(active.update(&self.db).await?)),
        }
    }

    pub async fn update_by_connection_id(
        &self,
        connection_id: i64,
        patch: UpdateErpConnectionCredentials,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<erp_connection_credentials::Model>, ErpConnectionCredentialsError> {
        let model = match txn {
            Some(txn) => {
                erp_connection_credentials::Entity::find()
                    .filter(erp_connection_credentials::Column::ConnectionId.eq(connection_id))
                    .one(txn)
                    .await?
            }
            None => {
                erp_connection_credentials::Entity::find()
                    .filter(erp_connection_credentials::Column::ConnectionId.eq(connection_id))
                    .one(&self.db)
                    .await?
            }
        };

        let Some(model) = model else {
            return Err(ErpConnectionCredentialsError::NotFound);
        };

        let mut active: erp_connection_credentials::ActiveModel = model.into();
        apply_credentials_patch(&mut active, patch);
        active.updated_at = Set(chrono::Utc::now().into());

        match txn {
            Some(txn) => Ok(Some(active.update(txn).await?)),
            None => Ok(Some(active.update(&self.db).await?)),
        }
    }
}

fn apply_credentials_patch(
    active: &mut erp_connection_credentials::ActiveModel,
    patch: UpdateErpConnectionCredentials,
) {
    if let Some(v) = patch.client_id {
        active.client_id = Set(Some(v));
    }
    if let Some(v) = patch.issuer_base_url {
        active.issuer_base_url = Set(Some(v));
    }
    if let Some(v) = patch.token_type {
        active.token_type = Set(v);
    }
    if patch.reauth_required_reason.is_some() {
        active.reauth_required_reason = Set(patch.reauth_required_reason);
    }
    if let Some(v) = patch.reauth_url {
        active.reauth_url = Set(Some(v));
    }
    if let Some(v) = patch.enc_scheme {
        active.enc_scheme = Set(v);
    }
    if let Some(v) = patch.enc_key_id {
        active.enc_key_id = Set(v);
    }
    if let Some(v) = patch.enc_version {
        active.enc_version = Set(v);
    }
    if patch.enc_iv.is_some() {
        active.enc_iv = Set(patch.enc_iv);
    }
    if patch.enc_tag.is_some() {
        active.enc_tag = Set(patch.enc_tag);
    }
    if patch.access_token.is_some() {
        active.access_token = Set(patch.access_token);
    }
    if patch.refresh_token.is_some() {
        active.refresh_token = Set(patch.refresh_token);
    }
    if patch.access_token_expires_at.is_some() {
        active.access_token_expires_at = Set(patch.access_token_expires_at.map(Into::into));
    }
    if patch.refresh_token_expires_at.is_some() {
        active.refresh_token_expires_at = Set(patch.refresh_token_expires_at.map(Into::into));
    }
    if patch.id_token_enc.is_some() {
        active.id_token_enc = Set(patch.id_token_enc);
    }
    if patch.provider_user_id.is_some() {
        active.provider_user_id = Set(patch.provider_user_id);
    }
    if patch.provider_password.is_some() {
        active.provider_password = Set(patch.provider_password);
    }
    if patch.client_cert.is_some() {
        active.client_cert = Set(patch.client_cert);
    }
    if patch.private_key.is_some() {
        active.private_key = Set(patch.private_key);
    }
    if patch.cert_expires_at.is_some() {
        active.cert_expires_at = Set(patch.cert_expires_at.map(Into::into));
    }
    if patch.session_token.is_some() {
        active.session_token = Set(patch.session_token);
    }
    if patch.session_expires_at.is_some() {
        active.session_expires_at = Set(patch.session_expires_at.map(Into::into));
    }
    if patch.api_access_token.is_some() {
        active.api_access_token = Set(patch.api_access_token);
    }
    if patch.api_access_token_key.is_some() {
        active.api_access_token_key = Set(patch.api_access_token_key);
    }
}
