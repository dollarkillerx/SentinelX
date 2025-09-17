use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use sentinel_common::{ClientInfo, ClientStatus, SystemMetrics, Task, RelayConfig, TaskType, IptablesRule};
use std::sync::Arc;
use tokio::time::{interval, Duration};

use crate::db::Database;

pub struct ClientManager {
    clients: Arc<DashMap<String, ClientState>>,
    db: Database,
}

#[derive(Debug, Clone)]
pub struct ClientState {
    pub info: ClientInfo,
    pub status: ClientStatus,
    pub last_heartbeat: DateTime<Utc>,
    pub metrics: Option<SystemMetrics>,
    pub token: String,
}

impl ClientManager {
    pub fn new(db: Database) -> Self {
        Self {
            clients: Arc::new(DashMap::new()),
            db,
        }
    }

    pub async fn register_client(&self, info: ClientInfo) -> Result<String> {
        let client_id = info.id.clone();
        let token = self.generate_token();

        let state = ClientState {
            info: info.clone(),
            status: ClientStatus::Online,
            last_heartbeat: Utc::now(),
            metrics: None,
            token: token.clone(),
        };

        self.clients.insert(client_id.clone(), state);
        self.db.save_client(&info).await?;

        tracing::info!("Client registered: {}", client_id);
        Ok(token)
    }

    pub async fn update_heartbeat(&self, client_id: &str) -> Result<()> {
        if let Some(mut client) = self.clients.get_mut(client_id) {
            client.last_heartbeat = Utc::now();
            client.status = ClientStatus::Online;

            self.db.update_heartbeat(client_id).await?;
        } else {
            anyhow::bail!("Client not found: {}", client_id);
        }

        Ok(())
    }

    pub async fn update_metrics(&self, client_id: &str, metrics: SystemMetrics) -> Result<()> {
        if let Some(mut client) = self.clients.get_mut(client_id) {
            client.metrics = Some(metrics.clone());

            self.db.save_metrics(client_id, &metrics).await?;
        }

        Ok(())
    }

    pub async fn get_pending_tasks(&self, client_id: &str) -> Result<Vec<Task>> {
        self.db.get_pending_tasks(client_id).await
    }

    pub async fn list_clients(&self) -> Vec<ClientInfo> {
        self.clients
            .iter()
            .map(|entry| entry.info.clone())
            .collect()
    }

    pub async fn start_cleanup_task(&self) {
        let mut ticker = interval(Duration::from_secs(60));

        loop {
            ticker.tick().await;
            self.cleanup_inactive_clients().await;
        }
    }

    async fn cleanup_inactive_clients(&self) {
        let now = Utc::now();
        let timeout = chrono::Duration::seconds(120);

        let mut to_remove = vec![];

        for entry in self.clients.iter() {
            let client_id = entry.key().clone();
            let last_heartbeat = entry.last_heartbeat;

            if now.signed_duration_since(last_heartbeat) > timeout {
                to_remove.push(client_id);
            }
        }

        for client_id in to_remove {
            if let Some((_, mut client)) = self.clients.remove(&client_id) {
                client.status = ClientStatus::Offline;
                let _ = self.db.update_status(&client_id, "offline").await;
                tracing::info!("Client {} marked as offline", client_id);
            }
        }
    }

    pub async fn create_relay_task(&self, client_id: &str, relay_config: RelayConfig) -> Result<()> {
        let task = Task {
            id: uuid::Uuid::new_v4().to_string(),
            task_type: TaskType::StartRelay,
            payload: serde_json::to_value(relay_config)?,
            created_at: Utc::now(),
        };

        self.db.create_task(client_id, &task).await?;
        tracing::info!("Created relay task for client: {}", client_id);
        Ok(())
    }

    pub async fn create_stop_relay_task(&self, client_id: &str, relay_config: RelayConfig) -> Result<()> {
        let task = Task {
            id: uuid::Uuid::new_v4().to_string(),
            task_type: TaskType::StopRelay,
            payload: serde_json::to_value(relay_config)?,
            created_at: Utc::now(),
        };

        self.db.create_task(client_id, &task).await?;
        tracing::info!("Created stop relay task for client: {}", client_id);
        Ok(())
    }

    pub async fn create_iptables_task(&self, client_id: &str, rules: Vec<IptablesRule>) -> Result<()> {
        let rules_count = rules.len();
        let task = Task {
            id: uuid::Uuid::new_v4().to_string(),
            task_type: TaskType::UpdateIptables,
            payload: serde_json::to_value(rules)?,
            created_at: Utc::now(),
        };

        self.db.create_task(client_id, &task).await?;
        tracing::info!("Created iptables task for client: {} with {} rules", client_id, rules_count);
        Ok(())
    }

    fn generate_token(&self) -> String {
        uuid::Uuid::new_v4().to_string()
    }
}