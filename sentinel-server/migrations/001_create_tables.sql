-- Create clients table
CREATE TABLE IF NOT EXISTS clients (
    id VARCHAR(255) PRIMARY KEY,
    hostname VARCHAR(255) NOT NULL,
    ip_address VARCHAR(45) NOT NULL,
    version VARCHAR(50),
    capabilities JSONB,
    last_heartbeat TIMESTAMP WITH TIME ZONE,
    status VARCHAR(20) DEFAULT 'offline',
    -- Latest monitoring data
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

-- Create metrics history table
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

-- Create tasks table
CREATE TABLE IF NOT EXISTS tasks (
    id VARCHAR(255) PRIMARY KEY,
    client_id VARCHAR(255) REFERENCES clients(id) ON DELETE CASCADE,
    task_type VARCHAR(50) NOT NULL,
    payload JSONB,
    status VARCHAR(20) DEFAULT 'pending',
    result JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE
);

CREATE INDEX idx_tasks_client_id ON tasks(client_id);
CREATE INDEX idx_tasks_status ON tasks(status);

-- Create cleanup function for old metrics
CREATE OR REPLACE FUNCTION cleanup_old_metrics()
RETURNS void AS $$
BEGIN
    DELETE FROM client_metrics
    WHERE recorded_at < NOW() - INTERVAL '30 days';
END;
$$ LANGUAGE plpgsql;