use crate::cli::CLIArgs;

use clap::Parser;
use tracing::{debug, info};

mod cli;
mod ecash;
mod pool;
mod config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("⛏️ plebs be hashin ⚡");

    let args = CLIArgs::parse();

    debug!("Loading configs from: {}", args.config);

    let plebpool_config = config::PlebPoolConfig::new(args.config.clone())?;

    // launch mint_service
    let mint_service_handle = ecash::service::mint_service(args.config.clone());

    let pool_service = pool::service::PoolService::new(plebpool_config.pool).await?;
    let pool_service_handle = pool_service.serve();

    mint_service_handle.await?;
    pool_service_handle.await?;

    Ok(())
}
