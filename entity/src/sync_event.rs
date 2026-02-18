//! `SeaORM` Entity for sync_event

use super::sea_orm_active_enums::{
    SyncEventCategory, SyncEventDirection, SyncEventMethod, SyncEventStatus,
};
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, Eq, DeriveEntityModel)]
#[sea_orm(table_name = "sync_event")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i64,
    #[sea_orm(unique)]
    pub uuid: Uuid,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: DateTimeWithTimeZone,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub original_record_body: Option<Json>,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub details: Option<Json>,
    pub event_direction: SyncEventDirection,
    pub inventory_record_event_id: Option<i64>,
    pub sync_event_method: SyncEventMethod,
    pub sync_event_category: SyncEventCategory,
    pub attempts: i32,
    pub status: SyncEventStatus,
    #[sea_orm(column_type = "JsonBinary", nullable)]
    pub last_error: Option<Json>,
    pub last_errored_date: Option<DateTimeWithTimeZone>,
    pub connection_sync_state_id: Option<i64>,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::inventory_record_event::Entity",
        from = "Column::InventoryRecordEventId",
        to = "super::inventory_record_event::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    InventoryRecordEvent,
    #[sea_orm(
        belongs_to = "super::erp_connection_sync_state::Entity",
        from = "Column::ConnectionSyncStateId",
        to = "super::erp_connection_sync_state::Column::Id",
        on_update = "Cascade",
        on_delete = "SetNull"
    )]
    ErpConnectionSyncState,
}

impl Related<super::inventory_record_event::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::InventoryRecordEvent.def()
    }
}

impl Related<super::erp_connection_sync_state::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ErpConnectionSyncState.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
