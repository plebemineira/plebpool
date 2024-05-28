use crate::cli::CLIArgs;

use clap::Parser;
use tracing::{debug, info};

mod cli;
mod config;
mod ln;
mod pool;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    info!("⛏️ plebs be hashin ⚡");

    let args = CLIArgs::parse();

    debug!("Loading configs from: {}", args.config);

    let plebpool_config = config::PlebPoolConfig::new(args.config.clone())?;

    let ln_service = ln::service::LnService::new(plebpool_config.ln)?;
    let ln_service_handle = ln_service.serve();

    let pool_service = pool::service::PoolService::new(plebpool_config.pool).await?;
    let pool_service_handle = pool_service.serve();

    pool_service_handle.await?;
    ln_service_handle.await?;

    // let the services do their jobs asynchronously,
    // while keeping the main thread alive
    loop {
        tokio::task::yield_now().await;
    }

    #[allow(unreachable_code)]
    Ok(())
}
