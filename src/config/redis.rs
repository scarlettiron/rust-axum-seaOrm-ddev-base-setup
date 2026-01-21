use redis::{Client, aio::ConnectionManager};
use std::env;

///default Redis URL if REDIS_URL env var is not set
///format: redis://[host]:[port]
const DEFAULT_REDIS_URL: &str = "redis://redis:6379";

///gets Redis URL from REDIS_URL env var
///falls back to default DDEV Redis URL if not set
fn get_redis_url() -> String {
    env::var("REDIS_URL").unwrap_or_else(|_| DEFAULT_REDIS_URL.to_string())
}

///creates and returns an async Redis connection manager
///ConnectionManager can be cloned and shared across routes
pub async fn connect() -> Result<ConnectionManager, redis::RedisError> {
    let redis_url = get_redis_url();

    tracing::info!("Connecting to Redis at {}...", redis_url);

    let client = Client::open(redis_url.as_str())?;
    let connection_manager = ConnectionManager::new(client).await?;

    tracing::info!("Redis connection established");

    Ok(connection_manager)
}

///gets a Redis client (for connection pooling or async operations)
pub fn get_client() -> Result<Client, redis::RedisError> {
    let redis_url = get_redis_url();
    Client::open(redis_url.as_str())
}
