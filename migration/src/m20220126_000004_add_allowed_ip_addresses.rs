use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum AllowedIpAddress {
    Table,
    Id,
    Uuid,
    IpAddress,
    CreatedAt,
    UpdatedAt,
    Status,
}

#[derive(DeriveIden)]
enum AllowedIpAddressStatus {
    Enum,
    Active,
    Inactive,
    Banned,
}

#[derive(DeriveIden)]
enum AllowedIpAddressIndexes {
    UuidIdx,
    IdIdx,
    StatusIdx,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        //create enum type with unique name using raw SQL
        let db = manager.get_connection();
        db.execute_unprepared(
            r#"
            DO $$ BEGIN
                CREATE TYPE allowed_ip_address_status_enum AS ENUM ('active', 'inactive', 'banned');
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            "#,
        )
        .await?;
        
        manager.create_table(
            Table::create()
                .table(AllowedIpAddress::Table)
                .if_not_exists()
                .col(ColumnDef::new(AllowedIpAddress::Id)
                    .big_integer()
                    .not_null()
                    .auto_increment()
                    .primary_key()
                )
                .col(ColumnDef::new(AllowedIpAddress::Uuid)
                    .uuid()
                    .not_null()
                    .unique_key()
                )
                .col(ColumnDef::new(AllowedIpAddress::IpAddress)
                    .text()
                    .not_null()
                    .unique_key()
                )
                .col(ColumnDef::new(AllowedIpAddress::CreatedAt)
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Expr::current_timestamp())
                )
                .col(ColumnDef::new(AllowedIpAddress::UpdatedAt)
                    .timestamp_with_time_zone()
                    .not_null()
                    .default(Expr::current_timestamp())
                )
                .col(ColumnDef::new(AllowedIpAddress::Status)
                    .custom(sea_orm_migration::prelude::Alias::new("allowed_ip_address_status_enum"))
                    .not_null()
                    .default("active")
                )
                .to_owned()
        ).await?;

        manager.create_index(
            Index::create()
                .name(AllowedIpAddressIndexes::UuidIdx.to_string())
                .table(AllowedIpAddress::Table)
                .col(AllowedIpAddress::Uuid)
                .unique()
                .to_owned(),
        )
        .await?;

        manager.create_index(
            Index::create()
                .name(AllowedIpAddressIndexes::IdIdx.to_string())
                .table(AllowedIpAddress::Table)
                .col(AllowedIpAddress::Id)
                .unique()
                .to_owned()
        ).await?;

        manager.create_index(
            Index::create()
                .name(AllowedIpAddressIndexes::StatusIdx.to_string())
                .table(AllowedIpAddress::Table)
                .col(AllowedIpAddress::Status)
                .to_owned()
        ).await?;

        //add default UUID generation
        db.execute_unprepared(
            r#"
            CREATE EXTENSION IF NOT EXISTS "pgcrypto";
            "#,
        )
        .await?;

        let table_name = AllowedIpAddress::Table.to_string();
        db.execute_unprepared(
            &format!(
                r#"
                ALTER TABLE {}
                ALTER COLUMN uuid
                SET DEFAULT gen_random_uuid();
                "#,
                table_name
            ),
        )
        .await?;

        Ok(())
    }
    
}