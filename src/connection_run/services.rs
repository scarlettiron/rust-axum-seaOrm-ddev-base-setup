use entity::connection_run;
use entity::sea_orm_active_enums::{ConnectionRunStatus, ConnectionRunType};
use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr, EntityTrait,
    QueryFilter, QueryOrder, Set,
};
use uuid::Uuid;

#[allow(dead_code)]
#[derive(Debug)]
pub enum ConnectionRunError {
    NotFound,
    Db(DbErr),
}

#[allow(dead_code)]
impl From<DbErr> for ConnectionRunError {
    fn from(err: DbErr) -> Self {
        ConnectionRunError::Db(err)
    }
}

#[allow(dead_code)]
pub struct ConnectionRunService {
    db: DatabaseConnection,
}

#[allow(dead_code)]
pub struct CreateConnectionRun {
    pub connection_id: i64,
    pub status: Option<ConnectionRunStatus>,
    pub run_type: Option<ConnectionRunType>,
    pub error_message: Option<String>,
}

#[allow(dead_code)]
pub struct UpdateConnectionRun {
    pub status: Option<ConnectionRunStatus>,
    pub error_message: Option<String>,
}

#[allow(dead_code)]
impl ConnectionRunService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get_by_id(
        &self,
        id: i64,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<connection_run::Model>, DbErr> {
        match txn {
            Some(txn) => connection_run::Entity::find_by_id(id).one(txn).await,
            None => connection_run::Entity::find_by_id(id).one(&self.db).await,
        }
    }

    pub async fn get_by_uuid(
        &self,
        uuid: Uuid,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<connection_run::Model>, DbErr> {
        match txn {
            Some(txn) => {
                connection_run::Entity::find()
                    .filter(connection_run::Column::Uuid.eq(uuid))
                    .one(txn)
                    .await
            }
            None => {
                connection_run::Entity::find()
                    .filter(connection_run::Column::Uuid.eq(uuid))
                    .one(&self.db)
                    .await
            }
        }
    }

    pub async fn list_by_connection_id(
        &self,
        connection_id: i64,
        limit: u64,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Vec<connection_run::Model>, DbErr> {
        let query = connection_run::Entity::find()
            .filter(connection_run::Column::ConnectionId.eq(connection_id))
            .order_by_desc(connection_run::Column::CreatedAt)
            .limit(limit);

        match txn {
            Some(txn) => query.all(txn).await,
            None => query.all(&self.db).await,
        }
    }

    pub async fn create(
        &self,
        data: CreateConnectionRun,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<connection_run::Model, DbErr> {
        let active = connection_run::ActiveModel {
            connection_id: Set(data.connection_id),
            status: Set(data.status.unwrap_or(ConnectionRunStatus::Success)),
            run_type: Set(data.run_type.unwrap_or(ConnectionRunType::Poll)),
            error_message: Set(data.error_message),
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
        patch: UpdateConnectionRun,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<connection_run::Model>, ConnectionRunError> {
        let model = match txn {
            Some(txn) => {
                connection_run::Entity::find()
                    .filter(connection_run::Column::Uuid.eq(uuid))
                    .one(txn)
                    .await?
            }
            None => {
                connection_run::Entity::find()
                    .filter(connection_run::Column::Uuid.eq(uuid))
                    .one(&self.db)
                    .await?
            }
        };

        let Some(model) = model else {
            return Err(ConnectionRunError::NotFound);
        };

        let mut active: connection_run::ActiveModel = model.into();
        if let Some(v) = patch.status {
            active.status = Set(v);
        }
        if patch.error_message.is_some() {
            active.error_message = Set(patch.error_message);
        }
        active.updated_at = Set(chrono::Utc::now().into());

        match txn {
            Some(txn) => Ok(Some(active.update(txn).await?)),
            None => Ok(Some(active.update(&self.db).await?)),
        }
    }
}
