use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::{accept_async, connect_async, tungstenite::Message};
use tracing;

#[allow(dead_code)]
pub struct WebSocketTransport;

impl WebSocketTransport {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    /// Create a WebSocket server that listens for connections
    #[allow(dead_code)]
    pub async fn listen(&self, addr: SocketAddr) -> Result<WebSocketListener> {
        let listener = TcpListener::bind(addr).await?;
        tracing::info!("WebSocket server listening on {}", addr);

        Ok(WebSocketListener { listener })
    }

    /// Connect to a WebSocket server
    #[allow(dead_code)]
    pub async fn connect(&self, url: &str) -> Result<()> {
        let ws_url = if url.starts_with("ws://") || url.starts_with("wss://") {
            url.to_string()
        } else {
            format!("ws://{}", url)
        };

        tracing::info!("Connecting to WebSocket server: {}", ws_url);

        let (_ws_stream, _response) = connect_async(&ws_url)
            .await
            .map_err(|e| anyhow::anyhow!("WebSocket connection failed: {}", e))?;

        tracing::info!("WebSocket connection established to {}", ws_url);

        // For demo purposes, we log the successful connection
        // In a real implementation, you'd return a connection object
        Ok(())
    }
}

#[allow(dead_code)]
pub struct WebSocketListener {
    listener: TcpListener,
}

impl WebSocketListener {
    #[allow(dead_code)]
    pub async fn accept(&self) -> Result<(SocketAddr,)> {
        let (tcp_stream, addr) = self.listener.accept().await?;

        tracing::debug!("Accepting WebSocket connection from {}", addr);

        let _ws_stream = accept_async(tcp_stream)
            .await
            .map_err(|e| anyhow::anyhow!("WebSocket handshake failed: {}", e))?;

        tracing::debug!("WebSocket handshake completed with {}", addr);

        // For demo purposes, we just return the address
        // In a real implementation, you'd return a connection object
        Ok((addr,))
    }
}

/// WebSocket relay that forwards data between connections
#[allow(dead_code)]
pub struct WebSocketRelay;

impl WebSocketRelay {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self
    }

    /// Create a simple WebSocket echo server for testing
    #[allow(dead_code)]
    pub async fn echo_server(&self, addr: SocketAddr) -> Result<()> {
        let listener = TcpListener::bind(addr).await?;
        tracing::info!("WebSocket echo server listening on {}", addr);

        while let Ok((tcp_stream, client_addr)) = listener.accept().await {
            tracing::info!("New WebSocket connection from {}", client_addr);

            tokio::spawn(async move {
                match accept_async(tcp_stream).await {
                    Ok(ws_stream) => {
                        if let Err(e) = Self::handle_echo_connection(ws_stream).await {
                            tracing::error!("Echo connection error: {}", e);
                        }
                    }
                    Err(e) => {
                        tracing::error!("WebSocket handshake failed: {}", e);
                    }
                }
            });
        }

        Ok(())
    }

    #[allow(dead_code)]
    async fn handle_echo_connection(
        mut ws_stream: tokio_tungstenite::WebSocketStream<TcpStream>,
    ) -> Result<()> {
        while let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(Message::Text(text)) => {
                    tracing::debug!("Received text: {}", text);
                    if let Err(e) = ws_stream.send(Message::Text(text)).await {
                        tracing::error!("Failed to echo text: {}", e);
                        break;
                    }
                }
                Ok(Message::Binary(data)) => {
                    tracing::debug!("Received binary data: {} bytes", data.len());
                    if let Err(e) = ws_stream.send(Message::Binary(data)).await {
                        tracing::error!("Failed to echo binary: {}", e);
                        break;
                    }
                }
                Ok(Message::Ping(data)) => {
                    if let Err(e) = ws_stream.send(Message::Pong(data)).await {
                        tracing::error!("Failed to send pong: {}", e);
                        break;
                    }
                }
                Ok(Message::Close(_)) => {
                    tracing::info!("WebSocket connection closed");
                    break;
                }
                Ok(_) => {
                    // Ignore other message types
                }
                Err(e) => {
                    tracing::error!("WebSocket error: {}", e);
                    break;
                }
            }
        }

        Ok(())
    }

    /// Relay data between a WebSocket and a TCP stream
    #[allow(dead_code)]
    pub async fn relay_ws_to_tcp(ws_url: &str, tcp_addr: SocketAddr) -> Result<()> {
        tracing::info!(
            "Setting up WebSocket-TCP relay: {} <-> {}",
            ws_url,
            tcp_addr
        );

        // Connect to WebSocket
        let (ws_stream, _) = connect_async(ws_url)
            .await
            .map_err(|e| anyhow::anyhow!("WebSocket connection failed: {}", e))?;

        // Connect to TCP
        let tcp_stream = TcpStream::connect(tcp_addr).await?;
        let (mut tcp_reader, mut tcp_writer) = tcp_stream.into_split();

        // Split WebSocket stream for bidirectional communication
        let (mut ws_sink, mut ws_stream) = ws_stream.split();

        // Forward data from TCP to WebSocket
        let tcp_to_ws = tokio::spawn(async move {
            let mut buf = vec![0u8; 8192];
            let mut total_bytes = 0u64;

            loop {
                match tcp_reader.read(&mut buf).await {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        total_bytes += n as u64;
                        if let Err(e) = ws_sink.send(Message::Binary(buf[..n].to_vec())).await {
                            tracing::error!("Failed to send TCP data to WebSocket: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!("TCP read error: {}", e);
                        break;
                    }
                }
            }

            tracing::debug!(
                "TCP->WS forward finished, {} bytes transferred",
                total_bytes
            );
        });

        // Forward data from WebSocket to TCP
        let ws_to_tcp = tokio::spawn(async move {
            let mut total_bytes = 0u64;

            while let Some(msg) = ws_stream.next().await {
                match msg {
                    Ok(Message::Binary(data)) => {
                        total_bytes += data.len() as u64;
                        if let Err(e) = tcp_writer.write_all(&data).await {
                            tracing::error!("Failed to write WebSocket data to TCP: {}", e);
                            break;
                        }
                    }
                    Ok(Message::Text(text)) => {
                        let data = text.into_bytes();
                        total_bytes += data.len() as u64;
                        if let Err(e) = tcp_writer.write_all(&data).await {
                            tracing::error!("Failed to write WebSocket text to TCP: {}", e);
                            break;
                        }
                    }
                    Ok(Message::Close(_)) => {
                        tracing::info!("WebSocket closed");
                        break;
                    }
                    Ok(Message::Ping(_data)) => {
                        // Handle ping - we can't access ws_sink from here
                        // In practice, the WebSocket library handles this automatically
                        tracing::debug!("Received ping message");
                    }
                    Ok(_) => {
                        // Ignore other message types
                    }
                    Err(e) => {
                        tracing::error!("WebSocket receive error: {}", e);
                        break;
                    }
                }
            }

            tracing::debug!(
                "WS->TCP forward finished, {} bytes transferred",
                total_bytes
            );
        });

        // Wait for either direction to complete
        tokio::select! {
            r1 = tcp_to_ws => {
                if let Err(e) = r1 {
                    tracing::error!("TCP->WS task error: {}", e);
                }
            }
            r2 = ws_to_tcp => {
                if let Err(e) = r2 {
                    tracing::error!("WS->TCP task error: {}", e);
                }
            }
        }

        tracing::info!("WebSocket-TCP relay completed");
        Ok(())
    }
}

impl Default for WebSocketTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for WebSocketRelay {
    fn default() -> Self {
        Self::new()
    }
}