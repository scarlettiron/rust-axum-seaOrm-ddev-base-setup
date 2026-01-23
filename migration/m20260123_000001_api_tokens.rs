use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum ApiToken {
    Table,
    Id,
    Uuid,
    Token,
    Status,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.create_table(Table::create().table(ApiToken::Table)
            .if_not_exists()
            .col(ColumnDef::new(ApiToken::Id).integer().not_null().auto_increment().primary_key())
            .col(ColumnDef::new(ApiToken::Uuid).uuid().not_null().unique_key())
            .col(ColumnDef::new(ApiToken::Token).text().not_null())
            .col(ColumnDef::new(ApiToken::Status).string().not_null().default("active"))
            .col(ColumnDef::new(ApiToken::CreatedAt).timestamp().not_null())
            .col(ColumnDef::new(ApiToken::UpdatedAt).timestamp().not_null())
            .to_owned())
            .await?;

        // Add check constraint for status column using raw SQL
        let db = manager.get_connection();
        let table_name = ApiToken::Table.to_string();
        db.execute_unprepared(
            &format!(
                r#"
                ALTER TABLE {} 
                ADD CONSTRAINT check_status 
                CHECK (status IN ('active', 'inactive', 'banned'))
                "#,
                table_name
            ),
        )
        .await?;
    }
}