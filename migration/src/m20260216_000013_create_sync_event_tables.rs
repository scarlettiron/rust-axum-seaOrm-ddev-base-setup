use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::Type;

// ── Enums ──

#[derive(DeriveIden)]
enum SyncEventDirection {
    #[sea_orm(iden = "sync_event_direction")]
    Enum,
    #[sea_orm(iden = "push")]
    Push,
    #[sea_orm(iden = "pull")]
    Pull,
}

#[derive(DeriveIden)]
enum SyncEventMethod {
    #[sea_orm(iden = "sync_event_method")]
    Enum,
    #[sea_orm(iden = "list")]
    List,
    #[sea_orm(iden = "get")]
    Get,
    #[sea_orm(iden = "create")]
    Create,
    #[sea_orm(iden = "update")]
    Update,
    #[sea_orm(iden = "delete")]
    Delete,
}

#[derive(DeriveIden)]
enum SyncEventCategory {
    #[sea_orm(iden = "sync_event_category")]
    Enum,
    #[sea_orm(iden = "inventory")]
    Inventory,
    #[sea_orm(iden = "order")]
    Order,
    #[sea_orm(iden = "customer")]
    Customer,
    #[sea_orm(iden = "other")]
    Other,
}

#[derive(DeriveIden)]
enum SyncEventStatus {
    #[sea_orm(iden = "sync_event_status")]
    Enum,
    #[sea_orm(iden = "pending")]
    Pending,
    #[sea_orm(iden = "in_progress")]
    InProgress,
    #[sea_orm(iden = "success")]
    Success,
    #[sea_orm(iden = "error")]
    Error,
}

// ── Table ──

#[derive(DeriveIden)]
enum SyncEvent {
    Table,
    Id,
    Uuid,
    CreatedAt,
    UpdatedAt,
    OriginalRecordBody,
    Details,
    EventDirection,
    InventoryRecordEventId,
    SyncEventMethod,
    SyncEventCategory,
    Attempts,
    Status,
    LastError,
    LastErroredDate,
    ConnectionSyncStateId,
}

#[derive(DeriveIden)]
enum InventoryRecordEvent {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum ErpConnectionSyncState {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum SyncEventIndexes {
    SyncEventUuidIdx,
    SyncEventInventoryRecordEventIdIdx,
    SyncEventConnectionSyncStateIdIdx,
    SyncEventStatusIdx,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── Create enums ──

        manager
            .create_type(
                Type::create()
                    .as_enum(SyncEventDirection::Enum)
                    .values(vec![SyncEventDirection::Push, SyncEventDirection::Pull])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(SyncEventMethod::Enum)
                    .values(vec![
                        SyncEventMethod::List,
                        SyncEventMethod::Get,
                        SyncEventMethod::Create,
                        SyncEventMethod::Update,
                        SyncEventMethod::Delete,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(SyncEventCategory::Enum)
                    .values(vec![
                        SyncEventCategory::Inventory,
                        SyncEventCategory::Order,
                        SyncEventCategory::Customer,
                        SyncEventCategory::Other,
                    ])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(SyncEventStatus::Enum)
                    .values(vec![
                        SyncEventStatus::Pending,
                        SyncEventStatus::InProgress,
                        SyncEventStatus::Success,
                        SyncEventStatus::Error,
                    ])
                    .to_owned(),
            )
            .await?;

        // ── Create table ──

        manager
            .create_table(
                Table::create()
                    .table(SyncEvent::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SyncEvent::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(SyncEvent::Uuid)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(SyncEvent::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SyncEvent::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SyncEvent::OriginalRecordBody)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(SyncEvent::Details)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(SyncEvent::EventDirection)
                            .enumeration(
                                SyncEventDirection::Enum,
                                [SyncEventDirection::Push, SyncEventDirection::Pull],
                            )
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SyncEvent::InventoryRecordEventId)
                            .big_integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(SyncEvent::SyncEventMethod)
                            .enumeration(
                                SyncEventMethod::Enum,
                                [
                                    SyncEventMethod::List,
                                    SyncEventMethod::Get,
                                    SyncEventMethod::Create,
                                    SyncEventMethod::Update,
                                    SyncEventMethod::Delete,
                                ],
                            )
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SyncEvent::SyncEventCategory)
                            .enumeration(
                                SyncEventCategory::Enum,
                                [
                                    SyncEventCategory::Inventory,
                                    SyncEventCategory::Order,
                                    SyncEventCategory::Customer,
                                    SyncEventCategory::Other,
                                ],
                            )
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SyncEvent::Attempts)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SyncEvent::Status)
                            .enumeration(
                                SyncEventStatus::Enum,
                                [
                                    SyncEventStatus::Pending,
                                    SyncEventStatus::InProgress,
                                    SyncEventStatus::Success,
                                    SyncEventStatus::Error,
                                ],
                            )
                            .not_null()
                            .default(Expr::cust("'pending'::sync_event_status")),
                    )
                    .col(
                        ColumnDef::new(SyncEvent::LastError)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(SyncEvent::LastErroredDate)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(SyncEvent::ConnectionSyncStateId)
                            .big_integer()
                            .null(),
                    )
                    // Optional FK: inventory_record_event_id may be null
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                SyncEvent::Table,
                                SyncEvent::InventoryRecordEventId,
                            )
                            .to(InventoryRecordEvent::Table, InventoryRecordEvent::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    // Optional FK: connection_sync_state_id may be null
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                SyncEvent::Table,
                                SyncEvent::ConnectionSyncStateId,
                            )
                            .to(ErpConnectionSyncState::Table, ErpConnectionSyncState::Id)
                            .on_delete(ForeignKeyAction::SetNull)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(SyncEventIndexes::SyncEventUuidIdx.to_string())
                    .table(SyncEvent::Table)
                    .col(SyncEvent::Uuid)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(SyncEventIndexes::SyncEventInventoryRecordEventIdIdx.to_string())
                    .table(SyncEvent::Table)
                    .col(SyncEvent::InventoryRecordEventId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(SyncEventIndexes::SyncEventConnectionSyncStateIdIdx.to_string())
                    .table(SyncEvent::Table)
                    .col(SyncEvent::ConnectionSyncStateId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(SyncEventIndexes::SyncEventStatusIdx.to_string())
                    .table(SyncEvent::Table)
                    .col(SyncEvent::Status)
                    .to_owned(),
            )
            .await?;

        let table_name = SyncEvent::Table.to_string();
        manager
            .get_connection()
            .execute_unprepared(&format!(
                r#"
                ALTER TABLE {}
                ALTER COLUMN uuid
                SET DEFAULT gen_random_uuid();
                "#,
                table_name
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(SyncEvent::Table).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(SyncEventStatus::Enum).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(SyncEventCategory::Enum).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(SyncEventMethod::Enum).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(SyncEventDirection::Enum).to_owned())
            .await?;
        Ok(())
    }
}
