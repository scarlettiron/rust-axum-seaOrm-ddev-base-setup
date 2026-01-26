use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, QueryFilter, Set,
};
use entity::api_token;
use entity::sea_orm_active_enums::ApiTokenStatusEnum as ApiTokenStatus;
use uuid::Uuid;


//DEBUG AND ERRORS ///
#[allow(dead_code)]
#[derive(Debug)]
pub enum ApiTokenError {
    NotFound,
    Db(DbErr),
}

#[allow(dead_code)]
impl From<DbErr> for ApiTokenError {
    fn from(err: DbErr) -> Self {
        ApiTokenError::Db(err)
    }
}

//END DEBUG AND ERRORS




/// BEGUN STRUCTS AND ENUMS ///
pub struct ApiTokenService {
    db: DatabaseConnection,
}

#[allow(dead_code)]
pub struct UpdateApiToken {
    pub token: Option<String>,
    pub status: Option<ApiTokenStatus>,
}

/// END STRUCTS AND ENUMS ///







/// BEGUN IMPLEMENTATION ///
#[allow(dead_code)]
impl ApiTokenService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }


    pub async fn get_by_token(
        &self,
        token: &str,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<api_token::Model>, DbErr> {
        match txn {
            Some(txn) => {
                api_token::Entity::find()
                    .filter(api_token::Column::Token.eq(token))
                    .one(txn)
                    .await
            }
            None => {
                api_token::Entity::find()
                    .filter(api_token::Column::Token.eq(token))
                    .one(&self.db)
                    .await
            }
        }
    }

    pub async fn get_by_uuid(
        &self,
        uuid: Uuid,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<api_token::Model>, DbErr> {
        match txn {
            Some(txn) => {
                api_token::Entity::find()
                    .filter(api_token::Column::Uuid.eq(uuid))
                    .one(txn)
                    .await
            }
            None => {
                api_token::Entity::find()
                    .filter(api_token::Column::Uuid.eq(uuid))
                    .one(&self.db)
                    .await
            }
        }
    }

    pub async fn create(
        &self,
        token: String,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<api_token::Model, DbErr> {
        let active = api_token::ActiveModel {
            token: Set(token),
            status: Set(ApiTokenStatus::Active),
            ..Default::default()
        };

        match txn {
            Some(txn) => active.insert(txn).await,
            None => active.insert(&self.db).await,
        }
    }


    pub async fn update_token_by_uuid(
        &self,
        uuid: Uuid,
        patch: UpdateApiToken,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<api_token::Model>, ApiTokenError> {
        let model = match txn {
            Some(txn) => {
                api_token::Entity::find()
                    .filter(api_token::Column::Uuid.eq(uuid))
                    .one(txn)
                    .await?
            }
            None => {
                api_token::Entity::find()
                    .filter(api_token::Column::Uuid.eq(uuid))
                    .one(&self.db)
                    .await?
            }
        };

        let Some(model) = model else {
            return Err(ApiTokenError::NotFound);
        };

        let mut new_data: api_token::ActiveModel = model.into();

        if let Some(token) = patch.token {
            new_data.token = Set(token);
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


    pub async fn is_token_valid(
        &self,
        token: &str,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<bool, DbErr> {
        let model = match self.get_by_token(token, txn).await? {
            Some(m) => m,
            None => return Ok(false),
        };

        if model.status == ApiTokenStatus::Active {
            return Ok(true);
        }

        Ok(false)
    }

}

