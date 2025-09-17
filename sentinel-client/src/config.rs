use anyhow::Result;
use config::{Config as ConfigBuilder, ConfigError, File};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub client: ClientConfig,
    pub server: ServerConfig,
    pub proxy: ProxyConfig,
    pub transport: TransportConfig,
    pub limits: LimitsConfig,
    pub monitoring: MonitoringConfig,
    pub logging: LoggingConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    pub id: Option<String>,
    pub hostname: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    pub url: String,
    pub token: Option<String>,
    pub heartbeat_interval: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyConfig {
    pub listen_addr: String,
    pub target_addr: String,
    pub buffer_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransportConfig {
    #[serde(rename = "type")]
    pub transport_type: String,
    pub encryption_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LimitsConfig {
    pub max_connections: usize,
    pub rate_limit_mbps: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub enabled: bool,
    pub report_interval: u64,
    pub collect_interval: u64,
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
            .set_default("client.id", "")?
            .set_default("client.hostname", "")?
            .set_default("server.heartbeat_interval", 30)?
            .set_default("proxy.listen_addr", "0.0.0.0:0")?
            .set_default("proxy.target_addr", "127.0.0.1:8080")?
            .set_default("proxy.buffer_size", 8192)?
            .set_default("transport.type", "direct")?
            .set_default("limits.max_connections", 1000)?
            .set_default("limits.rate_limit_mbps", 0)?
            .set_default("monitoring.enabled", true)?
            .set_default("monitoring.report_interval", 30)?
            .set_default("monitoring.collect_interval", 1)?
            .set_default("logging.level", "info")?
            .build()?;

        config.try_deserialize()
    }
}