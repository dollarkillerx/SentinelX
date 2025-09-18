use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};

pub struct StatsCollector {
    bytes_sent: AtomicU64,
    bytes_received: AtomicU64,
    connections: AtomicU64,
    connection_details: RwLock<HashMap<String, ConnectionStats>>,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub connected_at: std::time::Instant,
}

impl ConnectionStats {
    pub fn new() -> Self {
        Self {
            bytes_sent: 0,
            bytes_received: 0,
            connected_at: std::time::Instant::now(),
        }
    }
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

    #[allow(dead_code)]
    pub fn close_connection(&self, peer: &str) {
        let mut details = self.connection_details.write();
        details.remove(peer);
    }

    #[allow(dead_code)]
    pub fn get_stats(&self) -> Stats {
        Stats {
            bytes_sent: self.bytes_sent.load(Ordering::Relaxed),
            bytes_received: self.bytes_received.load(Ordering::Relaxed),
            total_connections: self.connections.load(Ordering::Relaxed),
            active_connections: self.connection_details.read().len() as u64,
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Stats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub total_connections: u64,
    pub active_connections: u64,
}