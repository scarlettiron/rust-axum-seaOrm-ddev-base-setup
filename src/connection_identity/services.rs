use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use entity::connection_identity;
use entity::sea_orm_active_enums::{
    ErpConnectionAuthStatus, ErpConnectionStatus, ErpEnvironment,
    ErpProvider, ErpProviderAuthType, ErpProviderType,
};
use uuid::Uuid;


//DEBUG AND ERRORS ///
#[allow(dead_code)]
#[derive(Debug)]
pub enum ConnectionIdentityError {
    NotFound,
    Db(DbErr),
}

#[allow(dead_code)]
impl From<DbErr> for ConnectionIdentityError {
    fn from(err: DbErr) -> Self {
        ConnectionIdentityError::Db(err)
    }
}

//END DEBUG AND ERRORS


/// BEGUN STRUCTS AND ENUMS ///
pub struct ConnectionIdentityService {
    db: DatabaseConnection,
}

#[allow(dead_code)]
pub struct CreateConnectionIdentity {
    pub tenant_id: i64,
    pub erp_provider: ErpProvider,
    pub erp_type: ErpProviderType,
    pub erp_auth_type: ErpProviderAuthType,
    pub display_name: Option<String>,
    pub environment: Option<ErpEnvironment>,
    pub scopes: Option<Vec<String>>,
    pub provider_realm_id: Option<String>,
    pub provider_tenant_id: Option<String>,
    pub company_file_identity: Option<String>,
    pub company_file_path: Option<String>,
    pub company_file_id: Option<String>,
    pub system_version: Option<String>,
    pub web_connector_app_name: Option<String>,
    pub secret_storage_ref: Option<String>,
    pub secret_version: Option<String>,
    pub sync_enabled_push: Option<bool>,
    pub sync_enabled_pull: Option<bool>,
}

#[allow(dead_code)]
pub struct UpdateConnectionIdentity {
    pub display_name: Option<String>,
    pub environment: Option<ErpEnvironment>,
    pub status: Option<ErpConnectionStatus>,
    pub auth_status: Option<ErpConnectionAuthStatus>,
    pub is_enabled: Option<bool>,
    pub scopes: Option<Vec<String>>,
    pub provider_realm_id: Option<String>,
    pub provider_tenant_id: Option<String>,
    pub company_file_identity: Option<String>,
    pub company_file_path: Option<String>,
    pub company_file_id: Option<String>,
    pub system_version: Option<String>,
    pub web_connector_app_name: Option<String>,
    pub secret_storage_ref: Option<String>,
    pub secret_version: Option<String>,
    pub sync_enabled_push: Option<bool>,
    pub sync_enabled_pull: Option<bool>,
    pub last_error_code: Option<String>,
    pub last_error_message: Option<String>,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct ConnectionIdentityFilter {
    pub tenant_id: Option<i64>,
    pub erp_provider: Option<ErpProvider>,
    pub erp_type: Option<ErpProviderType>,
    pub status: Option<ErpConnectionStatus>,
    pub auth_status: Option<ErpConnectionAuthStatus>,
    pub environment: Option<ErpEnvironment>,
    pub is_enabled: Option<bool>,
    pub display_name: Option<String>,
}

#[allow(dead_code)]
pub struct PaginatedConnectionIdentities {
    pub items: Vec<connection_identity::Model>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

/// END STRUCTS AND ENUMS ///


/// BEGUN IMPLEMENTATION ///
#[allow(dead_code)]
impl ConnectionIdentityService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get_by_id(
        &self,
        id: i64,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<connection_identity::Model>, DbErr> {
        match txn {
            Some(txn) => {
                connection_identity::Entity::find_by_id(id)
                    .one(txn)
                    .await
            }
            None => {
                connection_identity::Entity::find_by_id(id)
                    .one(&self.db)
                    .await
            }
        }
    }

    pub async fn get_by_uuid(
        &self,
        uuid: Uuid,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<connection_identity::Model>, DbErr> {
        match txn {
            Some(txn) => {
                connection_identity::Entity::find()
                    .filter(connection_identity::Column::Uuid.eq(uuid))
                    .one(txn)
                    .await
            }
            None => {
                connection_identity::Entity::find()
                    .filter(connection_identity::Column::Uuid.eq(uuid))
                    .one(&self.db)
                    .await
            }
        }
    }

    pub async fn get_all(
        &self,
        page: u64,
        per_page: u64,
        filter: Option<ConnectionIdentityFilter>,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<PaginatedConnectionIdentities, DbErr> {
        let mut condition = Condition::all();

        if let Some(f) = filter {
            if let Some(tenant_id) = f.tenant_id {
                condition = condition.add(connection_identity::Column::TenantId.eq(tenant_id));
            }
            if let Some(erp_provider) = f.erp_provider {
                condition = condition.add(connection_identity::Column::ErpProvider.eq(erp_provider));
            }
            if let Some(erp_type) = f.erp_type {
                condition = condition.add(connection_identity::Column::ErpType.eq(erp_type));
            }
            if let Some(status) = f.status {
                condition = condition.add(connection_identity::Column::Status.eq(status));
            }
            if let Some(auth_status) = f.auth_status {
                condition = condition.add(connection_identity::Column::AuthStatus.eq(auth_status));
            }
            if let Some(environment) = f.environment {
                condition = condition.add(connection_identity::Column::Environment.eq(environment));
            }
            if let Some(is_enabled) = f.is_enabled {
                condition = condition.add(connection_identity::Column::IsEnabled.eq(is_enabled));
            }
            if let Some(display_name) = f.display_name {
                condition = condition.add(connection_identity::Column::DisplayName.contains(&display_name));
            }
        }

        let query = connection_identity::Entity::find()
            .filter(condition)
            .order_by_desc(connection_identity::Column::CreatedAt);

        let total = match txn {
            Some(txn) => query.clone().count(txn).await?,
            None => query.clone().count(&self.db).await?,
        };

        let total_pages = (total as f64 / per_page as f64).ceil() as u64;

        let items = match txn {
            Some(txn) => {
                query
                    .paginate(txn, per_page)
                    .fetch_page(page.saturating_sub(1))
                    .await?
            }
            None => {
                query
                    .paginate(&self.db, per_page)
                    .fetch_page(page.saturating_sub(1))
                    .await?
            }
        };

        Ok(PaginatedConnectionIdentities {
            items,
            total,
            page,
            per_page,
            total_pages,
        })
    }

    pub async fn get_by_tenant_id(
        &self,
        tenant_id: i64,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Vec<connection_identity::Model>, DbErr> {
        let query = connection_identity::Entity::find()
            .filter(connection_identity::Column::TenantId.eq(tenant_id))
            .order_by_desc(connection_identity::Column::CreatedAt);

        match txn {
            Some(txn) => query.all(txn).await,
            None => query.all(&self.db).await,
        }
    }

    pub async fn create(
        &self,
        data: CreateConnectionIdentity,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<connection_identity::Model, DbErr> {
        let active = connection_identity::ActiveModel {
            tenant_id: Set(data.tenant_id),
            erp_provider: Set(data.erp_provider),
            erp_type: Set(data.erp_type),
            erp_auth_type: Set(data.erp_auth_type),
            display_name: Set(data.display_name),
            environment: Set(data.environment.unwrap_or(ErpEnvironment::Production)),
            status: Set(ErpConnectionStatus::Active),
            auth_status: Set(ErpConnectionAuthStatus::Connected),
            is_enabled: Set(true),
            sync_enabled_push: Set(data.sync_enabled_push.unwrap_or(true)),
            sync_enabled_pull: Set(data.sync_enabled_pull.unwrap_or(true)),
            scopes: Set(data.scopes),
            provider_realm_id: Set(data.provider_realm_id),
            provider_tenant_id: Set(data.provider_tenant_id),
            company_file_identity: Set(data.company_file_identity),
            company_file_path: Set(data.company_file_path),
            company_file_id: Set(data.company_file_id),
            system_version: Set(data.system_version),
            web_connector_app_name: Set(data.web_connector_app_name),
            secret_storage_ref: Set(data.secret_storage_ref),
            secret_version: Set(data.secret_version),
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
        patch: UpdateConnectionIdentity,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<connection_identity::Model>, ConnectionIdentityError> {
        let model = match txn {
            Some(txn) => {
                connection_identity::Entity::find()
                    .filter(connection_identity::Column::Uuid.eq(uuid))
                    .one(txn)
                    .await?
            }
            None => {
                connection_identity::Entity::find()
                    .filter(connection_identity::Column::Uuid.eq(uuid))
                    .one(&self.db)
                    .await?
            }
        };

        let Some(model) = model else {
            return Err(ConnectionIdentityError::NotFound);
        };

        let mut active: connection_identity::ActiveModel = model.into();

        if let Some(display_name) = patch.display_name {
            active.display_name = Set(Some(display_name));
        }
        if let Some(environment) = patch.environment {
            active.environment = Set(environment);
        }
        if let Some(status) = patch.status {
            active.status = Set(status);
        }
        if let Some(auth_status) = patch.auth_status {
            active.auth_status = Set(auth_status);
        }
        if let Some(is_enabled) = patch.is_enabled {
            active.is_enabled = Set(is_enabled);
        }
        if let Some(scopes) = patch.scopes {
            active.scopes = Set(Some(scopes));
        }
        if let Some(provider_realm_id) = patch.provider_realm_id {
            active.provider_realm_id = Set(Some(provider_realm_id));
        }
        if let Some(provider_tenant_id) = patch.provider_tenant_id {
            active.provider_tenant_id = Set(Some(provider_tenant_id));
        }
        if let Some(company_file_identity) = patch.company_file_identity {
            active.company_file_identity = Set(Some(company_file_identity));
        }
        if let Some(company_file_path) = patch.company_file_path {
            active.company_file_path = Set(Some(company_file_path));
        }
        if let Some(company_file_id) = patch.company_file_id {
            active.company_file_id = Set(Some(company_file_id));
        }
        if let Some(system_version) = patch.system_version {
            active.system_version = Set(Some(system_version));
        }
        if let Some(web_connector_app_name) = patch.web_connector_app_name {
            active.web_connector_app_name = Set(Some(web_connector_app_name));
        }
        if let Some(secret_storage_ref) = patch.secret_storage_ref {
            active.secret_storage_ref = Set(Some(secret_storage_ref));
        }
        if let Some(secret_version) = patch.secret_version {
            active.secret_version = Set(Some(secret_version));
        }
        if let Some(sync_enabled_push) = patch.sync_enabled_push {
            active.sync_enabled_push = Set(sync_enabled_push);
        }
        if let Some(sync_enabled_pull) = patch.sync_enabled_pull {
            active.sync_enabled_pull = Set(sync_enabled_pull);
        }
        if let Some(last_error_code) = patch.last_error_code {
            active.last_error_code = Set(Some(last_error_code));
        }
        if let Some(last_error_message) = patch.last_error_message {
            active.last_error_message = Set(Some(last_error_message));
        }

        active.updated_at = Set(chrono::Utc::now().into());

        match txn {
            Some(txn) => Ok(Some(active.update(txn).await?)),
            None => Ok(Some(active.update(&self.db).await?)),
        }
    }

    ///soft delete - sets status to removed instead of deleting
    pub async fn delete_by_uuid(
        &self,
        uuid: Uuid,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<connection_identity::Model>, ConnectionIdentityError> {
        self.update_by_uuid(
            uuid,
            UpdateConnectionIdentity {
                status: Some(ErpConnectionStatus::Removed),
                display_name: None,
                environment: None,
                auth_status: None,
                is_enabled: None,
                scopes: None,
                provider_realm_id: None,
                provider_tenant_id: None,
                company_file_identity: None,
                company_file_path: None,
                company_file_id: None,
                system_version: None,
                web_connector_app_name: None,
                secret_storage_ref: None,
                secret_version: None,
                sync_enabled_push: None,
                sync_enabled_pull: None,
                last_error_code: None,
                last_error_message: None,
            },
            txn,
        )
        .await
    }

    ///records a successful sync/operation timestamp
    pub async fn record_success(
        &self,
        uuid: Uuid,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<connection_identity::Model>, ConnectionIdentityError> {
        let model = match txn {
            Some(txn) => {
                connection_identity::Entity::find()
                    .filter(connection_identity::Column::Uuid.eq(uuid))
                    .one(txn)
                    .await?
            }
            None => {
                connection_identity::Entity::find()
                    .filter(connection_identity::Column::Uuid.eq(uuid))
                    .one(&self.db)
                    .await?
            }
        };

        let Some(model) = model else {
            return Err(ConnectionIdentityError::NotFound);
        };

        let mut active: connection_identity::ActiveModel = model.into();
        active.last_success_at = Set(Some(chrono::Utc::now().into()));
        active.last_error_code = Set(None);
        active.last_error_message = Set(None);
        active.error_at = Set(None);
        active.auth_status = Set(ErpConnectionAuthStatus::Connected);
        active.updated_at = Set(chrono::Utc::now().into());

        match txn {
            Some(txn) => Ok(Some(active.update(txn).await?)),
            None => Ok(Some(active.update(&self.db).await?)),
        }
    }

    ///records an error on the connection
    pub async fn record_error(
        &self,
        uuid: Uuid,
        error_code: &str,
        error_message: &str,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<connection_identity::Model>, ConnectionIdentityError> {
        let model = match txn {
            Some(txn) => {
                connection_identity::Entity::find()
                    .filter(connection_identity::Column::Uuid.eq(uuid))
                    .one(txn)
                    .await?
            }
            None => {
                connection_identity::Entity::find()
                    .filter(connection_identity::Column::Uuid.eq(uuid))
                    .one(&self.db)
                    .await?
            }
        };

        let Some(model) = model else {
            return Err(ConnectionIdentityError::NotFound);
        };

        let mut active: connection_identity::ActiveModel = model.into();
        active.last_error_code = Set(Some(error_code.to_string()));
        active.last_error_message = Set(Some(error_message.to_string()));
        active.error_at = Set(Some(chrono::Utc::now().into()));
        active.auth_status = Set(ErpConnectionAuthStatus::Error);
        active.updated_at = Set(chrono::Utc::now().into());

        match txn {
            Some(txn) => Ok(Some(active.update(txn).await?)),
            None => Ok(Some(active.update(&self.db).await?)),
        }
    }
}
