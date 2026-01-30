pub use sea_orm_migration::prelude::*;

mod m20220101_000001_create_table;
mod m20220126_000002_make_api_token_unique;
mod m20220126_000003_add_default_uuid_to_api_token;
mod m20220126_000004_add_allowed_ip_addresses;
mod m20220126_000005_rename_api_token_enum;
mod m20260128_0000006_create_tenant_table;
mod m20260129_000007_create_connection_identity_table;
mod m20260130_000008_create_erp_connection_sync_state_table;
mod m20260130_000009_create_erp_connection_credentials_table;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
           Box::new(m20220101_000001_create_table::Migration),
           Box::new(m20220126_000002_make_api_token_unique::Migration),
           Box::new(m20220126_000003_add_default_uuid_to_api_token::Migration),
           Box::new(m20220126_000004_add_allowed_ip_addresses::Migration),
           Box::new(m20220126_000005_rename_api_token_enum::Migration),
           Box::new(m20260128_0000006_create_tenant_table::Migration),
           Box::new(m20260129_000007_create_connection_identity_table::Migration),
           Box::new(m20260130_000008_create_erp_connection_sync_state_table::Migration),
           Box::new(m20260130_000009_create_erp_connection_credentials_table::Migration),
        ]
    }
}
