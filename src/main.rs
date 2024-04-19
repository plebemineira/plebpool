use crate::cli::CLIArgs;

use clap::Parser;
use tracing::{debug, info};

mod cli;
mod ecash;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    info!("⛏️ plebs be hashin ⚡");

    let args = CLIArgs::parse();

    debug!("Loading configs from: {}", args.config);

    // launch mint_service
    let mint_service_handle = ecash::service::service(args.config);

    // await on mint_service
    mint_service_handle.await?;

    Ok(())
}
