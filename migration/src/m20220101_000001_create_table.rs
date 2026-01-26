use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::Type;

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


#[derive(DeriveIden)]
enum ApiTokenStatus {
    Enum,
    Active,
    Inactive,
    Banned,
}

#[derive(DeriveIden)]
enum ApitTokenIndexes {
    UuidIdx,
    IdIdx,
    StatusIdx,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
   async fn up(&self, manager:&SchemaManager)  -> Result<(), DbErr>{
        manager.create_type(
            Type::create().as_enum(ApiTokenStatus::Enum).values(vec![
                ApiTokenStatus::Active,
                ApiTokenStatus::Inactive,
                ApiTokenStatus::Banned,
            ]).to_owned()
        ).await?;

        manager.create_table(
            Table::create().table(ApiToken::Table).if_not_exists()
            
            .col(ColumnDef:: new(ApiToken::Id)
            .big_integer().not_null()
            .auto_increment().primary_key()
            )

            .col(ColumnDef:: new(ApiToken::Uuid).uuid().not_null().unique_key())
            
            .col(ColumnDef:: new(ApiToken::Token).text().not_null().unique_key())
            
            .col(ColumnDef:: new(ApiToken::CreatedAt).timestamp_with_time_zone().not_null()
            .default(Expr::current_timestamp()))
            
            .col(ColumnDef:: new(ApiToken::UpdatedAt).timestamp_with_time_zone().not_null()
            .default(Expr::current_timestamp()))
            
            .col(ColumnDef:: new(ApiToken::Status).enumeration(
                ApiTokenStatus::Enum,
                [
                    ApiTokenStatus::Active,
                    ApiTokenStatus::Inactive,
                    ApiTokenStatus::Banned,
                ]
            ).not_null().default(ApiTokenStatus::Active.to_string()))

            .to_owned()
        ).await?;

       Ok(())
   }

}