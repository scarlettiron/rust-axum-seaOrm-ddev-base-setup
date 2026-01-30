use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    QueryFilter, Set,
};
use sea_orm::entity::prelude::Json;
use entity::erp_connection_sync_state;
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug)]
pub enum ErpConnectionSyncStateError {
    NotFound,
    Db(DbErr),
}

#[allow(dead_code)]
impl From<DbErr> for ErpConnectionSyncStateError {
    fn from(err: DbErr) -> Self {
        ErpConnectionSyncStateError::Db(err)
    }
}

#[allow(dead_code)]
pub struct ErpConnectionSyncStateService {
    db: DatabaseConnection,
}

#[allow(dead_code)]
pub struct CreateErpConnectionSyncState {
    pub connection_id: i64,
    pub sync_cursor: Option<Json>,
    pub sync_lock_owner: Option<String>,
    pub sync_lock_until: Option<chrono::DateTime<chrono::Utc>>,
    pub rate_limit_remaining: Option<i32>,
    pub rate_limit: Option<i32>,
    pub rate_limit_reset_at: Option<chrono::DateTime<chrono::Utc>>,
    pub rate_limit_backoff_until: Option<chrono::DateTime<chrono::Utc>>,
    pub rate_limit_window_seconds: Option<i32>,
}

#[allow(dead_code)]
pub struct UpdateErpConnectionSyncState {
    pub sync_cursor: Option<Json>,
    pub sync_lock_owner: Option<String>,
    pub sync_lock_until: Option<chrono::DateTime<chrono::Utc>>,
    pub rate_limit_remaining: Option<i32>,
    pub rate_limit: Option<i32>,
    pub rate_limit_reset_at: Option<chrono::DateTime<chrono::Utc>>,
    pub rate_limit_backoff_until: Option<chrono::DateTime<chrono::Utc>>,
    pub rate_limit_window_seconds: Option<i32>,
}

#[allow(dead_code)]
impl ErpConnectionSyncStateService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get_by_id(
        &self,
        id: i64,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<erp_connection_sync_state::Model>, DbErr> {
        match txn {
            Some(txn) => erp_connection_sync_state::Entity::find_by_id(id).one(txn).await,
            None => erp_connection_sync_state::Entity::find_by_id(id).one(&self.db).await,
        }
    }

    pub async fn get_by_uuid(
        &self,
        uuid: Uuid,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<erp_connection_sync_state::Model>, DbErr> {
        match txn {
            Some(txn) => {
                erp_connection_sync_state::Entity::find()
                    .filter(erp_connection_sync_state::Column::Uuid.eq(uuid))
                    .one(txn)
                    .await
            }
            None => {
                erp_connection_sync_state::Entity::find()
                    .filter(erp_connection_sync_state::Column::Uuid.eq(uuid))
                    .one(&self.db)
                    .await
            }
        }
    }

    pub async fn get_by_connection_id(
        &self,
        connection_id: i64,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<erp_connection_sync_state::Model>, DbErr> {
        match txn {
            Some(txn) => {
                erp_connection_sync_state::Entity::find()
                    .filter(erp_connection_sync_state::Column::ConnectionId.eq(connection_id))
                    .one(txn)
                    .await
            }
            None => {
                erp_connection_sync_state::Entity::find()
                    .filter(erp_connection_sync_state::Column::ConnectionId.eq(connection_id))
                    .one(&self.db)
                    .await
            }
        }
    }

    pub async fn create(
        &self,
        data: CreateErpConnectionSyncState,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<erp_connection_sync_state::Model, DbErr> {
        let active = erp_connection_sync_state::ActiveModel {
            connection_id: Set(data.connection_id),
            sync_cursor: Set(data.sync_cursor),
            sync_lock_owner: Set(data.sync_lock_owner),
            sync_lock_until: Set(data.sync_lock_until.map(Into::into)),
            rate_limit_remaining: Set(data.rate_limit_remaining),
            rate_limit: Set(data.rate_limit),
            rate_limit_reset_at: Set(data.rate_limit_reset_at.map(Into::into)),
            rate_limit_backoff_until: Set(data.rate_limit_backoff_until.map(Into::into)),
            rate_limit_window_seconds: Set(data.rate_limit_window_seconds),
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
        patch: UpdateErpConnectionSyncState,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<erp_connection_sync_state::Model>, ErpConnectionSyncStateError> {
        let model = match txn {
            Some(txn) => {
                erp_connection_sync_state::Entity::find()
                    .filter(erp_connection_sync_state::Column::Uuid.eq(uuid))
                    .one(txn)
                    .await?
            }
            None => {
                erp_connection_sync_state::Entity::find()
                    .filter(erp_connection_sync_state::Column::Uuid.eq(uuid))
                    .one(&self.db)
                    .await?
            }
        };

        let Some(model) = model else {
            return Err(ErpConnectionSyncStateError::NotFound);
        };

        let mut active: erp_connection_sync_state::ActiveModel = model.into();

        if let Some(v) = patch.sync_cursor {
            active.sync_cursor = Set(Some(v));
        }
        if let Some(v) = patch.sync_lock_owner {
            active.sync_lock_owner = Set(Some(v));
        }
        if patch.sync_lock_until.is_some() {
            active.sync_lock_until = Set(patch.sync_lock_until.map(Into::into));
        }
        if patch.rate_limit_remaining.is_some() {
            active.rate_limit_remaining = Set(patch.rate_limit_remaining);
        }
        if patch.rate_limit.is_some() {
            active.rate_limit = Set(patch.rate_limit);
        }
        if patch.rate_limit_reset_at.is_some() {
            active.rate_limit_reset_at = Set(patch.rate_limit_reset_at.map(Into::into));
        }
        if patch.rate_limit_backoff_until.is_some() {
            active.rate_limit_backoff_until = Set(patch.rate_limit_backoff_until.map(Into::into));
        }
        if patch.rate_limit_window_seconds.is_some() {
            active.rate_limit_window_seconds = Set(patch.rate_limit_window_seconds);
        }

        active.updated_at = Set(chrono::Utc::now().into());

        match txn {
            Some(txn) => Ok(Some(active.update(txn).await?)),
            None => Ok(Some(active.update(&self.db).await?)),
        }
    }

    pub async fn update_by_connection_id(
        &self,
        connection_id: i64,
        patch: UpdateErpConnectionSyncState,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<erp_connection_sync_state::Model>, ErpConnectionSyncStateError> {
        let model = match txn {
            Some(txn) => {
                erp_connection_sync_state::Entity::find()
                    .filter(erp_connection_sync_state::Column::ConnectionId.eq(connection_id))
                    .one(txn)
                    .await?
            }
            None => {
                erp_connection_sync_state::Entity::find()
                    .filter(erp_connection_sync_state::Column::ConnectionId.eq(connection_id))
                    .one(&self.db)
                    .await?
            }
        };

        let Some(model) = model else {
            return Err(ErpConnectionSyncStateError::NotFound);
        };

        let mut active: erp_connection_sync_state::ActiveModel = model.into();

        if let Some(v) = patch.sync_cursor {
            active.sync_cursor = Set(Some(v));
        }
        if let Some(v) = patch.sync_lock_owner {
            active.sync_lock_owner = Set(Some(v));
        }
        if patch.sync_lock_until.is_some() {
            active.sync_lock_until = Set(patch.sync_lock_until.map(Into::into));
        }
        if patch.rate_limit_remaining.is_some() {
            active.rate_limit_remaining = Set(patch.rate_limit_remaining);
        }
        if patch.rate_limit.is_some() {
            active.rate_limit = Set(patch.rate_limit);
        }
        if patch.rate_limit_reset_at.is_some() {
            active.rate_limit_reset_at = Set(patch.rate_limit_reset_at.map(Into::into));
        }
        if patch.rate_limit_backoff_until.is_some() {
            active.rate_limit_backoff_until = Set(patch.rate_limit_backoff_until.map(Into::into));
        }
        if patch.rate_limit_window_seconds.is_some() {
            active.rate_limit_window_seconds = Set(patch.rate_limit_window_seconds);
        }

        active.updated_at = Set(chrono::Utc::now().into());

        match txn {
            Some(txn) => Ok(Some(active.update(txn).await?)),
            None => Ok(Some(active.update(&self.db).await?)),
        }
    }
}
