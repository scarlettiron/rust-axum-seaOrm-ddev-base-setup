//! CRUD services for inventory_record_event (no routes).

use entity::inventory_record_event;
use entity::sea_orm_active_enums::Currency;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

//DEBUG AND ERRORS ///
#[allow(dead_code)]
#[derive(Debug)]
pub enum InventoryRecordEventError {
    NotFound,
    Db(DbErr),
}

#[allow(dead_code)]
impl From<DbErr> for InventoryRecordEventError {
    fn from(err: DbErr) -> Self {
        InventoryRecordEventError::Db(err)
    }
}

//END DEBUG AND ERRORS


/// BEGUN STRUCTS AND ENUMS ///
pub struct InventoryRecordEventService {
    db: DatabaseConnection,
}

#[allow(dead_code)]
pub struct CreateInventoryRecordEvent {
    pub inventory_record_id: i64,
    pub connection_id: i64,
    pub original_record_body: Option<serde_json::Value>,
    pub price: Option<i32>,
    pub currency: Option<Currency>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub attributes: Option<String>,
    pub qty: Option<i32>,
    pub external_code: Option<String>,
}

#[allow(dead_code)]
pub struct UpdateInventoryRecordEvent {
    pub original_record_body: Option<serde_json::Value>,
    pub price: Option<i32>,
    pub currency: Option<Currency>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub attributes: Option<String>,
    pub qty: Option<i32>,
    pub external_code: Option<String>,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct InventoryRecordEventFilter {
    pub inventory_record_id: Option<i64>,
    pub connection_id: Option<i64>,
}

#[allow(dead_code)]
pub struct PaginatedInventoryRecordEvents {
    pub items: Vec<inventory_record_event::Model>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

/// END STRUCTS AND ENUMS ///


/// BEGUN IMPLEMENTATION ///
#[allow(dead_code)]
impl InventoryRecordEventService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get_by_id(
        &self,
        id: i64,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<inventory_record_event::Model>, DbErr> {
        match txn {
            Some(txn) => inventory_record_event::Entity::find_by_id(id).one(txn).await,
            None => inventory_record_event::Entity::find_by_id(id).one(&self.db).await,
        }
    }

    pub async fn get_by_uuid(
        &self,
        uuid: Uuid,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<inventory_record_event::Model>, DbErr> {
        match txn {
            Some(txn) => inventory_record_event::Entity::find()
                .filter(inventory_record_event::Column::Uuid.eq(uuid))
                .one(txn)
                .await,
            None => inventory_record_event::Entity::find()
                .filter(inventory_record_event::Column::Uuid.eq(uuid))
                .one(&self.db)
                .await,
        }
    }

    pub async fn get_by_inventory_record_id(
        &self,
        inventory_record_id: i64,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Vec<inventory_record_event::Model>, DbErr> {
        let query = inventory_record_event::Entity::find()
            .filter(inventory_record_event::Column::InventoryRecordId.eq(inventory_record_id))
            .order_by_desc(inventory_record_event::Column::CreatedAt);
        match txn {
            Some(txn) => query.all(txn).await,
            None => query.all(&self.db).await,
        }
    }

    pub async fn get_by_connection_id(
        &self,
        connection_id: i64,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Vec<inventory_record_event::Model>, DbErr> {
        let query = inventory_record_event::Entity::find()
            .filter(inventory_record_event::Column::ConnectionId.eq(connection_id))
            .order_by_desc(inventory_record_event::Column::CreatedAt);
        match txn {
            Some(txn) => query.all(txn).await,
            None => query.all(&self.db).await,
        }
    }

    pub async fn get_all(
        &self,
        page: u64,
        per_page: u64,
        filter: Option<InventoryRecordEventFilter>,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<PaginatedInventoryRecordEvents, DbErr> {
        let mut condition = Condition::all();
        if let Some(f) = filter {
            if let Some(inventory_record_id) = f.inventory_record_id {
                condition = condition
                    .add(inventory_record_event::Column::InventoryRecordId.eq(inventory_record_id));
            }
            if let Some(connection_id) = f.connection_id {
                condition =
                    condition.add(inventory_record_event::Column::ConnectionId.eq(connection_id));
            }
        }

        let query = inventory_record_event::Entity::find()
            .filter(condition)
            .order_by_desc(inventory_record_event::Column::CreatedAt);

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

        Ok(PaginatedInventoryRecordEvents {
            items,
            total,
            page,
            per_page,
            total_pages,
        })
    }

    pub async fn create(
        &self,
        data: CreateInventoryRecordEvent,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<inventory_record_event::Model, DbErr> {
        let active = inventory_record_event::ActiveModel {
            inventory_record_id: Set(data.inventory_record_id),
            connection_id: Set(data.connection_id),
            original_record_body: Set(data.original_record_body),
            price: Set(data.price),
            currency: Set(data.currency),
            name: Set(data.name),
            description: Set(data.description),
            attributes: Set(data.attributes),
            qty: Set(data.qty),
            external_code: Set(data.external_code),
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
        patch: UpdateInventoryRecordEvent,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<inventory_record_event::Model>, InventoryRecordEventError> {
        let model = match txn {
            Some(txn) => inventory_record_event::Entity::find_by_id(id).one(txn).await?,
            None => inventory_record_event::Entity::find_by_id(id).one(&self.db).await?,
        };
        let Some(model) = model else {
            return Err(InventoryRecordEventError::NotFound);
        };
        let mut active: inventory_record_event::ActiveModel = model.into();
        if patch.original_record_body.is_some() {
            active.original_record_body = Set(patch.original_record_body);
        }
        if let Some(price) = patch.price {
            active.price = Set(Some(price));
        }
        if let Some(currency) = patch.currency {
            active.currency = Set(Some(currency));
        }
        if patch.name.is_some() {
            active.name = Set(patch.name);
        }
        if patch.description.is_some() {
            active.description = Set(patch.description);
        }
        if patch.attributes.is_some() {
            active.attributes = Set(patch.attributes);
        }
        if patch.qty.is_some() {
            active.qty = Set(patch.qty);
        }
        if patch.external_code.is_some() {
            active.external_code = Set(patch.external_code);
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
        patch: UpdateInventoryRecordEvent,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<inventory_record_event::Model>, InventoryRecordEventError> {
        let model = match txn {
            Some(txn) => inventory_record_event::Entity::find()
                .filter(inventory_record_event::Column::Uuid.eq(uuid))
                .one(txn)
                .await?,
            None => inventory_record_event::Entity::find()
                .filter(inventory_record_event::Column::Uuid.eq(uuid))
                .one(&self.db)
                .await?,
        };
        let Some(model) = model else {
            return Err(InventoryRecordEventError::NotFound);
        };
        self.update_by_id(model.id, patch, txn).await
    }

    pub async fn delete_by_id(
        &self,
        id: i64,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<inventory_record_event::Model>, InventoryRecordEventError> {
        let model = match txn {
            Some(txn) => inventory_record_event::Entity::find_by_id(id).one(txn).await?,
            None => inventory_record_event::Entity::find_by_id(id).one(&self.db).await?,
        };
        let Some(model) = model else {
            return Err(InventoryRecordEventError::NotFound);
        };
        let deleted = model.clone();
        let active: inventory_record_event::ActiveModel = model.into();
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
    ) -> Result<Option<inventory_record_event::Model>, InventoryRecordEventError> {
        let model = match txn {
            Some(txn) => inventory_record_event::Entity::find()
                .filter(inventory_record_event::Column::Uuid.eq(uuid))
                .one(txn)
                .await?,
            None => inventory_record_event::Entity::find()
                .filter(inventory_record_event::Column::Uuid.eq(uuid))
                .one(&self.db)
                .await?,
        };
        let Some(model) = model else {
            return Err(InventoryRecordEventError::NotFound);
        };
        self.delete_by_id(model.id, txn).await
    }
}

// END IMPLEMENTATION
