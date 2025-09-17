mod config;
mod register;
mod monitor;
mod proxy;
mod iptables;
mod stats;
mod limiter;
mod relay;
mod encryption;
mod websocket;

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tracing_subscriber;

use crate::config::Config;
use crate::monitor::{MetricsReporter, get_system_info};
use crate::proxy::ProxyServer;
use crate::register::RegistrationManager;
use crate::relay::RelayManager;
use crate::iptables::IptablesManager;
use sentinel_common::{RelayConfig, TaskType};
use sentinel_common::ClientInfo;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sentinel=info".into()),
        )
        .init();

    let args = Args::parse();
    let config = Config::from_file(&args.config)?;

    tracing::info!("Starting Sentinel Client v{}", env!("CARGO_PKG_VERSION"));

    let system_info = get_system_info();
    tracing::info!(
        "System: {} {}, CPU: {} cores, Memory: {} GB",
        system_info.os,
        system_info.kernel_version,
        system_info.cpu_cores,
        system_info.total_memory / (1024 * 1024 * 1024)
    );

    let client_info = ClientInfo {
        id: config.client.id.clone().unwrap_or_else(|| {
            uuid::Uuid::new_v4().to_string()
        }),
        hostname: config.client.hostname.clone().unwrap_or_else(|| {
            hostname::get()
                .ok()
                .and_then(|h| h.to_str().map(String::from))
                .unwrap_or_else(|| "unknown".to_string())
        }),
        ip: get_local_ip()?,
        version: env!("CARGO_PKG_VERSION").to_string(),
        capabilities: vec![
            "proxy".to_string(),
            "iptables".to_string(),
            "relay".to_string(),
            "monitoring".to_string(),
        ],
        system_info,
    };

    let registration = Arc::new(RegistrationManager::new(
        client_info,
        config.server.url.clone(),
        std::time::Duration::from_secs(config.server.heartbeat_interval),
    ));

    let reg_handle = {
        let registration = registration.clone();
        tokio::spawn(async move { registration.start().await })
    };

    let monitor_handle = if config.monitoring.enabled {
        let reporter = MetricsReporter::new(
            config.server.url.clone(),
            std::time::Duration::from_secs(config.monitoring.report_interval),
        );

        Some(tokio::spawn(async move { reporter.start().await }))
    } else {
        None
    };

    let proxy = ProxyServer::new(
        config.proxy.listen_addr.parse()?,
        config.proxy.target_addr.parse()?,
    );

    let relay_manager = Arc::new(RelayManager::new(config.server.url.clone())?);
    let iptables_manager = Arc::new(IptablesManager::new());

    let proxy_handle = tokio::spawn(async move { proxy.start().await });

    // Start task manager to handle all server tasks
    let task_handle = {
        let relay_manager = relay_manager.clone();
        let iptables_manager = iptables_manager.clone();
        let registration_clone = registration.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));

            loop {
                interval.tick().await;

                // Get pending tasks from server
                if let Ok(tasks) = registration_clone.get_pending_tasks().await {
                    for task in tasks {
                        match task.task_type {
                            TaskType::StartRelay => {
                                if let Ok(config) = serde_json::from_value::<RelayConfig>(task.payload) {
                                    if let Err(e) = relay_manager.start_relay(config).await {
                                        tracing::error!("Failed to start relay: {}", e);
                                    }
                                }
                            }
                            TaskType::StopRelay => {
                                if let Ok(config) = serde_json::from_value::<RelayConfig>(task.payload) {
                                    if let Err(e) = relay_manager.stop_relay(&config.entry_point, &config.exit_point).await {
                                        tracing::error!("Failed to stop relay: {}", e);
                                    }
                                }
                            }
                            TaskType::UpdateIptables => {
                                if let Err(e) = iptables_manager.process_task(&task).await {
                                    tracing::error!("Failed to process iptables task {}: {}", task.id, e);
                                }
                            }
                            TaskType::ConfigureProxy => {
                                tracing::info!("Proxy configuration task received, task ID: {}", task.id);
                                // TODO: Implement proxy reconfiguration
                            }
                            TaskType::UpdateConfig => {
                                tracing::info!("Configuration update task received, task ID: {}", task.id);
                                // TODO: Implement config update
                            }
                        }
                    }
                }
            }
        })
    };

    tokio::select! {
        r = reg_handle => {
            tracing::error!("Registration manager stopped: {:?}", r);
        }
        r = proxy_handle => {
            tracing::error!("Proxy server stopped: {:?}", r);
        }
        r = task_handle => {
            tracing::error!("Task manager stopped: {:?}", r);
        }
        r = async {
            if let Some(h) = monitor_handle {
                h.await
            } else {
                futures_util::future::pending().await
            }
        } => {
            tracing::error!("Monitor stopped: {:?}", r);
        }
    }

    Ok(())
}

fn get_local_ip() -> Result<String> {
    use std::net::{IpAddr, UdpSocket};

    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    let addr = socket.local_addr()?;

    match addr.ip() {
        IpAddr::V4(ip) => Ok(ip.to_string()),
        IpAddr::V6(ip) => Ok(ip.to_string()),
    }
}

#[allow(dead_code)]
extern crate hostname;
#[allow(dead_code)]
extern crate uuid;