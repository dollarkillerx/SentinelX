mod api;
mod config;
mod db;
mod manager;

use anyhow::Result;
use clap::Parser;
use std::sync::Arc;
use tracing_subscriber;

use crate::api::create_rpc_module;
use crate::config::Config;
use crate::db::Database;
use crate::manager::ClientManager;

#[derive(Parser, Debug)]
#[clap(author, version, about)]
struct Args {
    #[clap(short, long, default_value = "config.toml")]
    config: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "sentinel=info,tower_http=debug".into()),
        )
        .init();

    let args = Args::parse();
    let config = Config::from_file(&args.config)?;

    tracing::info!("Starting Sentinel Server v{}", env!("CARGO_PKG_VERSION"));

    let db = Database::connect(&config.database.url).await?;
    db.run_migrations().await?;

    let manager = Arc::new(ClientManager::new(db.clone()));

    let manager_clone = manager.clone();
    tokio::spawn(async move {
        manager_clone.start_cleanup_task().await;
    });

    let rpc_module = create_rpc_module(manager.clone()).await?;

    let server = jsonrpsee::server::ServerBuilder::default()
        .build(&config.server.bind_addr)
        .await?;

    let handle = server.start(rpc_module);

    tracing::info!("JSON-RPC server listening on {}", &config.server.bind_addr);

    handle.stopped().await;

    Ok(())
}