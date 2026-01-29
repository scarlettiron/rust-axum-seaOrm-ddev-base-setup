use sea_orm::{ConnectOptions, Database, DatabaseConnection};
use super::env;

///creates and returns a database connection using central config
pub async fn connect() -> Result<DatabaseConnection, sea_orm::DbErr> {
    let db = &env::get().db;

    let mut opt = ConnectOptions::new(&db.url);
    opt.max_connections(db.max_connections)
        .min_connections(db.min_connections)
        .connect_timeout(db.connect_timeout)
        .acquire_timeout(db.acquire_timeout)
        .idle_timeout(db.idle_timeout)
        .max_lifetime(db.max_lifetime)
        .sqlx_logging(db.sqlx_logging);

    tracing::info!("Connecting to database...");

    let conn = Database::connect(opt).await?;

    tracing::info!("Database connection established");

    Ok(conn)
}
