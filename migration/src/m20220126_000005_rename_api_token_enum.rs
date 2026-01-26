use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum ApiToken {
    Table,
    Status,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let table_name = ApiToken::Table.to_string();
        let column_name = ApiToken::Status.to_string();

        //create new enum type with unique name
        db.execute_unprepared(
            r#"
            DO $$ BEGIN
                CREATE TYPE api_token_status_enum AS ENUM ('active', 'inactive', 'banned');
            EXCEPTION
                WHEN duplicate_object THEN null;
            END $$;
            "#,
        )
        .await?;

        //drop default constraint first
        db.execute_unprepared(
            &format!(
                r#"
                ALTER TABLE {}
                ALTER COLUMN {}
                DROP DEFAULT;
                "#,
                table_name, column_name
            ),
        )
        .await?;

        //migrate data: convert column to text, then to new enum type
        db.execute_unprepared(
            &format!(
                r#"
                ALTER TABLE {}
                ALTER COLUMN {}
                TYPE api_token_status_enum
                USING {}::text::api_token_status_enum;
                "#,
                table_name, column_name, column_name
            ),
        )
        .await?;

        //restore default with new enum type
        db.execute_unprepared(
            &format!(
                r#"
                ALTER TABLE {}
                ALTER COLUMN {}
                SET DEFAULT 'active'::api_token_status_enum;
                "#,
                table_name, column_name
            ),
        )
        .await?;

        //drop old enum type if it exists and is not used elsewhere
        db.execute_unprepared(
            r#"
            DO $$ BEGIN
                DROP TYPE IF EXISTS "enum";
            EXCEPTION
                WHEN dependent_objects_still_exist THEN null;
            END $$;
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
