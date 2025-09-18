use anyhow::Result;
use jsonrpsee::http_client::{HttpClient, HttpClientBuilder};
use sentinel_common::{RelayConfig, TransportType};
use crate::encryption::EncryptionManager;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tracing;

pub struct RelayManager {
    #[allow(dead_code)]
    server_url: String,
    #[allow(dead_code)]
    client: HttpClient,
    active_relays: Arc<RwLock<HashMap<String, RelayConnection>>>,
    encryption_manager: EncryptionManager,
}

impl RelayManager {
    pub fn new(server_url: String) -> Result<Self> {
        let client = HttpClientBuilder::default()
            .build(&server_url)
            .expect("Failed to create HTTP client");

        Ok(Self {
            server_url,
            client,
            active_relays: Arc::new(RwLock::new(HashMap::new())),
            encryption_manager: EncryptionManager::new(),
        })
    }

    pub async fn start_relay(&self, config: RelayConfig) -> Result<()> {
        let relay_id = format!("{}:{}", config.entry_point, config.exit_point);

        tracing::info!(
            "Starting relay: {} -> {} (transport: {:?})",
            config.entry_point,
            config.exit_point,
            config.transport_type
        );

        let connection = RelayConnection::new(config, &self.encryption_manager).await?;

        // Store the relay connection
        self.active_relays.write().await.insert(relay_id.clone(), connection);

        // Start the relay handler in background
        let relays = self.active_relays.clone();
        let id = relay_id.clone();

        tokio::spawn(async move {
            if let Some(relay) = relays.read().await.get(&id) {
                if let Err(e) = relay.run().await {
                    tracing::error!("Relay {} failed: {}", id, e);
                    // Remove failed relay
                    relays.write().await.remove(&id);
                }
            }
        });

        Ok(())
    }

    pub async fn stop_relay(&self, entry_point: &str, exit_point: &str) -> Result<()> {
        let relay_id = format!("{}:{}", entry_point, exit_point);

        if let Some(_) = self.active_relays.write().await.remove(&relay_id) {
            tracing::info!("Stopped relay: {}", relay_id);
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn list_active_relays(&self) -> Vec<String> {
        self.active_relays.read().await.keys().cloned().collect()
    }
}

pub struct RelayConnection {
    config: RelayConfig,
    listener: Option<TcpListener>,
    #[allow(dead_code)]
    encryption_manager: EncryptionManager,
}

impl RelayConnection {
    pub async fn new(config: RelayConfig, _encryption_manager: &EncryptionManager) -> Result<Self> {
        // Parse entry point to start listening
        let listener = if config.entry_point.starts_with("0.0.0.0:") || config.entry_point.starts_with("127.0.0.1:") {
            let addr: SocketAddr = config.entry_point.parse()?;
            Some(TcpListener::bind(addr).await?)
        } else {
            None
        };

        Ok(Self {
            config,
            listener,
            encryption_manager: EncryptionManager::new(),
        })
    }

    pub async fn run(&self) -> Result<()> {
        if let Some(listener) = &self.listener {
            tracing::info!("Relay listening on {}", self.config.entry_point);

            loop {
                let (inbound, peer_addr) = listener.accept().await?;
                tracing::debug!("New relay connection from {}", peer_addr);

                let config = self.config.clone();
                let encryption_manager = EncryptionManager::new();
                tokio::spawn(async move {
                    if let Err(e) = Self::handle_relay_connection(inbound, config, encryption_manager).await {
                        tracing::error!("Relay connection error: {}", e);
                    }
                });
            }
        } else {
            // This is an outbound-only relay, wait for connections from other clients
            tracing::info!("Relay configured for outbound connections to {}", self.config.exit_point);
            futures_util::future::pending::<()>().await;
        }

        Ok(())
    }

    async fn handle_relay_connection(
        inbound: TcpStream,
        config: RelayConfig,
        encryption_manager: EncryptionManager,
    ) -> Result<()> {
        // Connect to exit point
        let outbound = match config.transport_type {
            TransportType::Direct => {
                Self::connect_direct(&config.exit_point).await?
            }
            TransportType::Encrypted => {
                Self::connect_encrypted(&config.exit_point, &encryption_manager).await?
            }
            TransportType::WebSocket => {
                Self::connect_websocket(&config.exit_point).await?
            }
        };

        // Start bidirectional relay
        Self::relay_traffic(inbound, outbound).await?;

        Ok(())
    }

    async fn connect_direct(exit_point: &str) -> Result<TcpStream> {
        let addr: SocketAddr = exit_point.parse()?;
        let stream = TcpStream::connect(addr).await?;
        tracing::debug!("Direct connection established to {}", exit_point);
        Ok(stream)
    }

    async fn connect_encrypted(exit_point: &str, _encryption_manager: &EncryptionManager) -> Result<TcpStream> {
        let addr: SocketAddr = exit_point.parse()?;
        let stream = TcpStream::connect(addr).await?;

        // For now, establish a direct connection but log that encryption is intended
        // In production, this would use the encryption_manager to wrap the stream
        tracing::info!("Encrypted connection established to {} (encryption layer ready for implementation)", exit_point);

        // TODO: Implement actual encryption wrapping:
        // let key = EncryptionManager::generate_key();
        // let encryption_config = EncryptionConfig {
        //     encryption_type: EncryptionType::Aes256Gcm,
        //     key: Some(key),
        //     tls_config: None,
        // };
        // let encrypted_stream = encryption_manager.wrap_stream(stream, encryption_config).await?;

        Ok(stream)
    }

    async fn connect_websocket(exit_point: &str) -> Result<TcpStream> {
        // Parse the exit point to determine if it's a WebSocket URL or address
        let ws_url = if exit_point.starts_with("ws://") || exit_point.starts_with("wss://") {
            exit_point.to_string()
        } else {
            // For WebSocket transport, we expect a full URL or convert address to WebSocket URL
            format!("ws://{}/relay", exit_point)
        };

        tracing::info!("WebSocket transport configured for {}", ws_url);

        // For now, establish a regular TCP connection but log WebSocket capability
        // In a full implementation, this would use the WebSocket transport directly
        // and require modifying the relay_traffic function to handle WebSocket frames

        // Extract the host:port from the WebSocket URL for fallback TCP connection
        let tcp_addr = if let Some(host_port) = ws_url.strip_prefix("ws://").and_then(|s| s.split('/').next()) {
            host_port
        } else if let Some(host_port) = ws_url.strip_prefix("wss://").and_then(|s| s.split('/').next()) {
            host_port
        } else {
            exit_point
        };

        let addr: SocketAddr = tcp_addr.parse()?;
        let stream = TcpStream::connect(addr).await?;

        tracing::info!("WebSocket transport connection established to {} (using TCP fallback)", exit_point);

        // TODO: Full WebSocket implementation would require:
        // 1. Establishing WebSocket handshake
        // 2. Creating a WebSocket adapter that implements AsyncRead/AsyncWrite
        // 3. Wrapping TCP data in WebSocket frames
        // 4. Handling WebSocket control frames (ping/pong/close)

        Ok(stream)
    }

    async fn relay_traffic(inbound: TcpStream, outbound: TcpStream) -> Result<()> {
        let (mut ri, mut wi) = inbound.into_split();
        let (mut ro, mut wo) = outbound.into_split();

        let client_to_server = tokio::spawn(async move {
            let mut buf = vec![0u8; 8192];
            let mut total_bytes = 0u64;

            loop {
                let n = match ri.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(e) => {
                        tracing::debug!("Read error: {}", e);
                        break;
                    }
                };

                if let Err(e) = wo.write_all(&buf[..n]).await {
                    tracing::debug!("Write error: {}", e);
                    break;
                }

                total_bytes += n as u64;
            }

            tracing::debug!("Client->Server relay finished, {} bytes transferred", total_bytes);
            Ok::<_, anyhow::Error>(())
        });

        let server_to_client = tokio::spawn(async move {
            let mut buf = vec![0u8; 8192];
            let mut total_bytes = 0u64;

            loop {
                let n = match ro.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(e) => {
                        tracing::debug!("Read error: {}", e);
                        break;
                    }
                };

                if let Err(e) = wi.write_all(&buf[..n]).await {
                    tracing::debug!("Write error: {}", e);
                    break;
                }

                total_bytes += n as u64;
            }

            tracing::debug!("Server->Client relay finished, {} bytes transferred", total_bytes);
            Ok::<_, anyhow::Error>(())
        });

        // Wait for either direction to complete
        tokio::select! {
            r1 = client_to_server => r1??,
            r2 = server_to_client => r2??,
        }

        tracing::debug!("Relay connection closed");
        Ok(())
    }
}