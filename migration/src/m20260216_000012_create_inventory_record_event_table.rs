use sea_orm_migration::prelude::*;

// ── Table ──

#[derive(DeriveIden)]
enum InventoryRecordEvent {
    Table,
    Id,
    Uuid,
    CreatedAt,
    UpdatedAt,
    InventoryRecordId,
    ConnectionId,
    OriginalRecordBody,
    Price,
    Currency,
    Name,
    Description,
    Attributes,
    Qty,
    ExternalCode,
}

#[derive(DeriveIden)]
enum InventoryRecord {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum ConnectionIdentity {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum InventoryRecordEventIndexes {
    InventoryRecordEventUuidIdx,
    InventoryRecordEventInventoryRecordIdIdx,
    InventoryRecordEventConnectionIdIdx,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(InventoryRecordEvent::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InventoryRecordEvent::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(InventoryRecordEvent::Uuid)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(InventoryRecordEvent::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(InventoryRecordEvent::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(InventoryRecordEvent::InventoryRecordId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InventoryRecordEvent::ConnectionId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InventoryRecordEvent::OriginalRecordBody)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(InventoryRecordEvent::Price)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(InventoryRecordEvent::Currency)
                            .custom("currency")
                            .null(),
                    )
                    .col(
                        ColumnDef::new(InventoryRecordEvent::Name)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(InventoryRecordEvent::Description)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(InventoryRecordEvent::Attributes)
                            .array(ColumnType::Text)
                            .null(),
                    )
                    .col(
                        ColumnDef::new(InventoryRecordEvent::Qty)
                            .integer()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(InventoryRecordEvent::ExternalCode)
                            .text()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                InventoryRecordEvent::Table,
                                InventoryRecordEvent::InventoryRecordId,
                            )
                            .to(InventoryRecord::Table, InventoryRecord::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                InventoryRecordEvent::Table,
                                InventoryRecordEvent::ConnectionId,
                            )
                            .to(ConnectionIdentity::Table, ConnectionIdentity::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(InventoryRecordEventIndexes::InventoryRecordEventUuidIdx.to_string())
                    .table(InventoryRecordEvent::Table)
                    .col(InventoryRecordEvent::Uuid)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(
                        InventoryRecordEventIndexes::InventoryRecordEventInventoryRecordIdIdx
                            .to_string(),
                    )
                    .table(InventoryRecordEvent::Table)
                    .col(InventoryRecordEvent::InventoryRecordId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(
                        InventoryRecordEventIndexes::InventoryRecordEventConnectionIdIdx
                            .to_string(),
                    )
                    .table(InventoryRecordEvent::Table)
                    .col(InventoryRecordEvent::ConnectionId)
                    .to_owned(),
            )
            .await?;

        let table_name = InventoryRecordEvent::Table.to_string();
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
                    .table(InventoryRecordEvent::Table)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
