use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum ErpConnectionSyncState {
    Table,
    Id,
    Uuid,
    ConnectionId,
    SyncCursor,
    SyncLockOwner,
    SyncLockUntil,
    RateLimitRemaining,
    RateLimit,
    RateLimitResetAt,
    RateLimitBackoffUntil,
    RateLimitWindowSeconds,
    UpdatedAt,
    CreatedAt,
}

// References connection_identity table from m20260129_000007_create_connection_identity_table
#[derive(DeriveIden)]
enum ConnectionIdentity {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum ErpConnectionSyncStateIndexes {
    ErpConnectionSyncStateUuidIdx,
    ErpConnectionSyncStateConnectionIdIdx,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ErpConnectionSyncState::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ErpConnectionSyncState::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionSyncState::Uuid)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionSyncState::ConnectionId)
                            .big_integer()
                            .not_null(), // FK -> connection_identity.id
                    )
                    .col(
                        ColumnDef::new(ErpConnectionSyncState::SyncCursor)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionSyncState::SyncLockOwner)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionSyncState::SyncLockUntil)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionSyncState::RateLimitRemaining)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionSyncState::RateLimit)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionSyncState::RateLimitResetAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionSyncState::RateLimitBackoffUntil)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionSyncState::RateLimitWindowSeconds)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionSyncState::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionSyncState::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                ErpConnectionSyncState::Table,
                                ErpConnectionSyncState::ConnectionId,
                            )
                            .to(ConnectionIdentity::Table, ConnectionIdentity::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        // Unique index on uuid
        manager
            .create_index(
                Index::create()
                    .name(ErpConnectionSyncStateIndexes::ErpConnectionSyncStateUuidIdx.to_string())
                    .table(ErpConnectionSyncState::Table)
                    .col(ErpConnectionSyncState::Uuid)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Index on connection_id (one row per connection expected; unique makes sense)
        manager
            .create_index(
                Index::create()
                    .name(
                        ErpConnectionSyncStateIndexes::ErpConnectionSyncStateConnectionIdIdx
                            .to_string(),
                    )
                    .table(ErpConnectionSyncState::Table)
                    .col(ErpConnectionSyncState::ConnectionId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Default uuid to gen_random_uuid()
        let table_name = ErpConnectionSyncState::Table.to_string();
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
            .drop_table(
                Table::drop()
                    .table(ErpConnectionSyncState::Table)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
