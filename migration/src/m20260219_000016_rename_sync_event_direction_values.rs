use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                ALTER TYPE sync_event_direction RENAME VALUE 'push' TO 'push_to_external';
                ALTER TYPE sync_event_direction RENAME VALUE 'pull' TO 'pull_from_external';
                "#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"
                ALTER TYPE sync_event_direction RENAME VALUE 'push_to_external' TO 'push';
                ALTER TYPE sync_event_direction RENAME VALUE 'pull_from_external' TO 'pull';
                "#,
            )
            .await?;
        Ok(())
    }
}
