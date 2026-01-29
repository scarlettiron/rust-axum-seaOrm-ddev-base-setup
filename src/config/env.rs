use axum::http::Method;
use std::env;
use std::sync::OnceLock;
use std::time::Duration;

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

///central application configuration loaded from environment variables
///mirrors NestJS ConfigModule pattern - all env vars defined in one place
#[derive(Debug)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub db: DatabaseConfig,
    pub redis: RedisConfig,
    pub cors: CorsConfig,
    pub hosts: HostsConfig,
    pub middleware: MiddlewareConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug)]
pub struct ServerConfig {
    pub port: String,
    pub rust_log: String,
}

#[derive(Debug)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: Duration,
    pub acquire_timeout: Duration,
    pub idle_timeout: Duration,
    pub max_lifetime: Duration,
    pub sqlx_logging: bool,
}

#[derive(Debug)]
pub struct RedisConfig {
    pub url: String,
}

#[derive(Debug)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<Method>,
    pub allowed_headers: Vec<String>,
    pub allow_credentials: bool,
}

#[derive(Debug)]
pub struct HostsConfig {
    pub allowed: Vec<String>,
}

#[derive(Debug)]
pub struct MiddlewareConfig {
    pub api_token_auth_enabled: bool,
    pub ip_address_auth_enabled: bool,
    pub request_logging_enabled: bool,
}

#[derive(Debug)]
pub struct LoggingConfig {
    pub sensitive_headers: Vec<String>,
}

impl AppConfig {
    ///loads configuration from environment variables with defaults
    fn from_env() -> Self {
        Self {
            server: ServerConfig {
                port: env::var("PORT").unwrap_or_else(|_| "3000".to_string()),
                rust_log: env::var("RUST_LOG").unwrap_or_else(|_| "debug".to_string()),
            },

            db: DatabaseConfig {
                url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgres://db:db@db:5432/db".to_string()),
                max_connections: env::var("DB_MAX_CONNECTIONS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(100),
                min_connections: env::var("DB_MIN_CONNECTIONS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(5),
                connect_timeout: Duration::from_secs(
                    env::var("DB_CONNECT_TIMEOUT")
                        .ok()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(8),
                ),
                acquire_timeout: Duration::from_secs(
                    env::var("DB_ACQUIRE_TIMEOUT")
                        .ok()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(8),
                ),
                idle_timeout: Duration::from_secs(
                    env::var("DB_IDLE_TIMEOUT")
                        .ok()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(8),
                ),
                max_lifetime: Duration::from_secs(
                    env::var("DB_MAX_LIFETIME")
                        .ok()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or(8),
                ),
                sqlx_logging: env::var("DB_SQLX_LOGGING")
                    .map(|v| v.to_lowercase() != "false" && v != "0")
                    .unwrap_or(true),
            },

            redis: RedisConfig {
                url: env::var("REDIS_URL")
                    .unwrap_or_else(|_| "redis://redis:6379".to_string()),
            },

            cors: CorsConfig {
                allowed_origins: env::var("CORS_ALLOWED_ORIGINS")
                    .map(|origins| {
                        origins
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect()
                    })
                    .unwrap_or_else(|_| vec!["https://erp-proxy-server.ddev.site".to_string()]),
                allowed_methods: vec![
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::OPTIONS,
                    Method::PATCH,
                ],
                allowed_headers: vec![
                    "authorization".to_string(),
                    "content-type".to_string(),
                    "x-requested-with".to_string(),
                    "x-custom-host".to_string(),
                    "accept".to_string(),
                    "origin".to_string(),
                ],
                allow_credentials: env::var("CORS_ALLOW_CREDENTIALS")
                    .map(|v| v.to_lowercase() != "false" && v != "0")
                    .unwrap_or(true),
            },

            hosts: HostsConfig {
                allowed: env::var("ALLOWED_HOSTS")
                    .map(|hosts| {
                        hosts
                            .split(',')
                            .map(|s| s.trim().to_string())
                            .filter(|s| !s.is_empty())
                            .collect()
                    })
                    .unwrap_or_else(|_| vec!["erp-proxy-server.ddev.site".to_string()]),
            },

            middleware: MiddlewareConfig {
                api_token_auth_enabled: env::var("API_TOKEN_AUTH_ENABLED")
                    .map(|v| v.to_lowercase() != "false" && v != "0")
                    .unwrap_or(true),
                ip_address_auth_enabled: env::var("IP_ADDRESS_AUTH_ENABLED")
                    .map(|v| v.to_lowercase() != "false" && v != "0")
                    .unwrap_or(true),
                request_logging_enabled: env::var("REQUEST_LOGGING")
                    .map(|v| v.to_lowercase() != "false" && v != "0")
                    .unwrap_or(true),
            },

            logging: LoggingConfig {
                sensitive_headers: vec![
                    "authorization".to_string(),
                    "cookie".to_string(),
                    "set-cookie".to_string(),
                    "x-api-key".to_string(),
                    "x-auth-token".to_string(),
                    "x-access-token".to_string(),
                    "x-refresh-token".to_string(),
                    "proxy-authorization".to_string(),
                ],
            },
        }
    }
}

///initializes the global config from environment variables
///call once at startup after dotenvy::dotenv()
pub fn init() {
    if CONFIG.set(AppConfig::from_env()).is_err() {
        panic!("Config already initialized");
    }
    tracing::info!("Application config loaded");
}

///returns a reference to the global application config
///panics if config has not been initialized
pub fn get() -> &'static AppConfig {
    CONFIG.get().expect("Config not initialized - call config::env::init() first")
}
