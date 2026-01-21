use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use std::env;
use std::time::Duration;

///default database URL if DATABASE_URL env var is not set
const DEFAULT_DATABASE_URL: &str = "postgres://db:db@db:5432/db";

///gets database URL from DATABASE_URL env var
///falls back to default DDEV database URL if not set
fn get_database_url() -> String {
    env::var("DATABASE_URL").unwrap_or_else(|_| DEFAULT_DATABASE_URL.to_string())
}

///creates and returns a database connection
pub async fn connect() -> Result<DatabaseConnection, sea_orm::DbErr> {
    let database_url = get_database_url();

    let mut opt = ConnectOptions::new(&database_url);
    opt.max_connections(100)
        .min_connections(5)
        .connect_timeout(Duration::from_secs(8))
        .acquire_timeout(Duration::from_secs(8))
        .idle_timeout(Duration::from_secs(8))
        .max_lifetime(Duration::from_secs(8))
        .sqlx_logging(true);

    tracing::info!("Connecting to database...");

    let db = Database::connect(opt).await?;

    tracing::info!("Database connection established");

    Ok(db)
}
