use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::Type;

#[derive(DeriveIden)]
enum Tenant {
    Table,
    Id,
    Uuid,
    DisplayName,
    TenantId,
    CreatedAt,
    UpdatedAt,
    Status,
}

#[derive(DeriveIden)]
enum TenantStatus {
    Enum,
    Active,
    Removed,
}

#[derive(DeriveIden)]
enum TenantIndexes {
    TenantUuidIdx,
    TenantIdIdx,
    TenantStatusIdx,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {          
                // Needed for gen_random_uuid()
                manager
                .get_connection()
                .execute_unprepared(r#"CREATE EXTENSION IF NOT EXISTS "pgcrypto";"#)
                .await?;
        
        manager.create_type(
            Type::create().as_enum(TenantStatus::Enum).values(vec![
                TenantStatus::Active,
                TenantStatus::Removed,
            ]).to_owned()
        ).await?;


        manager.create_table(
            Table::create().table(Tenant::Table).if_not_exists()
            .col(ColumnDef::new(Tenant::Id).big_integer().not_null().auto_increment().primary_key())
            .col(ColumnDef::new(Tenant::Uuid).uuid().not_null().unique_key())
            .col(ColumnDef::new(Tenant::DisplayName).text().null())
            .col(ColumnDef::new(Tenant::TenantId).text().not_null())        
            .col(ColumnDef::new(Tenant::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .col(ColumnDef::new(Tenant::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
            .col(ColumnDef::new(Tenant::Status).enumeration(TenantStatus::Enum, [TenantStatus::Active, TenantStatus::Removed]).not_null().default(TenantStatus::Active.to_string()))
            .to_owned()
        ).await?;

        manager.create_index(
            Index::create()
            .name(TenantIndexes::TenantUuidIdx.to_string())
            .table(Tenant::Table)
            .col(Tenant::Uuid)
            .unique()
            .to_owned()
        ).await?;

        manager.create_index(
            Index::create()
            .name(TenantIndexes::TenantIdIdx.to_string())
            .table(Tenant::Table)
            .col(Tenant::TenantId)
            .unique()
            .to_owned()
        ).await?;

        manager.create_index(
            Index::create()
            .name(TenantIndexes::TenantStatusIdx.to_string())
            .table(Tenant::Table)
            .col(Tenant::Status)
            .to_owned()
        ).await?;
                
        let table_name = Tenant::Table.to_string();
        manager.get_connection().execute_unprepared(
            &format!(
                r#"
                ALTER TABLE {}
                ALTER COLUMN uuid
                SET DEFAULT gen_random_uuid();
                "#,
                table_name
            ),
        ).await?;

        Ok(())
    }
}
