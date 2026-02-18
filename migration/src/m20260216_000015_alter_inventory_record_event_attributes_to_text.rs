use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum InventoryRecordEvent {
    Table,
    Attributes,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Change attributes from text[] to text (textarea). Convert existing array to newline-separated string.
        manager
            .get_connection()
            .execute_unprepared(
                r#"ALTER TABLE inventory_record_event
                   ALTER COLUMN attributes TYPE text
                   USING (CASE WHEN attributes IS NULL THEN NULL ELSE array_to_string(attributes, E'\n') END)"#,
            )
            .await?;
        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(
                r#"ALTER TABLE inventory_record_event
                   ALTER COLUMN attributes TYPE text[] USING string_to_array(attributes, E'\n')"#,
            )
            .await?;
        Ok(())
    }
}
