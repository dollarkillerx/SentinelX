use anyhow::Result;
use chrono::{DateTime, Utc};
use sentinel_common::{ClientInfo, SystemMetrics, Task};
use sqlx::{postgres::PgPoolOptions, PgPool};

#[derive(Clone)]
pub struct Database {
    pool: PgPool,
}

impl Database {
    pub async fn connect(url: &str) -> Result<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(10)
            .connect(url)
            .await?;

        Ok(Self { pool })
    }

    pub async fn run_migrations(&self) -> Result<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn save_client(&self, info: &ClientInfo) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO clients (id, hostname, ip_address, version, capabilities, last_heartbeat, status)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (id) DO UPDATE SET
                hostname = EXCLUDED.hostname,
                ip_address = EXCLUDED.ip_address,
                version = EXCLUDED.version,
                capabilities = EXCLUDED.capabilities,
                last_heartbeat = EXCLUDED.last_heartbeat,
                updated_at = NOW()
            "#,
        )
        .bind(&info.id)
        .bind(&info.hostname)
        .bind(&info.ip)
        .bind(&info.version)
        .bind(serde_json::to_value(&info.capabilities)?)
        .bind(Utc::now())
        .bind("online")
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_heartbeat(&self, client_id: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE clients
            SET last_heartbeat = $1, status = 'online', updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(Utc::now())
        .bind(client_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_status(&self, client_id: &str, status: &str) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE clients
            SET status = $1, updated_at = NOW()
            WHERE id = $2
            "#,
        )
        .bind(status)
        .bind(client_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn save_metrics(&self, client_id: &str, metrics: &SystemMetrics) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE clients
            SET cpu_usage = $1, memory_usage = $2, disk_usage = $3,
                network_rx_rate = $4, network_tx_rate = $5, updated_at = NOW()
            WHERE id = $6
            "#,
        )
        .bind(metrics.cpu_usage)
        .bind(metrics.memory_usage)
        .bind(metrics.disk_usage)
        .bind(metrics.network_rx_rate as i64)
        .bind(metrics.network_tx_rate as i64)
        .bind(client_id)
        .execute(&self.pool)
        .await?;

        sqlx::query(
            r#"
            INSERT INTO client_metrics (
                client_id, cpu_usage, memory_used, memory_total,
                disk_used, disk_total, network_rx_bytes, network_tx_bytes,
                network_rx_rate, network_tx_rate
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
            "#,
        )
        .bind(client_id)
        .bind(metrics.cpu_usage)
        .bind(metrics.memory_used as i64)
        .bind(metrics.memory_total as i64)
        .bind(metrics.disk_used as i64)
        .bind(metrics.disk_total as i64)
        .bind(metrics.network_rx_bytes as i64)
        .bind(metrics.network_tx_bytes as i64)
        .bind(metrics.network_rx_rate as i64)
        .bind(metrics.network_tx_rate as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_pending_tasks(&self, client_id: &str) -> Result<Vec<Task>> {
        #[derive(sqlx::FromRow)]
        struct TaskRow {
            id: String,
            task_type: String,
            payload: serde_json::Value,
            created_at: DateTime<Utc>,
        }

        let rows = sqlx::query_as::<_, TaskRow>(
            r#"
            SELECT id, task_type, payload, created_at
            FROM client_tasks
            WHERE client_id = $1 AND status = 'pending'
            ORDER BY created_at ASC
            "#,
        )
        .bind(client_id)
        .fetch_all(&self.pool)
        .await?;

        let tasks = rows
            .into_iter()
            .filter_map(|row| {
                let task_type = match row.task_type.as_str() {
                    "start_relay" => sentinel_common::TaskType::StartRelay,
                    "stop_relay" => sentinel_common::TaskType::StopRelay,
                    "update_iptables" => sentinel_common::TaskType::UpdateIptables,
                    "configure_proxy" => sentinel_common::TaskType::ConfigureProxy,
                    "update_config" => sentinel_common::TaskType::UpdateConfig,
                    _ => return None,
                };

                Some(Task {
                    id: row.id,
                    task_type,
                    payload: row.payload,
                    created_at: row.created_at,
                })
            })
            .collect();

        Ok(tasks)
    }

    pub async fn create_task(&self, client_id: &str, task: &Task) -> Result<()> {
        let task_type_str = match task.task_type {
            sentinel_common::TaskType::StartRelay => "start_relay",
            sentinel_common::TaskType::StopRelay => "stop_relay",
            sentinel_common::TaskType::UpdateIptables => "update_iptables",
            sentinel_common::TaskType::ConfigureProxy => "configure_proxy",
            sentinel_common::TaskType::UpdateConfig => "update_config",
        };

        sqlx::query(
            r#"
            INSERT INTO client_tasks (id, client_id, task_type, payload, status, created_at)
            VALUES ($1, $2, $3, $4, 'pending', $5)
            "#,
        )
        .bind(&task.id)
        .bind(client_id)
        .bind(task_type_str)
        .bind(&task.payload)
        .bind(task.created_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_latest_metrics(&self, client_id: &str) -> Result<Option<SystemMetrics>> {
        #[derive(sqlx::FromRow)]
        struct MetricsRow {
            cpu_usage: Option<f32>,
            memory_usage: Option<f32>,
            disk_usage: Option<f32>,
            network_rx_rate: Option<i64>,
            network_tx_rate: Option<i64>,
        }

        let row = sqlx::query_as::<_, MetricsRow>(
            r#"
            SELECT cpu_usage, memory_usage, disk_usage, network_rx_rate, network_tx_rate
            FROM clients
            WHERE id = $1
            "#,
        )
        .bind(client_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(r) = row {
            if r.cpu_usage.is_some() {
                Ok(Some(SystemMetrics {
                    cpu_usage: r.cpu_usage.unwrap_or(0.0),
                    memory_used: 0,
                    memory_total: 0,
                    memory_usage: r.memory_usage.unwrap_or(0.0),
                    disk_used: 0,
                    disk_total: 0,
                    disk_usage: r.disk_usage.unwrap_or(0.0),
                    network_rx_bytes: 0,
                    network_tx_bytes: 0,
                    network_rx_rate: r.network_rx_rate.unwrap_or(0) as u64,
                    network_tx_rate: r.network_tx_rate.unwrap_or(0) as u64,
                    timestamp: Utc::now().timestamp(),
                }))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}