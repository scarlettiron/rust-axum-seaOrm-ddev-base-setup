use sea_orm_migration::prelude::*;

// References connection_identity table from m20260129_000007_create_connection_identity_table
#[derive(DeriveIden)]
enum ConnectionIdentity {
    Table,
    Id,
}

// Existing enum from m20260129_000007_create_connection_identity_table (do not create)
#[derive(DeriveIden)]
enum ErpConnectionAuthTokenType {
    #[sea_orm(iden = "erp_connection_auth_token_type")]
    Enum,
    Bearer,
}

// Existing enum from m20260129_000007_create_connection_identity_table (do not create)
#[derive(DeriveIden)]
enum ErpConnectionReauthReason {
    #[sea_orm(iden = "erp_connection_reauth_reason")]
    Enum,
    RefreshExpired,
    Revoked,
    InvalidGrant,
    ScopesChanged,
}

#[derive(DeriveIden)]
enum ErpConnectionCredentials {
    Table,
    Id,
    Uuid,
    CreatedAt,
    UpdatedAt,
    ConnectionId,
    ClientId,
    IssuerBaseUrl,
    TokenType,
    ReauthRequiredReason,
    ReauthUrl,
    EncScheme,
    EncKeyId,
    EncVersion,
    EncIv,
    EncTag,
    AccessToken,
    RefreshToken,
    AccessTokenExpiresAt,
    RefreshTokenExpiresAt,
    IdTokenEnc,
    ProviderUserId,
    ProviderPassword,
    ClientCert,
    PrivateKey,
    CertExpiresAt,
    SessionToken,
    SessionExpiresAt,
    ApiAccessToken,
    ApiAccessTokenKey,
}

#[derive(DeriveIden)]
enum ErpConnectionCredentialsIndexes {
    ErpConnectionCredentialsUuidIdx,
    ErpConnectionCredentialsConnectionIdIdx,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(ErpConnectionCredentials::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::Id)
                            .big_integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::Uuid)
                            .uuid()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::ConnectionId)
                            .big_integer()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::ClientId)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::IssuerBaseUrl)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::TokenType)
                            .custom(ErpConnectionAuthTokenType::Enum.to_string())
                            .not_null()
                            .default(Expr::cust("'bearer'::erp_connection_auth_token_type")),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::ReauthRequiredReason)
                            .custom(ErpConnectionReauthReason::Enum.to_string())
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::ReauthUrl)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::EncScheme)
                            .text()
                            .not_null()
                            .default("kms-envelope-v1"),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::EncKeyId)
                            .text()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::EncVersion)
                            .integer()
                            .not_null()
                            .default(1),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::EncIv)
                            .binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::EncTag)
                            .binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::AccessToken)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::RefreshToken)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::AccessTokenExpiresAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::RefreshTokenExpiresAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::IdTokenEnc)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::ProviderUserId)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::ProviderPassword)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::ClientCert)
                            .binary()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::PrivateKey)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::CertExpiresAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::SessionToken)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::SessionExpiresAt)
                            .timestamp_with_time_zone()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::ApiAccessToken)
                            .text()
                            .null(),
                    )
                    .col(
                        ColumnDef::new(ErpConnectionCredentials::ApiAccessTokenKey)
                            .text()
                            .null(),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(
                                ErpConnectionCredentials::Table,
                                ErpConnectionCredentials::ConnectionId,
                            )
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
                    .name(ErpConnectionCredentialsIndexes::ErpConnectionCredentialsUuidIdx.to_string())
                    .table(ErpConnectionCredentials::Table)
                    .col(ErpConnectionCredentials::Uuid)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Unique index on connection_id (one credentials row per connection)
        manager
            .create_index(
                Index::create()
                    .name(
                        ErpConnectionCredentialsIndexes::ErpConnectionCredentialsConnectionIdIdx
                            .to_string(),
                    )
                    .table(ErpConnectionCredentials::Table)
                    .col(ErpConnectionCredentials::ConnectionId)
                    .unique()
                    .to_owned(),
            )
            .await?;

        // Default uuid to gen_random_uuid()
        let table_name = ErpConnectionCredentials::Table.to_string();
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

        // At least one credential column must be non-null
        manager
            .get_connection()
            .execute_unprepared(&format!(
                r#"
                ALTER TABLE {}
                ADD CONSTRAINT credentials_not_all_null CHECK (
                    access_token IS NOT NULL OR
                    refresh_token IS NOT NULL OR
                    provider_password IS NOT NULL OR
                    private_key IS NOT NULL OR
                    session_token IS NOT NULL OR
                    api_access_token IS NOT NULL
                );
                "#,
                table_name
            ))
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table(ErpConnectionCredentials::Table)
                    .to_owned(),
            )
            .await?;
        Ok(())
    }
}
