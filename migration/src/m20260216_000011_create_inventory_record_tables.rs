use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::Type;

// ── Enums ──

#[derive(DeriveIden)]
enum Currency {
    #[sea_orm(iden = "currency")]
    Enum,
    #[sea_orm(iden = "usd")]
    Usd,
}

#[derive(DeriveIden)]
enum SystemIdKey {
    #[sea_orm(iden = "system_id_key")]
    Enum,
    #[sea_orm(iden = "qbd")]
    Qbd,
    #[sea_orm(iden = "qbo")]
    Qbo,
    #[sea_orm(iden = "sapo")]
    Sapo,
}

// ── Table ──

#[derive(DeriveIden)]
enum InventoryRecord {
    Table,
    Id,
    Uuid,
    CreatedAt,
    UpdatedAt,
    TenantId,
    OriginatingConnectionId,
    OriginalRecordBody,
    SystemIdKey,
    SystemId,
}

#[derive(DeriveIden)]
enum Tenant {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum ConnectionIdentity {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum InventoryRecordIndexes {
    InventoryRecordUuidIdx,
    InventoryRecordTenantIdIdx,
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
                    .as_enum(Currency::Enum)
                    .values(vec![Currency::Usd])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(SystemIdKey::Enum)
                    .values(vec![SystemIdKey::Qbd, SystemIdKey::Qbo, SystemIdKey::Sapo])
                    .to_owned(),
            )
            .await?;

        // ── Create table ──

        manager
            .create_table(
                Table::create()
                    .table(InventoryRecord::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(InventoryRecord::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(InventoryRecord::Uuid)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(InventoryRecord::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(InventoryRecord::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(InventoryRecord::TenantId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InventoryRecord::OriginatingConnectionId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InventoryRecord::OriginalRecordBody)
                            .json_binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(InventoryRecord::SystemIdKey)
                            .enumeration(
                                SystemIdKey::Enum,
                                [
                                    SystemIdKey::Qbd,
                                    SystemIdKey::Qbo,
                                    SystemIdKey::Sapo,
                                ],
                            )
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(InventoryRecord::SystemId)
                            .string_len(255)
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(InventoryRecord::Table, InventoryRecord::TenantId)
                            .to(Tenant::Table, Tenant::Id)
                            .on_delete(ForeignKeyAction::Cascade)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                InventoryRecord::Table,
                                InventoryRecord::OriginatingConnectionId,
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
                    .name(InventoryRecordIndexes::InventoryRecordUuidIdx.to_string())
                    .table(InventoryRecord::Table)
                    .col(InventoryRecord::Uuid)
                    .unique()
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name(InventoryRecordIndexes::InventoryRecordTenantIdIdx.to_string())
                    .table(InventoryRecord::Table)
                    .col(InventoryRecord::TenantId)
                    .to_owned(),
            )
            .await?;

        let table_name = InventoryRecord::Table.to_string();
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
                    .table(InventoryRecord::Table)
                    .to_owned(),
            )
            .await?;
        manager
            .drop_type(Type::drop().name(SystemIdKey::Enum).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(Currency::Enum).to_owned())
            .await?;
        Ok(())
    }
}
