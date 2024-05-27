use crate::cli::CLIArgs;

use clap::Parser;
use tracing::{debug, info};

mod cli;
mod ln;
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

    let _ln_service = ln::service::LnService::new(plebpool_config.ln)?;

    let pool_service = pool::service::PoolService::new(plebpool_config.pool).await?;
    let pool_service_handle = pool_service.serve();

    pool_service_handle.await?;

    // let the services do their jobs asynchronously,
    // while keeping the main thread alive
    loop { tokio::task::yield_now().await; }

    #[allow(unreachable_code)]
    Ok(())
}
