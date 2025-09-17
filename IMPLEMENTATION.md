# SentinelX Implementation Guide

## Quick Start

### Prerequisites

- Rust 1.83+
- PostgreSQL 14+
- Docker & Docker Compose (optional)

### Local Development

1. **Install dependencies:**
```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install PostgreSQL (macOS)
brew install postgresql
brew services start postgresql

# Install PostgreSQL (Ubuntu)
sudo apt-get install postgresql postgresql-contrib
```

2. **Setup database:**
```bash
# Create database and user
createdb sentinel
createuser sentinel

# Run migrations
psql -U sentinel -d sentinel -f sentinel-server/migrations/001_create_tables.sql
```

3. **Build the project:**
```bash
make build
```

4. **Run the server:**
```bash
make run-server
```

5. **Run the client (in another terminal):**
```bash
make run-client
```

### Docker Deployment

```bash
# Build and start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

## Project Structure

```
SentinelX/
├── sentinel-common/       # Shared types and protocols
├── sentinel-client/       # Client implementation
│   ├── src/
│   │   ├── main.rs       # Entry point
│   │   ├── register.rs   # Registration & heartbeat
│   │   ├── monitor.rs    # System monitoring
│   │   ├── proxy.rs      # TCP proxy
│   │   ├── iptables.rs   # iptables management
│   │   ├── stats.rs      # Traffic statistics
│   │   └── limiter.rs    # Rate limiting
│   └── config.toml       # Client configuration
├── sentinel-server/       # Server implementation
│   ├── src/
│   │   ├── main.rs       # Entry point
│   │   ├── api/          # JSON-RPC API
│   │   ├── db/           # Database layer
│   │   └── manager.rs    # Client management
│   └── config.toml       # Server configuration
└── docker-compose.yml     # Docker orchestration
```

## Key Features Implemented

### Client Features
- ✅ Auto-registration with server
- ✅ Heartbeat mechanism
- ✅ System monitoring (CPU, memory, disk, network)
- ✅ TCP port forwarding
- ✅ Traffic statistics
- ✅ Rate limiting
- ✅ iptables management

### Server Features
- ✅ Client registration & authentication
- ✅ Heartbeat tracking
- ✅ Metrics storage (PostgreSQL)
- ✅ JSON-RPC API
- ✅ Auto-cleanup of inactive clients
- ✅ Metrics history

### System Monitoring
- CPU usage percentage
- Memory usage (used/total)
- Disk usage (used/total)
- Network bandwidth (rx/tx rates)
- Real-time metrics reporting

## Configuration

### Client Configuration
```toml
[client]
id = ""                    # Auto-generated if empty
hostname = ""              # Auto-detected if empty

[server]
url = "http://localhost:8080"
heartbeat_interval = 30    # seconds

[proxy]
listen_addr = "0.0.0.0:8888"
target_addr = "127.0.0.1:8080"

[monitoring]
enabled = true
report_interval = 30       # seconds
```

### Server Configuration
```toml
[server]
bind_addr = "0.0.0.0:8080"

[database]
url = "postgres://sentinel:sentinel123@localhost/sentinel"

[client_management]
heartbeat_timeout = 120    # seconds
cleanup_interval = 60      # seconds
```

## API Endpoints

### JSON-RPC Methods

- `client.register` - Register new client
- `client.heartbeat` - Send heartbeat with metrics
- `client.list` - List all clients
- `metrics.get_latest` - Get latest metrics for client
- `metrics.get_history` - Get metrics history
- `metrics.get_summary` - Get system-wide summary

## Testing

```bash
# Run unit tests
cargo test --workspace

# Run with logging
RUST_LOG=debug cargo run --bin sentinel-server

# Test client registration
curl -X POST http://localhost:8080/rpc \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"client.list","params":[],"id":1}'
```

## Performance Optimization

- Zero-copy TCP proxy using tokio
- Connection pooling for database
- Async/await throughout
- Rate limiting with token bucket algorithm
- Efficient metrics collection with sysinfo

## Security Features

- Token-based authentication
- Secure credential storage
- Network isolation with Docker
- iptables rule validation
- Rate limiting

## Monitoring & Debugging

```bash
# Enable debug logging
RUST_LOG=sentinel=debug cargo run

# Monitor system resources
docker stats

# Database queries
psql -U sentinel -d sentinel -c "SELECT * FROM clients;"
psql -U sentinel -d sentinel -c "SELECT * FROM client_metrics ORDER BY recorded_at DESC LIMIT 10;"
```

## Common Issues

1. **Permission denied for iptables:**
   - Run client with `sudo` or in Docker with `--cap-add NET_ADMIN`

2. **Database connection failed:**
   - Check PostgreSQL is running
   - Verify connection string in config

3. **High CPU usage:**
   - Check monitoring interval settings
   - Verify rate limiting is configured

## Next Steps

- [ ] Add WebSocket transport
- [ ] Implement encrypted transport
- [ ] Add Web UI
- [ ] Support client groups
- [ ] Add alerting system
- [ ] Implement HA for server
- [ ] Add Prometheus metrics export

## License

MIT