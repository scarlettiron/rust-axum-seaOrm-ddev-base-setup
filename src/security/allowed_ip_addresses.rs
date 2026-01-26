use sea_orm::{
    ActiveModelTrait, ColumnTrait, DatabaseConnection, DatabaseTransaction, DbErr,
    EntityTrait, QueryFilter, Set,
};
use entity::allowed_ip_address;
use entity::sea_orm_active_enums::AllowedIpAddressStatusEnum as AllowedIpAddressStatus;
use uuid::Uuid;


//DEBUG AND ERRORS ///
#[allow(dead_code)]
#[derive(Debug)]
pub enum AllowedIpAddressError {
    NotFound,
    Db(DbErr),
}

#[allow(dead_code)]
impl From<DbErr> for AllowedIpAddressError {
    fn from(err: DbErr) -> Self {
        AllowedIpAddressError::Db(err)
    }
}

#[allow(dead_code)]
impl std::fmt::Display for AllowedIpAddressError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AllowedIpAddressError::NotFound => write!(f, "IP address not found"),
            AllowedIpAddressError::Db(e) => write!(f, "Database error: {}", e),
        }
    }
}

//END DEBUG AND ERRORS ///


/// BEGUN STRUCTS AND ENUMS ///
pub struct AllowedIpAddressService {
    db: DatabaseConnection,
}

#[allow(dead_code)]
pub struct UpdateAllowedIpAddress {
    pub ip_address: Option<String>,
    pub status: Option<AllowedIpAddressStatus>,
}


/// END STRUCTS AND ENUMS ///

impl AllowedIpAddressService {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn get_by_ip_address(
        &self,
        ip_address: &str,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<Option<allowed_ip_address::Model>, AllowedIpAddressError> {
        match txn {
            Some(txn) => {
                Ok(allowed_ip_address::Entity::find()
                    .filter(allowed_ip_address::Column::IpAddress.eq(ip_address))
                    .one(txn)
                    .await?)
            }
            None => {
                Ok(allowed_ip_address::Entity::find()
                    .filter(allowed_ip_address::Column::IpAddress.eq(ip_address))
                    .one(&self.db)
                    .await?)
            }
        }
    }

    pub async fn ip_address_allowed(
        &self,
        ip_address: &str,
        txn: Option<&DatabaseTransaction>,
    ) -> Result<bool, AllowedIpAddressError> {
        let model = self.get_by_ip_address(ip_address, txn).await?;
        if let Some(m) = model {
            if m.status == AllowedIpAddressStatus::Active {
                return Ok(true);
            }
        }
        Ok(false)
    }
}