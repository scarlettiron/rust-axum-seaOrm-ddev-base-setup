use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum ApiToken {
    Table,
    Uuid,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();

        db.execute_unprepared(
            r#"
            CREATE EXTENSION IF NOT EXISTS "pgcrypto";
            "#,
        )
        .await?;

        db.execute_unprepared(
            r#"
            ALTER TABLE api_token
            ALTER COLUMN uuid
            SET DEFAULT gen_random_uuid();
            "#,
        )
        .await?;

        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Err(DbErr::Custom(
            "Down migration disabled for safety. Use a forward migration instead.".into(),
        ))
    }
}
