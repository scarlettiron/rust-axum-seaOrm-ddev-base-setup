//! CRUD services for inventory_record (no routes).

use entity::inventory_record;
use entity::sea_orm_active_enums::SystemIdKey;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

//DEBUG AND ERRORS ///
#[allow(dead_code)]
#[derive(Debug)]
pub enum InventoryRecordError {
    NotFound,
    Db(DbErr),
}

#[allow(dead_code)]
impl From<DbErr> for InventoryRecordError {
    fn from(err: DbErr) -> Self {
        InventoryRecordError::Db(err)
    }
}

//END DEBUG AND ERRORS


/// BEGUN STRUCTS AND ENUMS ///
pub struct InventoryRecordService {
    db: DatabaseConnection,
}

#[allow(dead_code)]
pub struct CreateInventoryRecord {
    pub tenant_id: i64,
    pub originating_connection_id: i64,
    pub original_record_body: Option<serde_json::Value>,
    pub system_id_key: SystemIdKey,
    pub system_id: String,
}

#[allow(dead_code)]
pub struct UpdateInventoryRecord {
    pub original_record_body: Option<serde_json::Value>,
    pub system_id_key: Option<SystemIdKey>,
    pub system_id: Option<String>,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct InventoryRecordFilter {
    pub tenant_id: Option<i64>,
    pub originating_connection_id: Option<i64>,
    pub system_id_key: Option<SystemIdKey>,
}

#[allow(dead_code)]
pub struct PaginatedInventoryRecords {
    pub items: Vec<inventory_record::Model>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

/// END STRUCTS AND ENUMS ///


/// BEGUN IMPLEMENTATION ///
#[allow(dead_code)]
impl InventoryRecordService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Format for downstream consumers: join system_id_key and system_id (e.g. "QBD:abc123").
    #[allow(dead_code)]
    pub fn format_downstream_consumer_id(system_id_key: SystemIdKey, system_id: &str) -> String {
        let prefix = match system_id_key {
            SystemIdKey::Qbd => "QBD",
            SystemIdKey::Qbo => "QBO",
            SystemIdKey::Sapo => "SAPO",
        };
        format!("{}:{}", prefix, system_id)
    }

    pub async fn get_by_id(
        &self,
        id: i64,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<inventory_record::Model>, DbErr> {
        match txn {
            Some(txn) => inventory_record::Entity::find_by_id(id).one(txn).await,
            None => inventory_record::Entity::find_by_id(id).one(&self.db).await,
        }
    }

    pub async fn get_by_uuid(
        &self,
        uuid: Uuid,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<inventory_record::Model>, DbErr> {
        match txn {
            Some(txn) => inventory_record::Entity::find()
                .filter(inventory_record::Column::Uuid.eq(uuid))
                .one(txn)
                .await,
            None => inventory_record::Entity::find()
                .filter(inventory_record::Column::Uuid.eq(uuid))
                .one(&self.db)
                .await,
        }
    }

    pub async fn get_by_tenant_id(
        &self,
        tenant_id: i64,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Vec<inventory_record::Model>, DbErr> {
        let query = inventory_record::Entity::find()
            .filter(inventory_record::Column::TenantId.eq(tenant_id))
            .order_by_desc(inventory_record::Column::CreatedAt);
        match txn {
            Some(txn) => query.all(txn).await,
            None => query.all(&self.db).await,
        }
    }

    pub async fn get_all(
        &self,
        page: u64,
        per_page: u64,
        filter: Option<InventoryRecordFilter>,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<PaginatedInventoryRecords, DbErr> {
        let mut condition = Condition::all();
        if let Some(f) = filter {
            if let Some(tenant_id) = f.tenant_id {
                condition = condition.add(inventory_record::Column::TenantId.eq(tenant_id));
            }
            if let Some(conn_id) = f.originating_connection_id {
                condition = condition
                    .add(inventory_record::Column::OriginatingConnectionId.eq(conn_id));
            }
            if let Some(system_id_key) = f.system_id_key {
                condition =
                    condition.add(inventory_record::Column::SystemIdKey.eq(system_id_key));
            }
        }

        let query = inventory_record::Entity::find()
            .filter(condition)
            .order_by_desc(inventory_record::Column::CreatedAt);

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

        Ok(PaginatedInventoryRecords {
            items,
            total,
            page,
            per_page,
            total_pages,
        })
    }

    pub async fn create(
        &self,
        data: CreateInventoryRecord,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<inventory_record::Model, DbErr> {
        let active = inventory_record::ActiveModel {
            tenant_id: Set(data.tenant_id),
            originating_connection_id: Set(data.originating_connection_id),
            original_record_body: Set(data.original_record_body),
            system_id_key: Set(data.system_id_key),
            system_id: Set(data.system_id),
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
        patch: UpdateInventoryRecord,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<inventory_record::Model>, InventoryRecordError> {
        let model = match txn {
            Some(txn) => inventory_record::Entity::find_by_id(id).one(txn).await?,
            None => inventory_record::Entity::find_by_id(id).one(&self.db).await?,
        };
        let Some(model) = model else {
            return Err(InventoryRecordError::NotFound);
        };
        let mut active: inventory_record::ActiveModel = model.into();
        if patch.original_record_body.is_some() {
            active.original_record_body = Set(patch.original_record_body);
        }
        if let Some(system_id_key) = patch.system_id_key {
            active.system_id_key = Set(system_id_key);
        }
        if let Some(system_id) = patch.system_id {
            active.system_id = Set(system_id);
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
        patch: UpdateInventoryRecord,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<inventory_record::Model>, InventoryRecordError> {
        let model = match txn {
            Some(txn) => inventory_record::Entity::find()
                .filter(inventory_record::Column::Uuid.eq(uuid))
                .one(txn)
                .await?,
            None => inventory_record::Entity::find()
                .filter(inventory_record::Column::Uuid.eq(uuid))
                .one(&self.db)
                .await?,
        };
        let Some(model) = model else {
            return Err(InventoryRecordError::NotFound);
        };
        self.update_by_id(model.id, patch, txn).await
    }

    pub async fn delete_by_id(
        &self,
        id: i64,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<inventory_record::Model>, InventoryRecordError> {
        let model = match txn {
            Some(txn) => inventory_record::Entity::find_by_id(id).one(txn).await?,
            None => inventory_record::Entity::find_by_id(id).one(&self.db).await?,
        };
        let Some(model) = model else {
            return Err(InventoryRecordError::NotFound);
        };
        let deleted = model.clone();
        let active: inventory_record::ActiveModel = model.into();
        match txn {
            Some(txn) => active.delete(txn).await?,
            None => active.delete(&self.db).await?,
        };
        Ok(Some(deleted))
    }

    pub async fn delete_by_uuid(
        &self,
        uuid: Uuid,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<inventory_record::Model>, InventoryRecordError> {
        let model = match txn {
            Some(txn) => inventory_record::Entity::find()
                .filter(inventory_record::Column::Uuid.eq(uuid))
                .one(txn)
                .await?,
            None => inventory_record::Entity::find()
                .filter(inventory_record::Column::Uuid.eq(uuid))
                .one(&self.db)
                .await?,
        };
        let Some(model) = model else {
            return Err(InventoryRecordError::NotFound);
        };
        self.delete_by_id(model.id, txn).await
    }
}

// END IMPLEMENTATION
