use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, PaginatorTrait, QueryFilter, QueryOrder, Set,
};
use entity::tenant;
use entity::sea_orm_active_enums::Enum as TenantStatus;
use uuid::Uuid;


//DEBUG AND ERRORS ///
#[allow(dead_code)]
#[derive(Debug)]
pub enum TenantError {
    NotFound,
    Db(DbErr),
}

#[allow(dead_code)]
impl From<DbErr> for TenantError {
    fn from(err: DbErr) -> Self {
        TenantError::Db(err)
    }
}

//END DEBUG AND ERRORS


/// BEGUN STRUCTS AND ENUMS ///
pub struct TenantService {
    db: DatabaseConnection,
}

#[allow(dead_code)]
pub struct CreateTenant {
    pub display_name: Option<String>,
}

#[allow(dead_code)]
pub struct UpdateTenant {
    pub display_name: Option<String>,
    pub status: Option<TenantStatus>,
}

#[allow(dead_code)]
#[derive(Default)]
pub struct TenantFilter {
    pub status: Option<TenantStatus>,
    pub display_name: Option<String>,
    pub tenant_id: Option<String>,
}

#[allow(dead_code)]
pub struct PaginatedTenants {
    pub items: Vec<tenant::Model>,
    pub total: u64,
    pub page: u64,
    pub per_page: u64,
    pub total_pages: u64,
}

/// END STRUCTS AND ENUMS ///


/// BEGUN IMPLEMENTATION ///
#[allow(dead_code)]
impl TenantService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    ///generates a tenant ID in format TN_<uuid without dashes>
    fn generate_tenant_id() -> String {
        let uuid = Uuid::new_v4();
        let uuid_no_dashes = uuid.to_string().replace("-", "");
        format!("TN_{}", uuid_no_dashes)
    }

    pub async fn get_by_tenant_id(
        &self,
        tenant_id: &str,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<tenant::Model>, DbErr> {
        match txn {
            Some(txn) => {
                tenant::Entity::find()
                    .filter(tenant::Column::TenantId.eq(tenant_id))
                    .one(txn)
                    .await
            }
            None => {
                tenant::Entity::find()
                    .filter(tenant::Column::TenantId.eq(tenant_id))
                    .one(&self.db)
                    .await
            }
        }
    }

    pub async fn get_by_uuid(
        &self,
        uuid: Uuid,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<tenant::Model>, DbErr> {
        match txn {
            Some(txn) => {
                tenant::Entity::find()
                    .filter(tenant::Column::Uuid.eq(uuid))
                    .one(txn)
                    .await
            }
            None => {
                tenant::Entity::find()
                    .filter(tenant::Column::Uuid.eq(uuid))
                    .one(&self.db)
                    .await
            }
        }
    }

    pub async fn get_by_id(
        &self,
        id: i64,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<tenant::Model>, DbErr> {
        match txn {
            Some(txn) => {
                tenant::Entity::find_by_id(id)
                    .one(txn)
                    .await
            }
            None => {
                tenant::Entity::find_by_id(id)
                    .one(&self.db)
                    .await
            }
        }
    }

    pub async fn get_all(
        &self,
        page: u64,
        per_page: u64,
        filter: Option<TenantFilter>,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<PaginatedTenants, DbErr> {
        let mut condition = Condition::all();

        if let Some(f) = filter {
            if let Some(status) = f.status {
                condition = condition.add(tenant::Column::Status.eq(status));
            }
            if let Some(display_name) = f.display_name {
                condition = condition.add(tenant::Column::DisplayName.contains(&display_name));
            }
            if let Some(tenant_id) = f.tenant_id {
                condition = condition.add(tenant::Column::TenantId.contains(&tenant_id));
            }
        }

        let query = tenant::Entity::find()
            .filter(condition)
            .order_by_desc(tenant::Column::CreatedAt);

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

        Ok(PaginatedTenants {
            items,
            total,
            page,
            per_page,
            total_pages,
        })
    }

    pub async fn create(
        &self,
        data: CreateTenant,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<tenant::Model, DbErr> {
        let tenant_id = Self::generate_tenant_id();

        let active = tenant::ActiveModel {
            tenant_id: Set(tenant_id),
            display_name: Set(data.display_name),
            status: Set(TenantStatus::Active),
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
        patch: UpdateTenant,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<tenant::Model>, TenantError> {
        let model = match txn {
            Some(txn) => {
                tenant::Entity::find()
                    .filter(tenant::Column::Uuid.eq(uuid))
                    .one(txn)
                    .await?
            }
            None => {
                tenant::Entity::find()
                    .filter(tenant::Column::Uuid.eq(uuid))
                    .one(&self.db)
                    .await?
            }
        };

        let Some(model) = model else {
            return Err(TenantError::NotFound);
        };

        let mut new_data: tenant::ActiveModel = model.into();

        if let Some(display_name) = patch.display_name {
            new_data.display_name = Set(Some(display_name));
        }

        if let Some(status) = patch.status {
            new_data.status = Set(status);
        }

        new_data.updated_at = Set(chrono::Utc::now().into());

        match txn {
            Some(txn) => Ok(Some(new_data.update(txn).await?)),
            None => Ok(Some(new_data.update(&self.db).await?)),
        }
    }

    pub async fn update_by_tenant_id(
        &self,
        tenant_id: &str,
        patch: UpdateTenant,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<tenant::Model>, TenantError> {
        let model = match txn {
            Some(txn) => {
                tenant::Entity::find()
                    .filter(tenant::Column::TenantId.eq(tenant_id))
                    .one(txn)
                    .await?
            }
            None => {
                tenant::Entity::find()
                    .filter(tenant::Column::TenantId.eq(tenant_id))
                    .one(&self.db)
                    .await?
            }
        };

        let Some(model) = model else {
            return Err(TenantError::NotFound);
        };

        let mut new_data: tenant::ActiveModel = model.into();

        if let Some(display_name) = patch.display_name {
            new_data.display_name = Set(Some(display_name));
        }

        if let Some(status) = patch.status {
            new_data.status = Set(status);
        }

        new_data.updated_at = Set(chrono::Utc::now().into());

        match txn {
            Some(txn) => Ok(Some(new_data.update(txn).await?)),
            None => Ok(Some(new_data.update(&self.db).await?)),
        }
    }

    ///soft delete - sets status to removed instead of deleting
    pub async fn delete_by_uuid(
        &self,
        uuid: Uuid,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<tenant::Model>, TenantError> {
        self.update_by_uuid(
            uuid,
            UpdateTenant {
                display_name: None,
                status: Some(TenantStatus::Removed),
            },
            txn,
        )
        .await
    }

    ///soft delete - sets status to removed instead of deleting
    pub async fn delete_by_tenant_id(
        &self,
        tenant_id: &str,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<tenant::Model>, TenantError> {
        self.update_by_tenant_id(
            tenant_id,
            UpdateTenant {
                display_name: None,
                status: Some(TenantStatus::Removed),
            },
            txn,
        )
        .await
    }

    pub async fn is_tenant_active(
        &self,
        tenant_id: &str,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<bool, DbErr> {
        let model = match self.get_by_tenant_id(tenant_id, txn).await? {
            Some(m) => m,
            None => return Ok(false),
        };

        Ok(model.status == TenantStatus::Active)
    }
}
