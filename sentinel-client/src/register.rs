use anyhow::Result;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use jsonrpsee::core::client::ClientT;
use sentinel_common::{
    ClientInfo, HeartbeatRequest, HeartbeatResponse, RegisterRequest, RegisterResponse, Task,
};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::interval;

use crate::monitor::SystemMonitor;

pub struct RegistrationManager {
    client_info: ClientInfo,
    server_url: String,
    heartbeat_interval: Duration,
    token: Arc<RwLock<Option<String>>>,
    client: HttpClient,
}

impl RegistrationManager {
    pub fn new(client_info: ClientInfo, server_url: String, heartbeat_interval: Duration) -> Self {
        let client = HttpClientBuilder::default()
            .build(&server_url)
            .expect("Failed to create HTTP client");

        Self {
            client_info,
            server_url,
            heartbeat_interval,
            token: Arc::new(RwLock::new(None)),
            client,
        }
    }

    pub async fn start(&self) -> Result<()> {
        self.register().await?;

        let mut ticker = interval(self.heartbeat_interval);
        loop {
            ticker.tick().await;
            if let Err(e) = self.send_heartbeat().await {
                tracing::error!("Heartbeat failed: {}", e);
                self.register().await?;
            }
        }
    }

    async fn register(&self) -> Result<()> {
        tracing::info!("Registering client with server...");

        let request = RegisterRequest {
            client_info: self.client_info.clone(),
        };

        let response: RegisterResponse = self
            .client
            .request("client.register", (request,))
            .await?;

        *self.token.write().await = Some(response.token.clone());

        tracing::info!("Client registered successfully: {}", response.client_id);
        Ok(())
    }

    async fn send_heartbeat(&self) -> Result<()> {
        let token = self.token.read().await.clone();
        if token.is_none() {
            return Err(anyhow::anyhow!("No token available"));
        }

        let metrics = SystemMonitor::collect_metrics().await.ok();

        let request = HeartbeatRequest {
            client_id: self.client_info.id.clone(),
            token: token.unwrap(),
            metrics,
        };

        let response: HeartbeatResponse = self
            .client
            .request("client.heartbeat", (request,))
            .await?;

        if !response.tasks.is_empty() {
            tracing::info!("Received {} tasks from server", response.tasks.len());
            for task in response.tasks {
                self.handle_task(task).await?;
            }
        }

        Ok(())
    }

    async fn handle_task(&self, task: sentinel_common::Task) -> Result<()> {
        tracing::info!("Processing task: {} (type: {:?})", task.id, task.task_type);

        match task.task_type {
            sentinel_common::TaskType::UpdateIptables => {
                tracing::info!("Updating iptables rules...");
            }
            sentinel_common::TaskType::ConfigureProxy => {
                tracing::info!("Configuring proxy...");
            }
            sentinel_common::TaskType::StartRelay => {
                tracing::info!("Starting relay...");
            }
            sentinel_common::TaskType::StopRelay => {
                tracing::info!("Stopping relay...");
            }
            sentinel_common::TaskType::UpdateConfig => {
                tracing::info!("Updating configuration...");
            }
        }

        Ok(())
    }

    pub async fn get_pending_tasks(&self) -> Result<Vec<Task>> {
        let token = self.token.read().await.clone();
        if token.is_none() {
            return Ok(vec![]);
        }

        let request = HeartbeatRequest {
            client_id: self.client_info.id.clone(),
            token: token.unwrap(),
            metrics: None,
        };

        let response: HeartbeatResponse = self
            .client
            .request("client.heartbeat", (request,))
            .await?;

        Ok(response.tasks)
    }
}