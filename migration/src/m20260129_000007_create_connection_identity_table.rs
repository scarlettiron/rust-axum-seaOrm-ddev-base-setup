use sea_orm_migration::prelude::*;
use sea_orm_migration::prelude::extension::postgres::Type;

// ── Enums ──

#[derive(DeriveIden)]
enum ErpProvider {
    #[sea_orm(iden = "erp_provider")]
    Enum,
    Quickbooks,
    Dmsi,
    Sap,
    Salesforce,
}

#[derive(DeriveIden)]
enum ErpProviderType {
    #[sea_orm(iden = "erp_provider_type")]
    Enum,
    Desktop,
    Api,
    Edi,
    Idoc,
    Webconnector,
}

#[derive(DeriveIden)]
enum ErpProviderAuthType {
    #[sea_orm(iden = "erp_provider_auth_type")]
    Enum,
    Oauth,
    Oauth2,
    UsernamePassword,
    Certificate,
    ApiToken,
    SessionToken,
}

#[derive(DeriveIden)]
enum ErpEnvironment {
    #[sea_orm(iden = "erp_environment")]
    Enum,
    Production,
    Sandbox,
}

#[derive(DeriveIden)]
enum ErpConnectionStatus {
    #[sea_orm(iden = "erp_connection_status")]
    Enum,
    Removed,
    Active,
}

#[derive(DeriveIden)]
enum ErpConnectionAuthStatus {
    #[sea_orm(iden = "erp_connection_auth_status")]
    Enum,
    Connected,
    NeedsReauth,
    Revoked,
    Error,
}

#[derive(DeriveIden)]
enum ErpConnectionAuthTokenType {
    #[sea_orm(iden = "erp_connection_auth_token_type")]
    Enum,
    Bearer,
}

#[derive(DeriveIden)]
enum ErpConnectionReauthReason {
    #[sea_orm(iden = "erp_connection_reauth_reason")]
    Enum,
    RefreshExpired,
    Revoked,
    InvalidGrant,
    ScopesChanged,
}

// ── Table ──

#[derive(DeriveIden)]
enum ConnectionIdentity {
    Table,
    Id,
    Uuid,
    TenantId,
    ErpProvider,
    ErpType,
    ErpAuthType,
    DisplayName,
    Environment,
    Status,
    AuthStatus,
    CreatedAt,
    UpdatedAt,
    IsEnabled,
    LastSuccessAt,
    LastErrorCode,
    LastErrorMessage,
    ErrorAt,
    SyncEnabledPush,
    SyncEnabledPull,
    SecretStorageRef,
    SecretVersion,
    Scopes,
    ProviderRealmId,
    ProviderTenantId,
    CompanyFileIdentity,
    CompanyFilePath,
    CompanyFileId,
    SystemVersion,
    WebConnectorAppName,
}

#[derive(DeriveIden)]
enum Tenant {
    Table,
    Id,
}

#[derive(DeriveIden)]
enum ConnectionIdentityIndexes {
    ConnectionIdentityUuidIdx,
    ConnectionIdentityTenantIdIdx,
    ConnectionIdentityStatusIdx,
    ConnectionIdentityAuthStatusIdx,
    ConnectionIdentityProviderIdx,
}

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        // ── Create enums ──

        manager.create_type(
            Type::create().as_enum(ErpProvider::Enum).values(vec![
                ErpProvider::Quickbooks,
                ErpProvider::Dmsi,
                ErpProvider::Sap,
                ErpProvider::Salesforce,
            ]).to_owned()
        ).await?;

        manager.create_type(
            Type::create().as_enum(ErpProviderType::Enum).values(vec![
                ErpProviderType::Desktop,
                ErpProviderType::Api,
                ErpProviderType::Edi,
                ErpProviderType::Idoc,
                ErpProviderType::Webconnector,
            ]).to_owned()
        ).await?;

        manager.create_type(
            Type::create().as_enum(ErpProviderAuthType::Enum).values(vec![
                ErpProviderAuthType::Oauth,
                ErpProviderAuthType::Oauth2,
                ErpProviderAuthType::UsernamePassword,
                ErpProviderAuthType::Certificate,
                ErpProviderAuthType::ApiToken,
                ErpProviderAuthType::SessionToken,
            ]).to_owned()
        ).await?;

        manager.create_type(
            Type::create().as_enum(ErpEnvironment::Enum).values(vec![
                ErpEnvironment::Production,
                ErpEnvironment::Sandbox,
            ]).to_owned()
        ).await?;

        manager.create_type(
            Type::create().as_enum(ErpConnectionStatus::Enum).values(vec![
                ErpConnectionStatus::Removed,
                ErpConnectionStatus::Active,
            ]).to_owned()
        ).await?;

        manager.create_type(
            Type::create().as_enum(ErpConnectionAuthStatus::Enum).values(vec![
                ErpConnectionAuthStatus::Connected,
                ErpConnectionAuthStatus::NeedsReauth,
                ErpConnectionAuthStatus::Revoked,
                ErpConnectionAuthStatus::Error,
            ]).to_owned()
        ).await?;

        manager.create_type(
            Type::create().as_enum(ErpConnectionAuthTokenType::Enum).values(vec![
                ErpConnectionAuthTokenType::Bearer,
            ]).to_owned()
        ).await?;

        manager.create_type(
            Type::create().as_enum(ErpConnectionReauthReason::Enum).values(vec![
                ErpConnectionReauthReason::RefreshExpired,
                ErpConnectionReauthReason::Revoked,
                ErpConnectionReauthReason::InvalidGrant,
                ErpConnectionReauthReason::ScopesChanged,
            ]).to_owned()
        ).await?;

        // ── Create table ──

        manager.create_table(
            Table::create().table(ConnectionIdentity::Table).if_not_exists()
                // core identity
                .col(ColumnDef::new(ConnectionIdentity::Id).big_integer().not_null().auto_increment().primary_key())
                .col(ColumnDef::new(ConnectionIdentity::Uuid).uuid().not_null().unique_key())
                .col(ColumnDef::new(ConnectionIdentity::TenantId).big_integer().not_null())
                .col(ColumnDef::new(ConnectionIdentity::ErpProvider).enumeration(ErpProvider::Enum, [
                    ErpProvider::Quickbooks, ErpProvider::Dmsi, ErpProvider::Sap, ErpProvider::Salesforce,
                ]).not_null())
                .col(ColumnDef::new(ConnectionIdentity::ErpType).enumeration(ErpProviderType::Enum, [
                    ErpProviderType::Desktop, ErpProviderType::Api, ErpProviderType::Edi, ErpProviderType::Idoc, ErpProviderType::Webconnector,
                ]).not_null())
                .col(ColumnDef::new(ConnectionIdentity::ErpAuthType).enumeration(ErpProviderAuthType::Enum, [
                    ErpProviderAuthType::Oauth, ErpProviderAuthType::Oauth2, ErpProviderAuthType::UsernamePassword,
                    ErpProviderAuthType::Certificate, ErpProviderAuthType::ApiToken, ErpProviderAuthType::SessionToken,
                ]).not_null())
                .col(ColumnDef::new(ConnectionIdentity::DisplayName).text().null())
                .col(ColumnDef::new(ConnectionIdentity::Environment).enumeration(ErpEnvironment::Enum, [
                    ErpEnvironment::Production, ErpEnvironment::Sandbox,
                ]).not_null().default(ErpEnvironment::Production.to_string()))
                .col(ColumnDef::new(ConnectionIdentity::Status).enumeration(ErpConnectionStatus::Enum, [
                    ErpConnectionStatus::Removed, ErpConnectionStatus::Active,
                ]).not_null().default(ErpConnectionStatus::Active.to_string()))
                .col(ColumnDef::new(ConnectionIdentity::AuthStatus).enumeration(ErpConnectionAuthStatus::Enum, [
                    ErpConnectionAuthStatus::Connected, ErpConnectionAuthStatus::NeedsReauth,
                    ErpConnectionAuthStatus::Revoked, ErpConnectionAuthStatus::Error,
                ]).not_null().default(ErpConnectionAuthStatus::Connected.to_string()))
                .col(ColumnDef::new(ConnectionIdentity::CreatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                .col(ColumnDef::new(ConnectionIdentity::UpdatedAt).timestamp_with_time_zone().not_null().default(Expr::current_timestamp()))
                .col(ColumnDef::new(ConnectionIdentity::IsEnabled).boolean().not_null().default(true))

                // error tracking
                .col(ColumnDef::new(ConnectionIdentity::LastSuccessAt).timestamp_with_time_zone().null())
                .col(ColumnDef::new(ConnectionIdentity::LastErrorCode).string_len(255).null())
                .col(ColumnDef::new(ConnectionIdentity::LastErrorMessage).string_len(1024).null())
                .col(ColumnDef::new(ConnectionIdentity::ErrorAt).timestamp_with_time_zone().null())

                // sync config
                .col(ColumnDef::new(ConnectionIdentity::SyncEnabledPush).boolean().not_null().default(true))
                .col(ColumnDef::new(ConnectionIdentity::SyncEnabledPull).boolean().not_null().default(true))

                // secrets
                .col(ColumnDef::new(ConnectionIdentity::SecretStorageRef).text().null())
                .col(ColumnDef::new(ConnectionIdentity::SecretVersion).string_len(255).null())
                .col(ColumnDef::new(ConnectionIdentity::Scopes).array(ColumnType::Text).null())

                // provider-specific identifiers
                .col(ColumnDef::new(ConnectionIdentity::ProviderRealmId).string_len(255).null())
                .col(ColumnDef::new(ConnectionIdentity::ProviderTenantId).string_len(255).null())

                // company file (desktop ERPs)
                .col(ColumnDef::new(ConnectionIdentity::CompanyFileIdentity).text().null())
                .col(ColumnDef::new(ConnectionIdentity::CompanyFilePath).text().null())
                .col(ColumnDef::new(ConnectionIdentity::CompanyFileId).string_len(255).null())

                // desktop/connector metadata
                .col(ColumnDef::new(ConnectionIdentity::SystemVersion).string_len(255).null())
                .col(ColumnDef::new(ConnectionIdentity::WebConnectorAppName).string_len(255).null())

                // foreign key
                .foreign_key(
                    ForeignKey::create()
                        .from(ConnectionIdentity::Table, ConnectionIdentity::TenantId)
                        .to(Tenant::Table, Tenant::Id)
                        .on_delete(ForeignKeyAction::Cascade)
                        .on_update(ForeignKeyAction::Cascade)
                )
                .to_owned()
        ).await?;

        // ── Indexes ──

        manager.create_index(
            Index::create()
                .name(ConnectionIdentityIndexes::ConnectionIdentityUuidIdx.to_string())
                .table(ConnectionIdentity::Table)
                .col(ConnectionIdentity::Uuid)
                .unique()
                .to_owned()
        ).await?;

        manager.create_index(
            Index::create()
                .name(ConnectionIdentityIndexes::ConnectionIdentityTenantIdIdx.to_string())
                .table(ConnectionIdentity::Table)
                .col(ConnectionIdentity::TenantId)
                .to_owned()
        ).await?;

        manager.create_index(
            Index::create()
                .name(ConnectionIdentityIndexes::ConnectionIdentityStatusIdx.to_string())
                .table(ConnectionIdentity::Table)
                .col(ConnectionIdentity::Status)
                .to_owned()
        ).await?;

        manager.create_index(
            Index::create()
                .name(ConnectionIdentityIndexes::ConnectionIdentityAuthStatusIdx.to_string())
                .table(ConnectionIdentity::Table)
                .col(ConnectionIdentity::AuthStatus)
                .to_owned()
        ).await?;

        manager.create_index(
            Index::create()
                .name(ConnectionIdentityIndexes::ConnectionIdentityProviderIdx.to_string())
                .table(ConnectionIdentity::Table)
                .col(ConnectionIdentity::ErpProvider)
                .to_owned()
        ).await?;

        // ── Auto-generate UUID default ──

        let table_name = ConnectionIdentity::Table.to_string();
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

   /*  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager.drop_table(Table::drop().table(ConnectionIdentity::Table).to_owned()).await?;

        manager.drop_type(Type::drop().name(ErpConnectionReauthReason::Enum).to_owned()).await?;
        manager.drop_type(Type::drop().name(ErpConnectionAuthTokenType::Enum).to_owned()).await?;
        manager.drop_type(Type::drop().name(ErpConnectionAuthStatus::Enum).to_owned()).await?;
        manager.drop_type(Type::drop().name(ErpConnectionStatus::Enum).to_owned()).await?;
        manager.drop_type(Type::drop().name(ErpEnvironment::Enum).to_owned()).await?;
        manager.drop_type(Type::drop().name(ErpProviderAuthType::Enum).to_owned()).await?;
        manager.drop_type(Type::drop().name(ErpProviderType::Enum).to_owned()).await?;
        manager.drop_type(Type::drop().name(ErpProvider::Enum).to_owned()).await?;

        Ok(())
    } */
}
