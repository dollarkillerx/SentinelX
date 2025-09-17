use anyhow::Result;
use config::{Config as ConfigBuilder, ConfigError, File};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub security: SecurityConfig,
    pub client_management: ClientManagementConfig,
    pub api: ApiConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub bind_addr: String,
    pub workers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub jwt_secret: String,
    pub token_expiry: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientManagementConfig {
    pub heartbeat_timeout: u64,
    pub cleanup_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub rate_limit: u32,
    pub max_request_size: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<String>,
}

impl Config {
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        let config = ConfigBuilder::builder()
            .add_source(File::with_name(path))
            .set_default("server.bind_addr", "0.0.0.0:8080")?
            .set_default("server.workers", 4)?
            .set_default("database.max_connections", 10)?
            .set_default("database.min_connections", 1)?
            .set_default("security.token_expiry", 3600)?
            .set_default("client_management.heartbeat_timeout", 120)?
            .set_default("client_management.cleanup_interval", 60)?
            .set_default("api.rate_limit", 100)?
            .set_default("api.max_request_size", "10MB")?
            .set_default("logging.level", "info")?
            .build()?;

        config.try_deserialize()
    }
}