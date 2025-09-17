# SentinelX - Rust 实现文档

## 项目结构

```
SentinelX/
├── sentinel-client/          # Client 端实现
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── config.rs        # 配置管理
│   │   ├── register.rs      # 注册与心跳
│   │   ├── iptables.rs      # iptables 管理
│   │   ├── proxy.rs         # 端口转发核心
│   │   ├── relay.rs         # 流量中继
│   │   ├── transport/       # 传输层
│   │   │   ├── mod.rs
│   │   │   ├── tcp.rs       # TCP 直连
│   │   │   ├── encrypted.rs # 加密传输
│   │   │   └── websocket.rs # WebSocket 封装
│   │   ├── limiter.rs       # 限速实现
│   │   ├── stats.rs         # 流量统计
│   │   └── monitor.rs       # 系统监控
│   └── config.toml          # 默认配置
│
├── sentinel-server/          # Server 端实现
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs
│   │   ├── api/             # JSON-RPC API
│   │   │   ├── mod.rs
│   │   │   ├── client.rs    # Client 管理
│   │   │   ├── task.rs      # 任务下发
│   │   │   └── stats.rs     # 统计接口
│   │   ├── db/              # 数据库层
│   │   │   ├── mod.rs
│   │   │   ├── models.rs    # 数据模型
│   │   │   └── migrations/  # 数据库迁移
│   │   ├── manager.rs       # Client 管理器
│   │   └── scheduler.rs     # 任务调度
│   └── config.toml
│
├── sentinel-common/          # 共享库
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs
│       ├── protocol.rs      # 通信协议定义
│       ├── crypto.rs        # 加密工具
│       └── types.rs         # 共享类型
│
├── sentinel-web/             # Web UI
│   ├── package.json
│   ├── src/
│   └── ...
│
└── Cargo.toml               # Workspace 配置
```

## 核心依赖

### Workspace Cargo.toml

```toml
[workspace]
members = [
    "sentinel-client",
    "sentinel-server",
    "sentinel-common",
]
resolver = "2"

[workspace.dependencies]
tokio = { version = "1.42", features = ["full"] }
axum = "0.8"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tracing = "0.1"
tracing-subscriber = "0.3"
anyhow = "1.0"
thiserror = "2.0"
config = "0.14"
```

### sentinel-client/Cargo.toml

```toml
[package]
name = "sentinel-client"
version = "0.1.0"
edition = "2021"

[dependencies]
sentinel-common = { path = "../sentinel-common" }
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
tracing = { workspace = true }
anyhow = { workspace = true }

# 网络相关
tokio-tungstenite = "0.24"
hyper = "1.5"
tower = "0.5"
futures-util = "0.3"

# 系统操作
nix = "0.29"
libc = "0.2"
sysinfo = "0.33"  # 系统信息监控
chrono = "0.4"  # 时间处理

# 限速与统计
governor = "0.8"
metrics = "0.24"
prometheus = "0.13"

# 配置
config = { workspace = true }
clap = { version = "4.5", features = ["derive"] }
```

### sentinel-server/Cargo.toml

```toml
[package]
name = "sentinel-server"
version = "0.1.0"
edition = "2021"

[dependencies]
sentinel-common = { path = "../sentinel-common" }
tokio = { workspace = true }
axum = { workspace = true }
serde = { workspace = true }
tracing = { workspace = true }

# 数据库
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres", "migrate"] }
sea-orm = { version = "1.1", features = ["sqlx-postgres", "runtime-tokio"] }

# JSON-RPC
jsonrpsee = { version = "0.24", features = ["server", "macros"] }
tower-http = { version = "0.6", features = ["cors", "trace"] }

# 调度与缓存
tokio-cron-scheduler = "0.13"
dashmap = "6.1"
moka = "0.12"
```

## 核心模块实现

### 1. Client 主程序 (sentinel-client/src/main.rs)

```rust
use anyhow::Result;
use clap::Parser;
use sentinel_client::{
    config::Config,
    register::RegistrationManager,
    proxy::ProxyServer,
    monitor::{MetricsReporter, get_system_info},
};
use std::sync::Arc;
use tracing_subscriber;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    /// 配置文件路径
    #[clap(short, long, default_value = "/etc/sentinel/client.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // 初始化日志
    tracing_subscriber::fmt()
        .with_env_filter("sentinel=debug")
        .init();

    let args = Args::parse();
    let config = Config::from_file(&args.config)?;

    tracing::info!("Starting Sentinel Client v{}", env!("CARGO_PKG_VERSION"));

    // 收集系统信息
    let system_info = get_system_info();
    tracing::info!("System: {} {}, CPU: {} cores, Memory: {} GB",
        system_info.os,
        system_info.kernel_version,
        system_info.cpu_cores,
        system_info.total_memory / (1024 * 1024 * 1024)
    );

    // 创建客户端信息
    let client_info = ClientInfo {
        id: config.client.id.clone(),
        hostname: config.client.hostname.clone(),
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

    // 启动注册管理器
    let registration = Arc::new(RegistrationManager::new(
        client_info,
        config.server.url.clone(),
        config.server.heartbeat_interval,
    ));

    let reg_handle = {
        let registration = registration.clone();
        tokio::spawn(async move {
            registration.start().await
        })
    };

    // 启动系统监控（如果启用）
    let monitor_handle = if config.monitoring.enabled {
        let reporter = MetricsReporter::new(
            config.server.url.clone(),
            std::time::Duration::from_secs(config.monitoring.report_interval),
        );

        Some(tokio::spawn(async move {
            reporter.start().await
        }))
    } else {
        None
    };

    // 启动代理服务器
    let proxy = ProxyServer::new(
        config.proxy.listen_addr,
        config.proxy.target_addr,
    );

    let proxy_handle = tokio::spawn(async move {
        proxy.start().await
    });

    // 等待所有任务
    tokio::select! {
        r = reg_handle => {
            tracing::error!("Registration manager stopped: {:?}", r);
        }
        r = proxy_handle => {
            tracing::error!("Proxy server stopped: {:?}", r);
        }
        r = async {
            if let Some(h) = monitor_handle {
                h.await
            } else {
                futures::future::pending().await
            }
        } => {
            tracing::error!("Monitor stopped: {:?}", r);
        }
    }

    Ok(())
}

fn get_local_ip() -> Result<String> {
    // 获取本地IP地址的实现
    Ok("127.0.0.1".to_string())
}
```

### 2. Client 注册与心跳 (sentinel-client/src/register.rs)

```rust
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::time::interval;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub id: String,
    pub hostname: String,
    pub ip: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub system_info: SystemInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub kernel_version: String,
    pub cpu_cores: usize,
    pub total_memory: u64,  // bytes
    pub total_disk: u64,    // bytes
}

pub struct RegistrationManager {
    client_info: ClientInfo,
    server_url: String,
    heartbeat_interval: Duration,
}

impl RegistrationManager {
    pub async fn start(&self) -> Result<()> {
        // 初始注册
        self.register().await?;

        // 启动心跳
        let mut ticker = interval(self.heartbeat_interval);
        loop {
            ticker.tick().await;
            if let Err(e) = self.send_heartbeat().await {
                tracing::error!("Heartbeat failed: {}", e);
                // 重新注册
                self.register().await?;
            }
        }
    }

    async fn register(&self) -> Result<()> {
        // JSON-RPC 注册调用
        todo!()
    }

    async fn send_heartbeat(&self) -> Result<()> {
        // 获取系统监控数据
        let metrics = SystemMonitor::collect_metrics().await?;

        // 发送心跳包（包含监控数据）
        let heartbeat = HeartbeatRequest {
            client_id: self.client_info.id.clone(),
            metrics,
        };

        // JSON-RPC 调用
        todo!()
    }
}
```

### 2. iptables 管理 (sentinel-client/src/iptables.rs)

```rust
use anyhow::Result;
use std::process::Command;

pub struct IptablesManager;

impl IptablesManager {
    pub fn apply_rule(&self, rule: &IptablesRule) -> Result<()> {
        let mut cmd = Command::new("iptables");

        match &rule.action {
            Action::Insert => cmd.arg("-I"),
            Action::Append => cmd.arg("-A"),
            Action::Delete => cmd.arg("-D"),
        };

        cmd.arg(&rule.chain);

        if let Some(protocol) = &rule.protocol {
            cmd.args(["-p", protocol]);
        }

        if let Some(source) = &rule.source {
            cmd.args(["-s", source]);
        }

        if let Some(destination) = &rule.destination {
            cmd.args(["-d", destination]);
        }

        if let Some(dport) = &rule.dport {
            cmd.args(["--dport", &dport.to_string()]);
        }

        cmd.args(["-j", &rule.target]);

        let output = cmd.output()?;
        if !output.status.success() {
            anyhow::bail!("iptables command failed: {}",
                String::from_utf8_lossy(&output.stderr));
        }

        Ok(())
    }

    pub fn list_rules(&self, chain: &str) -> Result<Vec<String>> {
        let output = Command::new("iptables")
            .args(["-L", chain, "-n", "--line-numbers"])
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(String::from)
            .collect())
    }
}

#[derive(Debug, Clone)]
pub struct IptablesRule {
    pub action: Action,
    pub chain: String,
    pub protocol: Option<String>,
    pub source: Option<String>,
    pub destination: Option<String>,
    pub dport: Option<u16>,
    pub target: String,
}

#[derive(Debug, Clone)]
pub enum Action {
    Insert,
    Append,
    Delete,
}
```

### 3. 端口转发核心 (sentinel-client/src/proxy.rs)

```rust
use anyhow::Result;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::sync::Arc;
use std::net::SocketAddr;

pub struct ProxyServer {
    listen_addr: SocketAddr,
    target_addr: SocketAddr,
    stats: Arc<StatsCollector>,
    limiter: Option<Arc<RateLimiter>>,
}

impl ProxyServer {
    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(self.listen_addr).await?;
        tracing::info!("Proxy listening on {}", self.listen_addr);

        loop {
            let (inbound, peer_addr) = listener.accept().await?;
            let target = self.target_addr;
            let stats = self.stats.clone();
            let limiter = self.limiter.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(
                    inbound,
                    target,
                    stats,
                    limiter
                ).await {
                    tracing::error!("Connection error from {}: {}", peer_addr, e);
                }
            });
        }
    }

    async fn handle_connection(
        mut inbound: TcpStream,
        target: SocketAddr,
        stats: Arc<StatsCollector>,
        limiter: Option<Arc<RateLimiter>>,
    ) -> Result<()> {
        let mut outbound = TcpStream::connect(target).await?;

        let (mut ri, mut wi) = inbound.split();
        let (mut ro, mut wo) = outbound.split();

        let stats1 = stats.clone();
        let stats2 = stats.clone();
        let limiter1 = limiter.clone();
        let limiter2 = limiter.clone();

        let client_to_server = tokio::spawn(async move {
            let mut buf = vec![0u8; 8192];
            loop {
                let n = ri.read(&mut buf).await?;
                if n == 0 { break; }

                // 限速检查
                if let Some(limiter) = &limiter1 {
                    limiter.wait_for_capacity(n).await;
                }

                wo.write_all(&buf[..n]).await?;
                stats1.add_bytes_sent(n);
            }
            Ok::<_, anyhow::Error>(())
        });

        let server_to_client = tokio::spawn(async move {
            let mut buf = vec![0u8; 8192];
            loop {
                let n = ro.read(&mut buf).await?;
                if n == 0 { break; }

                // 限速检查
                if let Some(limiter) = &limiter2 {
                    limiter.wait_for_capacity(n).await;
                }

                wi.write_all(&buf[..n]).await?;
                stats2.add_bytes_received(n);
            }
            Ok::<_, anyhow::Error>(())
        });

        tokio::select! {
            r1 = client_to_server => r1??,
            r2 = server_to_client => r2??,
        }

        Ok(())
    }
}
```

### 4. 流量中继 (sentinel-client/src/relay.rs)

```rust
use anyhow::Result;
use tokio::net::TcpStream;
use crate::transport::{Transport, TransportType};

pub struct RelayManager {
    transport_type: TransportType,
}

impl RelayManager {
    pub async fn create_relay(
        &self,
        entry_point: String,
        exit_point: String,
    ) -> Result<RelayConnection> {
        let transport = match self.transport_type {
            TransportType::Direct => Box::new(DirectTransport::new()),
            TransportType::Encrypted => Box::new(EncryptedTransport::new()),
            TransportType::WebSocket => Box::new(WebSocketTransport::new()),
        };

        Ok(RelayConnection {
            entry: entry_point,
            exit: exit_point,
            transport,
        })
    }
}

pub struct RelayConnection {
    entry: String,
    exit: String,
    transport: Box<dyn Transport>,
}

impl RelayConnection {
    pub async fn start(&self) -> Result<()> {
        // 连接入口点
        let entry_stream = TcpStream::connect(&self.entry).await?;

        // 通过传输层连接出口点
        let exit_stream = self.transport.connect(&self.exit).await?;

        // 开始数据中继
        self.transport.relay(entry_stream, exit_stream).await?;

        Ok(())
    }
}
```

### 5. Server JSON-RPC API (sentinel-server/src/api/client.rs)

```rust
use jsonrpsee::{core::RpcResult, proc_macros::rpc};
use serde::{Deserialize, Serialize};

#[rpc(server)]
pub trait ClientApi {
    #[method(name = "client.register")]
    async fn register(&self, info: ClientInfo) -> RpcResult<RegisterResponse>;

    #[method(name = "client.heartbeat")]
    async fn heartbeat(&self, client_id: String, metrics: Option<SystemMetrics>) -> RpcResult<HeartbeatResponse>;

    #[method(name = "client.list")]
    async fn list_clients(&self) -> RpcResult<Vec<ClientStatus>>;

    #[method(name = "task.dispatch")]
    async fn dispatch_task(&self, task: Task) -> RpcResult<TaskResponse>;

    #[method(name = "metrics.get_latest")]
    async fn get_latest_metrics(&self, client_id: String) -> RpcResult<SystemMetrics>;

    #[method(name = "metrics.get_history")]
    async fn get_metrics_history(&self, client_id: String, hours: u32) -> RpcResult<Vec<SystemMetrics>>;

    #[method(name = "metrics.get_summary")]
    async fn get_metrics_summary(&self) -> RpcResult<MetricsSummary>;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetricsSummary {
    pub total_clients: u32,
    pub online_clients: u32,
    pub total_cpu_usage: f32,
    pub total_memory_usage: f32,
    pub total_bandwidth_rx: u64,  // bytes/s
    pub total_bandwidth_tx: u64,  // bytes/s
}

pub struct ClientApiImpl {
    manager: Arc<ClientManager>,
}

#[async_trait]
impl ClientApiServer for ClientApiImpl {
    async fn register(&self, info: ClientInfo) -> RpcResult<RegisterResponse> {
        let client_id = self.manager.register_client(info).await
            .map_err(|e| jsonrpsee::core::Error::Custom(e.to_string()))?;

        Ok(RegisterResponse {
            client_id,
            token: generate_token(),
        })
    }

    async fn heartbeat(&self, client_id: String, metrics: Option<SystemMetrics>) -> RpcResult<HeartbeatResponse> {
        // 更新心跳时间
        self.manager.update_heartbeat(&client_id).await
            .map_err(|e| jsonrpsee::core::Error::Custom(e.to_string()))?;

        // 如果包含监控数据，存储到数据库
        if let Some(m) = metrics {
            self.manager.update_metrics(&client_id, m).await
                .map_err(|e| jsonrpsee::core::Error::Custom(e.to_string()))?;
        }

        // 获取待处理任务
        let pending_tasks = self.manager.get_pending_tasks(&client_id).await?;

        Ok(HeartbeatResponse {
            status: "ok".to_string(),
            tasks: pending_tasks,
        })
    }
}
```

### 6. 限速实现 (sentinel-client/src/limiter.rs)

```rust
use governor::{Quota, RateLimiter as GovernorRateLimiter};
use std::num::NonZeroU32;
use std::sync::Arc;

pub struct RateLimiter {
    limiter: Arc<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl RateLimiter {
    pub fn new(bytes_per_second: u32) -> Self {
        let quota = Quota::per_second(NonZeroU32::new(bytes_per_second).unwrap());
        let limiter = Arc::new(GovernorRateLimiter::direct(quota));

        Self { limiter }
    }

    pub async fn wait_for_capacity(&self, bytes: usize) {
        let permits = (bytes / 1024).max(1) as u32; // 以 KB 为单位
        for _ in 0..permits {
            self.limiter.until_ready().await;
        }
    }

    pub fn try_consume(&self, bytes: usize) -> bool {
        let permits = (bytes / 1024).max(1) as u32;
        self.limiter.check_n(NonZeroU32::new(permits).unwrap()).is_ok()
    }
}
```

### 7. 流量统计 (sentinel-client/src/stats.rs)

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::HashMap;
use parking_lot::RwLock;

pub struct StatsCollector {
    bytes_sent: AtomicU64,
    bytes_received: AtomicU64,
    connections: AtomicU64,
    connection_details: RwLock<HashMap<String, ConnectionStats>>,
}

impl StatsCollector {
    pub fn new() -> Self {
        Self {
            bytes_sent: AtomicU64::new(0),
            bytes_received: AtomicU64::new(0),
            connections: AtomicU64::new(0),
            connection_details: RwLock::new(HashMap::new()),
        }
    }

    pub fn add_bytes_sent(&self, bytes: usize) {
        self.bytes_sent.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub fn add_bytes_received(&self, bytes: usize) {
        self.bytes_received.fetch_add(bytes as u64, Ordering::Relaxed);
    }

    pub fn new_connection(&self, peer: String) {
        self.connections.fetch_add(1, Ordering::Relaxed);

        let mut details = self.connection_details.write();
        details.insert(peer, ConnectionStats::new());
    }

    pub fn get_stats(&self) -> Stats {
        Stats {
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
            total_connections: self.connections.load(Ordering::Relaxed),
            active_connections: self.connection_details.read().len() as u64,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Stats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub total_connections: u64,
    pub active_connections: u64,
}
```

### 8. 系统监控 (sentinel-client/src/monitor.rs)

```rust
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use sysinfo::{CpuExt, DiskExt, NetworkExt, System, SystemExt};
use tokio::sync::RwLock;
use std::sync::Arc;
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f32,           // 百分比 (0-100)
    pub memory_used: u64,          // bytes
    pub memory_total: u64,         // bytes
    pub memory_usage: f32,         // 百分比 (0-100)
    pub disk_used: u64,            // bytes
    pub disk_total: u64,           // bytes
    pub disk_usage: f32,           // 百分比 (0-100)
    pub network_rx_bytes: u64,     // 下行总字节数
    pub network_tx_bytes: u64,     // 上行总字节数
    pub network_rx_rate: u64,      // 下行速率 bytes/s
    pub network_tx_rate: u64,      // 上行速率 bytes/s
    pub timestamp: i64,            // Unix timestamp
}

pub struct SystemMonitor {
    system: Arc<RwLock<System>>,
    network_history: Arc<RwLock<NetworkHistory>>,
}

struct NetworkHistory {
    samples: VecDeque<NetworkSample>,
    max_samples: usize,
}

#[derive(Clone)]
struct NetworkSample {
    rx_bytes: u64,
    tx_bytes: u64,
    timestamp: Instant,
}

impl SystemMonitor {
    pub fn new() -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            system: Arc::new(RwLock::new(system)),
            network_history: Arc::new(RwLock::new(NetworkHistory {
                samples: VecDeque::with_capacity(60),
                max_samples: 60,  // 保留60秒的历史数据
            })),
        }
    }

    pub async fn start_monitoring(self: Arc<Self>) {
        let mut interval = tokio::time::interval(Duration::from_secs(1));

        loop {
            interval.tick().await;
            if let Err(e) = self.update_metrics().await {
                tracing::error!("Failed to update metrics: {}", e);
            }
        }
    }

    async fn update_metrics(&self) -> Result<()> {
        let mut system = self.system.write().await;

        // 刷新系统信息
        system.refresh_cpu();
        system.refresh_memory();
        system.refresh_disks();
        system.refresh_networks();

        // 更新网络历史
        let (rx_bytes, tx_bytes) = self.get_network_bytes(&system);
        let mut history = self.network_history.write().await;

        history.samples.push_back(NetworkSample {
            rx_bytes,
            tx_bytes,
            timestamp: Instant::now(),
        });

        if history.samples.len() > history.max_samples {
            history.samples.pop_front();
        }

        Ok(())
    }

    pub async fn collect_metrics() -> Result<SystemMetrics> {
        let monitor = Self::new();
        let mut system = monitor.system.write().await;

        // 刷新所有系统信息
        system.refresh_all();

        // 等待一秒后再次刷新CPU（获得准确的CPU使用率）
        tokio::time::sleep(Duration::from_secs(1)).await;
        system.refresh_cpu();

        let cpu_usage = system.global_cpu_info().cpu_usage();
        let memory_used = system.used_memory();
        let memory_total = system.total_memory();
        let memory_usage = (memory_used as f32 / memory_total as f32) * 100.0;

        // 计算磁盘使用
        let (disk_used, disk_total) = system.disks()
            .iter()
            .map(|disk| (disk.total_space() - disk.available_space(), disk.total_space()))
            .fold((0u64, 0u64), |(used, total), (u, t)| (used + u, total + t));

        let disk_usage = if disk_total > 0 {
            (disk_used as f32 / disk_total as f32) * 100.0
        } else {
            0.0
        };

        // 获取网络统计
        let (rx_bytes, tx_bytes) = monitor.get_network_bytes(&system);
        let (rx_rate, tx_rate) = monitor.calculate_network_rates().await;

        Ok(SystemMetrics {
            cpu_usage,
            memory_used,
            memory_total,
            memory_usage,
            disk_used,
            disk_total,
            disk_usage,
            network_rx_bytes: rx_bytes,
            network_tx_bytes: tx_bytes,
            network_rx_rate: rx_rate,
            network_tx_rate: tx_rate,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }

    fn get_network_bytes(&self, system: &System) -> (u64, u64) {
        system.networks()
            .iter()
            .filter(|(name, _)| !name.starts_with("lo")) // 排除回环接口
            .map(|(_, data)| (data.total_received(), data.total_transmitted()))
            .fold((0u64, 0u64), |(rx, tx), (r, t)| (rx + r, tx + t))
    }

    async fn calculate_network_rates(&self) -> (u64, u64) {
        let history = self.network_history.read().await;

        if history.samples.len() < 2 {
            return (0, 0);
        }

        let recent = &history.samples[history.samples.len() - 1];
        let previous = &history.samples[history.samples.len() - 2];

        let duration = recent.timestamp.duration_since(previous.timestamp);
        if duration.as_secs() == 0 {
            return (0, 0);
        }

        let rx_rate = (recent.rx_bytes.saturating_sub(previous.rx_bytes)) / duration.as_secs();
        let tx_rate = (recent.tx_bytes.saturating_sub(previous.tx_bytes)) / duration.as_secs();

        (rx_rate, tx_rate)
    }
}

// 周期性报告任务
pub struct MetricsReporter {
    monitor: Arc<SystemMonitor>,
    server_url: String,
    interval: Duration,
}

impl MetricsReporter {
    pub fn new(server_url: String, interval: Duration) -> Self {
        Self {
            monitor: Arc::new(SystemMonitor::new()),
            server_url,
            interval,
        }
    }

    pub async fn start(&self) -> Result<()> {
        // 启动监控线程
        let monitor = self.monitor.clone();
        tokio::spawn(async move {
            monitor.start_monitoring().await;
        });

        // 定期报告
        let mut ticker = tokio::time::interval(self.interval);

        loop {
            ticker.tick().await;

            match SystemMonitor::collect_metrics().await {
                Ok(metrics) => {
                    if let Err(e) = self.report_metrics(metrics).await {
                        tracing::error!("Failed to report metrics: {}", e);
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to collect metrics: {}", e);
                }
            }
        }
    }

    async fn report_metrics(&self, metrics: SystemMetrics) -> Result<()> {
        // 发送到服务器
        // TODO: 实现 JSON-RPC 调用
        tracing::debug!("Reporting metrics: {:?}", metrics);
        Ok(())
    }
}

// 用于获取系统基本信息
pub fn get_system_info() -> SystemInfo {
    let mut system = System::new_all();
    system.refresh_all();

    SystemInfo {
        os: System::name().unwrap_or_else(|| "Unknown".to_string()),
        kernel_version: System::kernel_version().unwrap_or_else(|| "Unknown".to_string()),
        cpu_cores: system.cpus().len(),
        total_memory: system.total_memory(),
        total_disk: system.disks()
            .iter()
            .map(|disk| disk.total_space())
            .sum(),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os: String,
    pub kernel_version: String,
    pub cpu_cores: usize,
    pub total_memory: u64,
    pub total_disk: u64,
}
```

### 9. 数据库模型 (sentinel-server/src/db/models.rs)

```rust
use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "clients")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: String,
    pub hostname: String,
    pub ip_address: String,
    pub version: String,
    pub capabilities: Json,
    pub last_heartbeat: DateTime,
    pub status: ClientStatus,
    // 系统监控数据
    pub cpu_usage: Option<f32>,
    pub memory_usage: Option<f32>,
    pub disk_usage: Option<f32>,
    pub network_rx_rate: Option<i64>,  // bytes/s
    pub network_tx_rate: Option<i64>,  // bytes/s
    pub created_at: DateTime,
    pub updated_at: DateTime,
}

#[derive(Debug, Clone, PartialEq, EnumIter, DeriveActiveEnum, Serialize, Deserialize)]
#[sea_orm(rs_type = "String", db_type = "String(Some(20))")]
pub enum ClientStatus {
    #[sea_orm(string_value = "online")]
    Online,
    #[sea_orm(string_value = "offline")]
    Offline,
    #[sea_orm(string_value = "error")]
    Error,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::task::Entity")]
    Tasks,
    #[sea_orm(has_many = "super::rule::Entity")]
    Rules,
}

impl ActiveModelBehavior for ActiveModel {}

// 系统监控历史数据表
#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Serialize, Deserialize)]
#[sea_orm(table_name = "client_metrics")]
pub struct MetricsModel {
    #[sea_orm(primary_key)]
    pub id: i64,
    pub client_id: String,
    pub cpu_usage: f32,
    pub memory_used: i64,
    pub memory_total: i64,
    pub disk_used: i64,
    pub disk_total: i64,
    pub network_rx_bytes: i64,
    pub network_tx_bytes: i64,
    pub network_rx_rate: i64,
    pub network_tx_rate: i64,
    pub recorded_at: DateTime,
}
```

## 启动配置

### Client 配置 (config.toml)

```toml
[client]
id = "" # 自动生成或手动指定
hostname = "" # 自动获取

[server]
url = "http://localhost:8080"
token = ""
heartbeat_interval = 30 # 秒

[proxy]
listen_addr = "0.0.0.0:0" # 0 表示随机端口
target_addr = "127.0.0.1:8080"
buffer_size = 8192

[transport]
type = "direct" # direct | encrypted | websocket
encryption_key = ""

[limits]
max_connections = 1000
rate_limit_mbps = 100 # 0 表示不限制

[monitoring]
enabled = true
report_interval = 30 # 秒，系统监控上报间隔
collect_interval = 1 # 秒，本地采集间隔

[logging]
level = "info"
file = "/var/log/sentinel-client.log"
```

### Server 配置 (config.toml)

```toml
[server]
bind_addr = "0.0.0.0:8080"
workers = 4

[database]
url = "postgres://user:pass@localhost/sentinel"
max_connections = 10
min_connections = 1

[security]
jwt_secret = "your-secret-key"
token_expiry = 3600 # 秒

[client_management]
heartbeat_timeout = 120 # 秒
cleanup_interval = 60 # 秒

[api]
rate_limit = 100 # 每分钟请求数
max_request_size = "10MB"

[logging]
level = "info"
file = "/var/log/sentinel-server.log"
```

## 数据库迁移

### PostgreSQL 数据库初始化

```sql
-- migrations/001_create_clients_table.sql
CREATE TABLE IF NOT EXISTS clients (
    id VARCHAR(255) PRIMARY KEY,
    hostname VARCHAR(255) NOT NULL,
    ip_address VARCHAR(45) NOT NULL,
    version VARCHAR(50),
    capabilities JSONB,
    last_heartbeat TIMESTAMP WITH TIME ZONE,
    status VARCHAR(20) DEFAULT 'offline',
    -- 最新监控数据
    cpu_usage REAL,
    memory_usage REAL,
    disk_usage REAL,
    network_rx_rate BIGINT,
    network_tx_rate BIGINT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_clients_status ON clients(status);
CREATE INDEX idx_clients_last_heartbeat ON clients(last_heartbeat);

-- migrations/002_create_metrics_history.sql
CREATE TABLE IF NOT EXISTS client_metrics (
    id BIGSERIAL PRIMARY KEY,
    client_id VARCHAR(255) REFERENCES clients(id) ON DELETE CASCADE,
    cpu_usage REAL NOT NULL,
    memory_used BIGINT NOT NULL,
    memory_total BIGINT NOT NULL,
    disk_used BIGINT NOT NULL,
    disk_total BIGINT NOT NULL,
    network_rx_bytes BIGINT NOT NULL,
    network_tx_bytes BIGINT NOT NULL,
    network_rx_rate BIGINT NOT NULL,
    network_tx_rate BIGINT NOT NULL,
    recorded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_metrics_client_id ON client_metrics(client_id);
CREATE INDEX idx_metrics_recorded_at ON client_metrics(recorded_at);

-- 创建分区表（按月分区，用于历史数据）
CREATE TABLE client_metrics_partitioned (
    LIKE client_metrics INCLUDING ALL
) PARTITION BY RANGE (recorded_at);

-- 创建自动清理历史数据的函数
CREATE OR REPLACE FUNCTION cleanup_old_metrics()
RETURNS void AS $$
BEGIN
    DELETE FROM client_metrics
    WHERE recorded_at < NOW() - INTERVAL '30 days';
END;
$$ LANGUAGE plpgsql;

-- 创建定时任务（需要pg_cron扩展）
-- CREATE EXTENSION IF NOT EXISTS pg_cron;
-- SELECT cron.schedule('cleanup-metrics', '0 2 * * *', 'SELECT cleanup_old_metrics();');
```

## 构建与部署

### 开发环境

```bash
# 克隆项目
git clone https://github.com/your-org/sentinelx.git
cd sentinelx

# 安装 Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 构建所有组件
cargo build --workspace

# 运行测试
cargo test --workspace

# 格式化代码
cargo fmt --all

# 代码检查
cargo clippy --all-targets --all-features
```

### 生产构建

```bash
# 优化构建
cargo build --release --workspace

# 构建 Docker 镜像
docker build -t sentinelx-server:latest -f docker/server.Dockerfile .
docker build -t sentinelx-client:latest -f docker/client.Dockerfile .
```

### Docker 部署

#### Server Dockerfile

```dockerfile
# docker/server.Dockerfile
FROM rust:1.83 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin sentinel-server

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/sentinel-server /usr/local/bin/
COPY sentinel-server/config.toml /etc/sentinel/config.toml

EXPOSE 8080
CMD ["sentinel-server", "--config", "/etc/sentinel/config.toml"]
```

#### Client Dockerfile

```dockerfile
# docker/client.Dockerfile
FROM rust:1.83 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin sentinel-client

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    iptables \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/sentinel-client /usr/local/bin/
COPY sentinel-client/config.toml /etc/sentinel/config.toml

CMD ["sentinel-client", "--config", "/etc/sentinel/config.toml"]
```

### Docker Compose

```yaml
# docker-compose.yml
version: '3.8'

services:
  postgres:
    image: postgres:16
    environment:
      POSTGRES_DB: sentinel
      POSTGRES_USER: sentinel
      POSTGRES_PASSWORD: sentinel123
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

  server:
    build:
      context: .
      dockerfile: docker/server.Dockerfile
    depends_on:
      - postgres
    environment:
      DATABASE_URL: postgres://sentinel:sentinel123@postgres/sentinel
    ports:
      - "8080:8080"
    volumes:
      - ./config/server.toml:/etc/sentinel/config.toml

  client:
    build:
      context: .
      dockerfile: docker/client.Dockerfile
    depends_on:
      - server
    environment:
      SERVER_URL: http://server:8080
    cap_add:
      - NET_ADMIN
      - NET_RAW
    volumes:
      - ./config/client.toml:/etc/sentinel/config.toml

volumes:
  postgres_data:
```

## 性能优化

### 1. 零拷贝优化

```rust
// 使用 splice 系统调用实现零拷贝
#[cfg(target_os = "linux")]
pub async fn zero_copy_relay(
    source: &mut TcpStream,
    dest: &mut TcpStream,
) -> Result<u64> {
    use nix::fcntl::{splice, SpliceFFlags};
    use std::os::unix::io::AsRawFd;

    let mut total = 0u64;
    let mut pipe = Pipe::new()?;

    loop {
        // 从源读取到管道
        let n = splice(
            source.as_raw_fd(),
            None,
            pipe.write_fd(),
            None,
            65536,
            SpliceFFlags::SPLICE_F_MOVE | SpliceFFlags::SPLICE_F_NONBLOCK,
        )?;

        if n == 0 {
            break;
        }

        // 从管道写入到目标
        splice(
            pipe.read_fd(),
            None,
            dest.as_raw_fd(),
            None,
            n,
            SpliceFFlags::SPLICE_F_MOVE | SpliceFFlags::SPLICE_F_NONBLOCK,
        )?;

        total += n as u64;
    }

    Ok(total)
}
```

### 2. 连接池优化

```rust
use deadpool::managed::{Manager, Pool};

pub struct ConnectionPool {
    pool: Pool<ConnectionManager>,
}

impl ConnectionPool {
    pub async fn get(&self) -> Result<Connection> {
        Ok(self.pool.get().await?)
    }
}

struct ConnectionManager {
    target: SocketAddr,
}

#[async_trait]
impl Manager for ConnectionManager {
    type Type = TcpStream;
    type Error = anyhow::Error;

    async fn create(&self) -> Result<Self::Type, Self::Error> {
        Ok(TcpStream::connect(self.target).await?)
    }

    async fn recycle(&self, conn: &mut Self::Type) -> Result<(), Self::Error> {
        // 检查连接是否健康
        conn.peer_addr()?;
        Ok(())
    }
}
```

### 3. 批量处理优化

```rust
use futures::stream::{self, StreamExt};

pub async fn batch_process_tasks(tasks: Vec<Task>) -> Vec<Result<TaskResult>> {
    stream::iter(tasks)
        .map(|task| async move {
            process_task(task).await
        })
        .buffer_unordered(10) // 并发处理10个任务
        .collect()
        .await
}
```

## 监控与告警

### Prometheus 指标

```rust
use prometheus::{Counter, Gauge, Histogram, register_counter, register_gauge, register_histogram};

lazy_static! {
    static ref BYTES_TRANSFERRED: Counter = register_counter!(
        "sentinel_bytes_transferred_total",
        "Total bytes transferred"
    ).unwrap();

    static ref ACTIVE_CONNECTIONS: Gauge = register_gauge!(
        "sentinel_active_connections",
        "Number of active connections"
    ).unwrap();

    static ref REQUEST_DURATION: Histogram = register_histogram!(
        "sentinel_request_duration_seconds",
        "Request duration in seconds"
    ).unwrap();
}

pub fn record_transfer(bytes: u64) {
    BYTES_TRANSFERRED.inc_by(bytes as f64);
}

pub fn set_active_connections(count: i64) {
    ACTIVE_CONNECTIONS.set(count as f64);
}
```

## 安全考虑

### 1. 输入验证

```rust
use validator::Validate;

#[derive(Debug, Validate, Deserialize)]
pub struct CreateProxyRequest {
    #[validate(range(min = 1, max = 65535))]
    pub listen_port: u16,

    #[validate(ip)]
    pub target_ip: String,

    #[validate(range(min = 1, max = 65535))]
    pub target_port: u16,

    #[validate(range(min = 0, max = 1000))]
    pub rate_limit_mbps: Option<u32>,
}

pub async fn create_proxy(req: CreateProxyRequest) -> Result<()> {
    req.validate()?;
    // 处理请求
    Ok(())
}
```

### 2. 权限控制

```rust
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub client_id: String,
    pub permissions: Vec<String>,
    pub exp: usize,
}

pub fn verify_token(token: &str) -> Result<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(JWT_SECRET.as_ref()),
        &Validation::default(),
    )?;

    Ok(token_data.claims)
}

pub fn check_permission(claims: &Claims, required: &str) -> Result<()> {
    if !claims.permissions.contains(&required.to_string()) {
        anyhow::bail!("Permission denied: {} required", required);
    }
    Ok(())
}
```

## 测试策略

### 单元测试

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_proxy_connection() {
        let proxy = ProxyServer::new("127.0.0.1:0", "127.0.0.1:8080");
        let addr = proxy.start().await.unwrap();

        // 创建测试连接
        let stream = TcpStream::connect(addr).await.unwrap();
        // 验证连接
        assert!(stream.peer_addr().is_ok());
    }

    #[test]
    fn test_rate_limiter() {
        let limiter = RateLimiter::new(1000); // 1KB/s

        // 应该允许小数据
        assert!(limiter.try_consume(100));

        // 大数据应该被限制
        assert!(!limiter.try_consume(10000));
    }
}
```

### 集成测试

```rust
// tests/integration_test.rs
use sentinel_client::Client;
use sentinel_server::Server;

#[tokio::test]
async fn test_full_flow() {
    // 启动服务器
    let server = Server::new(test_config()).await.unwrap();
    tokio::spawn(async move {
        server.start().await.unwrap();
    });

    // 启动客户端
    let client = Client::new(client_config()).await.unwrap();

    // 测试注册
    client.register().await.unwrap();

    // 测试任务下发
    let task = create_test_task();
    client.execute_task(task).await.unwrap();

    // 验证结果
    assert_eq!(client.get_status().await.unwrap(), "active");
}
```

## 故障排查

### 日志配置

```rust
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sentinel=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
}
```

### 调试工具

```bash
# 查看网络连接
ss -tunlp | grep sentinel

# 查看 iptables 规则
iptables -L -n -v

# 监控系统调用
strace -p $(pgrep sentinel-client)

# 性能分析
perf record -g ./target/release/sentinel-client
perf report
```

## 未来扩展

1. **支持更多协议**
   - QUIC 传输支持
   - HTTP/3 支持
   - 自定义协议扩展

2. **高级功能**
   - 智能路由选择
   - 故障自动切换
   - 负载均衡算法

3. **管理功能**
   - 配置热更新
   - 插件系统
   - API Gateway 功能

4. **监控增强**
   - 分布式追踪
   - 实时告警
   - 性能分析仪表板

## 项目时间线

### 第一阶段 (2周)
- [x] 项目架构设计
- [ ] 基础框架搭建
- [ ] Client 注册与心跳
- [ ] Server 基础 API

### 第二阶段 (2周)
- [ ] TCP 端口转发
- [ ] 流量统计
- [ ] 基础 Web UI
- [ ] Docker 部署

### 第三阶段 (2周)
- [ ] iptables 管理
- [ ] 加密传输
- [ ] WebSocket 封装
- [ ] 限速功能

### 第四阶段 (2周)
- [ ] 流量中继
- [ ] 高级路由
- [ ] 监控告警
- [ ] 性能优化

### 第五阶段 (持续)
- [ ] 功能完善
- [ ] 测试覆盖
- [ ] 文档完善
- [ ] 生产部署

---

本文档将随项目进展持续更新，请参考最新版本进行开发。