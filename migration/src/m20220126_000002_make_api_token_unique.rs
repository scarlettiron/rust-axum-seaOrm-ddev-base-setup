use sea_orm_migration::prelude::*;

#[derive(DeriveIden)]
enum ApiToken {
    Table,
    Token,
}

#[derive(DeriveIden)]
enum ApiTokenIndexes {
    TokenUniqueIdx,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // Add UNIQUE index on token
        manager
            .create_index(
                Index::create()
                    .name(ApiTokenIndexes::TokenUniqueIdx.to_string())
                    .table(ApiToken::Table)
                    .col(ApiToken::Token)
                    .unique()
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

}
