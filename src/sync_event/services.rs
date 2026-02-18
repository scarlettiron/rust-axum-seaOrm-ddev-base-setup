//! CRUD services for sync_event (no routes).
//!
//! When sync method is list and pagination is used: create a new sync event when the allotted
//! pagination span has been used (e.g. page size 25, 50 total → pull 25, then create a new sync
//! event for the next page). Track sync cursor via connection_sync_state or details. Multiple sync
//! events can happen in the same request. Create a new connection_sync_state per sync event ONLY
//! if the sync method is list.

use entity::sync_event;
use entity::sea_orm_active_enums::{
    SyncEventCategory, SyncEventDirection, SyncEventMethod, SyncEventStatus,
};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

//DEBUG AND ERRORS ///
#[allow(dead_code)]
#[derive(Debug)]
pub enum SyncEventError {
    NotFound,
    Db(DbErr),
}

#[allow(dead_code)]
impl From<DbErr> for SyncEventError {
    fn from(err: DbErr) -> Self {
        SyncEventError::Db(err)
    }
}

//END DEBUG AND ERRORS


/// BEGUN STRUCTS AND ENUMS ///
pub struct SyncEventService {
    db: DatabaseConnection,
}

#[allow(dead_code)]
pub struct CreateSyncEvent {
    pub original_record_body: Option<serde_json::Value>,
    pub details: Option<serde_json::Value>,
    pub event_direction: SyncEventDirection,
    pub inventory_record_event_id: Option<i64>,
    pub sync_event_method: SyncEventMethod,
    pub sync_event_category: SyncEventCategory,
    pub attempts: Option<i32>,
    pub status: Option<SyncEventStatus>,
    pub last_error: Option<serde_json::Value>,
    pub last_errored_date: Option<chrono::DateTime<chrono::Utc>>,
    pub connection_sync_state_id: Option<i64>,
}

#[allow(dead_code)]
pub struct UpdateSyncEvent {
    pub original_record_body: Option<serde_json::Value>,
    pub details: Option<serde_json::Value>,
    pub event_direction: Option<SyncEventDirection>,
    pub inventory_record_event_id: Option<i64>,
    pub sync_event_method: Option<SyncEventMethod>,
    pub sync_event_category: Option<SyncEventCategory>,
    pub attempts: Option<i32>,
    pub status: Option<SyncEventStatus>,
    pub last_error: Option<serde_json::Value>,
    pub last_errored_date: Option<chrono::DateTime<chrono::Utc>>,
    pub connection_sync_state_id: Option<i64>,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct SyncEventFilter {
    pub inventory_record_event_id: Option<i64>,
    pub connection_sync_state_id: Option<i64>,
    pub sync_event_method: Option<SyncEventMethod>,
    pub sync_event_category: Option<SyncEventCategory>,
    pub status: Option<SyncEventStatus>,
}

#[allow(dead_code)]
pub struct PaginatedSyncEvents {
    pub items: Vec<sync_event::Model>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

/// END STRUCTS AND ENUMS ///


/// BEGUN IMPLEMENTATION ///
#[allow(dead_code)]
impl SyncEventService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get_by_id(
        &self,
        id: i64,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<sync_event::Model>, DbErr> {
        match txn {
            Some(txn) => sync_event::Entity::find_by_id(id).one(txn).await,
            None => sync_event::Entity::find_by_id(id).one(&self.db).await,
        }
    }

    /// Get by uuid (idempotent key).
    pub async fn get_by_uuid(
        &self,
        uuid: Uuid,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<sync_event::Model>, DbErr> {
        match txn {
            Some(txn) => sync_event::Entity::find()
                .filter(sync_event::Column::Uuid.eq(uuid))
                .one(txn)
                .await,
            None => sync_event::Entity::find()
                .filter(sync_event::Column::Uuid.eq(uuid))
                .one(&self.db)
                .await,
        }
    }

    pub async fn get_by_inventory_record_event_id(
        &self,
        inventory_record_event_id: i64,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Vec<sync_event::Model>, DbErr> {
        let query = sync_event::Entity::find()
            .filter(sync_event::Column::InventoryRecordEventId.eq(inventory_record_event_id))
            .order_by_desc(sync_event::Column::CreatedAt);
        match txn {
            Some(txn) => query.all(txn).await,
            None => query.all(&self.db).await,
        }
    }

    pub async fn get_by_connection_sync_state_id(
        &self,
        connection_sync_state_id: i64,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Vec<sync_event::Model>, DbErr> {
        let query = sync_event::Entity::find()
            .filter(sync_event::Column::ConnectionSyncStateId.eq(connection_sync_state_id))
            .order_by_desc(sync_event::Column::CreatedAt);
        match txn {
            Some(txn) => query.all(txn).await,
            None => query.all(&self.db).await,
        }
    }

    pub async fn get_all(
        &self,
        page: u64,
        per_page: u64,
        filter: Option<SyncEventFilter>,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<PaginatedSyncEvents, DbErr> {
        let mut condition = Condition::all();
        if let Some(f) = filter {
            if let Some(id) = f.inventory_record_event_id {
                condition =
                    condition.add(sync_event::Column::InventoryRecordEventId.eq(id));
            }
            if let Some(id) = f.connection_sync_state_id {
                condition =
                    condition.add(sync_event::Column::ConnectionSyncStateId.eq(id));
            }
            if let Some(m) = f.sync_event_method {
                condition = condition.add(sync_event::Column::SyncEventMethod.eq(m));
            }
            if let Some(c) = f.sync_event_category {
                condition = condition.add(sync_event::Column::SyncEventCategory.eq(c));
            }
            if let Some(s) = f.status {
                condition = condition.add(sync_event::Column::Status.eq(s));
            }
        }

        let query = sync_event::Entity::find()
            .filter(condition)
            .order_by_desc(sync_event::Column::CreatedAt);

        let total = match txn {
            Some(txn) => query.clone().count(txn).await?,
            None => query.clone().count(&self.db).await?,
        };
        let total_pages = (total as f64 / per_page as f64).ceil() as u64;

        let items = match txn {
            Some(txn) => query
                .paginate(txn, per_page)
                .fetch_page(page.saturating_sub(1))
                .await?,
            None => query
                .paginate(&self.db, per_page)
                .fetch_page(page.saturating_sub(1))
                .await?,
        };

        Ok(PaginatedSyncEvents {
            items,
            total,
            page,
            per_page,
            total_pages,
        })
    }

    pub async fn create(
        &self,
        data: CreateSyncEvent,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<sync_event::Model, DbErr> {
        let active = sync_event::ActiveModel {
            original_record_body: Set(data.original_record_body),
            details: Set(data.details),
            event_direction: Set(data.event_direction),
            inventory_record_event_id: Set(data.inventory_record_event_id),
            sync_event_method: Set(data.sync_event_method),
            sync_event_category: Set(data.sync_event_category),
            attempts: Set(data.attempts.unwrap_or(0)),
            status: Set(data.status.unwrap_or(SyncEventStatus::Pending)),
            last_error: Set(data.last_error),
            last_errored_date: Set(data.last_errored_date.map(Into::into)),
            connection_sync_state_id: Set(data.connection_sync_state_id),
            ..Default::default()
        };
        match txn {
            Some(txn) => active.insert(txn).await,
            None => active.insert(&self.db).await,
        }
    }

    pub async fn update_by_id(
        &self,
        id: i64,
        patch: UpdateSyncEvent,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<sync_event::Model>, SyncEventError> {
        let model = match txn {
            Some(txn) => sync_event::Entity::find_by_id(id).one(txn).await?,
            None => sync_event::Entity::find_by_id(id).one(&self.db).await?,
        };
        let Some(model) = model else {
            return Err(SyncEventError::NotFound);
        };
        let mut active: sync_event::ActiveModel = model.into();
        if patch.original_record_body.is_some() {
            active.original_record_body = Set(patch.original_record_body);
        }
        if patch.details.is_some() {
            active.details = Set(patch.details);
        }
        if let Some(v) = patch.event_direction {
            active.event_direction = Set(v);
        }
        if patch.inventory_record_event_id.is_some() {
            active.inventory_record_event_id = Set(patch.inventory_record_event_id);
        }
        if let Some(v) = patch.sync_event_method {
            active.sync_event_method = Set(v);
        }
        if let Some(v) = patch.sync_event_category {
            active.sync_event_category = Set(v);
        }
        if let Some(v) = patch.attempts {
            active.attempts = Set(v);
        }
        if let Some(v) = patch.status {
            active.status = Set(v);
        }
        if patch.last_error.is_some() {
            active.last_error = Set(patch.last_error);
        }
        if patch.last_errored_date.is_some() {
            active.last_errored_date = Set(patch.last_errored_date.map(Into::into));
        }
        if patch.connection_sync_state_id.is_some() {
            active.connection_sync_state_id = Set(patch.connection_sync_state_id);
        }
        active.updated_at = Set(chrono::Utc::now().into());
        match txn {
            Some(txn) => Ok(Some(active.update(txn).await?)),
            None => Ok(Some(active.update(&self.db).await?)),
        }
    }

    pub async fn update_by_uuid(
        &self,
        uuid: Uuid,
        patch: UpdateSyncEvent,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<sync_event::Model>, SyncEventError> {
        let model = match txn {
            Some(txn) => sync_event::Entity::find()
                .filter(sync_event::Column::Uuid.eq(uuid))
                .one(txn)
                .await?,
            None => sync_event::Entity::find()
                .filter(sync_event::Column::Uuid.eq(uuid))
                .one(&self.db)
                .await?,
        };
        let Some(model) = model else {
            return Err(SyncEventError::NotFound);
        };
        self.update_by_id(model.id, patch, txn).await
    }

    pub async fn delete_by_id(
        &self,
        id: i64,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<sync_event::Model>, SyncEventError> {
        let model = match txn {
            Some(txn) => sync_event::Entity::find_by_id(id).one(txn).await?,
            None => sync_event::Entity::find_by_id(id).one(&self.db).await?,
        };
        let Some(model) = model else {
            return Err(SyncEventError::NotFound);
        };
        let deleted = model.clone();
        let active: sync_event::ActiveModel = model.into();
        match txn {
            Some(txn) => active.delete(txn).await?,
            None => active.delete(&self.db).await?,
        }
        Ok(Some(deleted))
    }

    pub async fn delete_by_uuid(
        &self,
        uuid: Uuid,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<sync_event::Model>, SyncEventError> {
        let model = match txn {
            Some(txn) => sync_event::Entity::find()
                .filter(sync_event::Column::Uuid.eq(uuid))
                .one(txn)
                .await?,
            None => sync_event::Entity::find()
                .filter(sync_event::Column::Uuid.eq(uuid))
                .one(&self.db)
                .await?,
        };
        let Some(model) = model else {
            return Err(SyncEventError::NotFound);
        };
        self.delete_by_id(model.id, txn).await
    }
}

/// END IMPLEMENTATION ///
