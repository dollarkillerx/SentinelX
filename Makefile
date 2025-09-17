.PHONY: build test run clean docker help

# Default target
all: build

# Build all components
build:
	cargo build --workspace

# Build release version
release:
	cargo build --release --workspace

# Run tests
test:
	cargo test --workspace

# Format code
fmt:
	cargo fmt --all

# Lint code
lint:
	cargo clippy --all-targets --all-features

# Clean build artifacts
clean:
	cargo clean

# Build Docker images
docker-build:
	docker-compose build

# Run with Docker Compose
docker-up:
	docker-compose up -d

# Stop Docker Compose
docker-down:
	docker-compose down

# View Docker logs
docker-logs:
	docker-compose logs -f

# Run server locally
run-server:
	cargo run --bin sentinel-server -- --config sentinel-server/config.toml

# Run client locally
run-client:
	cargo run --bin sentinel-client -- --config sentinel-client/config.toml

# Setup database (requires running PostgreSQL)
setup-db:
	psql -U sentinel -d sentinel -f sentinel-server/migrations/001_create_tables.sql

# Help
help:
	@echo "Available targets:"
	@echo "  build        - Build all components"
	@echo "  release      - Build release version"
	@echo "  test         - Run tests"
	@echo "  fmt          - Format code"
	@echo "  lint         - Lint code with clippy"
	@echo "  clean        - Clean build artifacts"
	@echo "  docker-build - Build Docker images"
	@echo "  docker-up    - Start services with Docker Compose"
	@echo "  docker-down  - Stop Docker Compose services"
	@echo "  docker-logs  - View Docker logs"
	@echo "  run-server   - Run server locally"
	@echo "  run-client   - Run client locally"
	@echo "  setup-db     - Setup database schema"
	@echo "  help         - Show this help message"