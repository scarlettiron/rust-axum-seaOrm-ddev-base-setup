use redis::{Client, aio::ConnectionManager};
use super::env;

///creates and returns an async Redis connection manager
///ConnectionManager can be cloned and shared across routes
pub async fn connect() -> Result<ConnectionManager, redis::RedisError> {
    let redis = &env::get().redis;

    tracing::info!("Connecting to Redis at {}...", redis.url);

    let client = Client::open(redis.url.as_str())?;
    let connection_manager = ConnectionManager::new(client).await?;

    tracing::info!("Redis connection established");

    Ok(connection_manager)
}

///gets a Redis client (for connection pooling or async operations)
pub fn get_client() -> Result<Client, redis::RedisError> {
    let redis = &env::get().redis;
    Client::open(redis.url.as_str())
}
