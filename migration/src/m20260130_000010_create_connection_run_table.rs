use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::Type;

// ── Enums ──

#[derive(DeriveIden)]
enum ConnectionRunStatus {
    #[sea_orm(iden = "connection_run_status")]
    Enum,
    Success,
    Error,
}

#[derive(DeriveIden)]
enum ConnectionRunType {
    #[sea_orm(iden = "connection_run_type")]
    Enum,
    Poll,
}

// References connection_identity table from m20260129_000007_create_connection_identity_table
#[derive(DeriveIden)]
enum ConnectionIdentity {
    Table,
    Id,
}

// ── Table ──

#[derive(DeriveIden)]
enum ConnectionRun {
    Table,
    Id,
    Uuid,
    CreatedAt,
    UpdatedAt,
    Status,
    ErrorMessage,
    RunType,
    ConnectionId,
}

#[derive(DeriveIden)]
enum ConnectionRunIndexes {
    ConnectionRunUuidIdx,
    ConnectionRunConnectionIdIdx,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Create enums
        manager
            .create_type(
                Type::create()
                    .as_enum(ConnectionRunStatus::Enum)
                    .values(vec![ConnectionRunStatus::Success, ConnectionRunStatus::Error])
                    .to_owned(),
            )
            .await?;

        manager
            .create_type(
                Type::create()
                    .as_enum(ConnectionRunType::Enum)
                    .values(vec![ConnectionRunType::Poll])
                    .to_owned(),
            )
            .await?;

        // Create table
        manager
            .create_table(
                Table::create()
                    .table(ConnectionRun::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ConnectionRun::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ConnectionRun::Uuid)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(ConnectionRun::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ConnectionRun::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ConnectionRun::Status)
                            .enumeration(
                                ConnectionRunStatus::Enum,
                                [ConnectionRunStatus::Success, ConnectionRunStatus::Error],
                            )
                            .not_null()
                            .default(ConnectionRunStatus::Success.to_string()),
                    )
                    .col(
                        ColumnDef::new(ConnectionRun::ErrorMessage)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ConnectionRun::RunType)
                            .enumeration(ConnectionRunType::Enum, [ConnectionRunType::Poll])
                            .not_null()
                            .default(ConnectionRunType::Poll.to_string()),
                    )
                    .col(
                        ColumnDef::new(ConnectionRun::ConnectionId)
                            .big_integer()
                            .not_null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(ConnectionRun::Table, ConnectionRun::ConnectionId)
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
                    .name(ConnectionRunIndexes::ConnectionRunUuidIdx.to_string())
                    .table(ConnectionRun::Table)
                    .col(ConnectionRun::Uuid)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Index on connection_id (many runs per connection)
        manager
            .create_index(
                Index::create()
                    .name(ConnectionRunIndexes::ConnectionRunConnectionIdIdx.to_string())
                    .table(ConnectionRun::Table)
                    .col(ConnectionRun::ConnectionId)
                    .to_owned(),
            )
            .await?;

        // Default uuid to gen_random_uuid()
        let table_name = ConnectionRun::Table.to_string();
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
            .drop_table(Table::drop().table(ConnectionRun::Table).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(ConnectionRunType::Enum).to_owned())
            .await?;
        manager
            .drop_type(Type::drop().name(ConnectionRunStatus::Enum).to_owned())
            .await?;
        Ok(())
    }
}
