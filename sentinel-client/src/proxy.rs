use anyhow::Result;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

use crate::limiter::RateLimiter;
use crate::stats::StatsCollector;

pub struct ProxyServer {
    listen_addr: SocketAddr,
    target_addr: SocketAddr,
    stats: Arc<StatsCollector>,
    limiter: Option<Arc<RateLimiter>>,
}

impl ProxyServer {
    pub fn new(listen_addr: SocketAddr, target_addr: SocketAddr) -> Self {
        Self {
            listen_addr,
            target_addr,
            stats: Arc::new(StatsCollector::new()),
            limiter: None,
        }
    }

    pub fn with_rate_limit(mut self, mbps: u32) -> Self {
        if mbps > 0 {
            self.limiter = Some(Arc::new(RateLimiter::new(mbps * 1024 * 1024 / 8)));
        }
        self
    }

    pub async fn start(&self) -> Result<()> {
        let listener = TcpListener::bind(self.listen_addr).await?;
        tracing::info!("Proxy listening on {}", listener.local_addr()?);

        loop {
            let (inbound, peer_addr) = listener.accept().await?;
            let target = self.target_addr;
            let stats = self.stats.clone();
            let limiter = self.limiter.clone();

            tokio::spawn(async move {
                if let Err(e) = Self::handle_connection(inbound, target, stats, limiter).await {
                    tracing::error!("Connection error from {}: {}", peer_addr, e);
                }
            });
        }
    }

    async fn handle_connection(
        inbound: TcpStream,
        target: SocketAddr,
        stats: Arc<StatsCollector>,
        limiter: Option<Arc<RateLimiter>>,
    ) -> Result<()> {
        let outbound = TcpStream::connect(target).await?;

        let peer_addr = inbound.peer_addr()?.to_string();
        stats.new_connection(peer_addr);

        let (mut ri, mut wi) = inbound.into_split();
        let (mut ro, mut wo) = outbound.into_split();

        let stats1 = stats.clone();
        let stats2 = stats.clone();
        let limiter1 = limiter.clone();
        let limiter2 = limiter.clone();

        let client_to_server = tokio::spawn(async move {
            let mut buf = vec![0u8; 8192];
            loop {
                let n = match ri.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(e) => {
                        tracing::error!("Read error: {}", e);
                        break;
                    }
                };

                if let Some(limiter) = &limiter1 {
                    limiter.wait_for_capacity(n).await;
                }

                if let Err(e) = wo.write_all(&buf[..n]).await {
                    tracing::error!("Write error: {}", e);
                    break;
                }

                stats1.add_bytes_sent(n);
            }
            Ok::<_, anyhow::Error>(())
        });

        let server_to_client = tokio::spawn(async move {
            let mut buf = vec![0u8; 8192];
            loop {
                let n = match ro.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => n,
                    Err(e) => {
                        tracing::error!("Read error: {}", e);
                        break;
                    }
                };

                if let Some(limiter) = &limiter2 {
                    limiter.wait_for_capacity(n).await;
                }

                if let Err(e) = wi.write_all(&buf[..n]).await {
                    tracing::error!("Write error: {}", e);
                    break;
                }

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