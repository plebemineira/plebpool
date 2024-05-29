mod job_declarator_service;
mod sv1_mining_channel_service;
mod sv2_mining_channel_service;
mod template_receiver_service;

use crate::pool::config::PoolConfig;

use tracing::info;

pub struct PoolService {
    sv1_mining_channel_service: sv1_mining_channel_service::Sv1MiningChannelService,
}

impl PoolService {
    pub async fn new(pool_config: PoolConfig) -> anyhow::Result<Self> {
        let downstream_service = sv1_mining_channel_service::Sv1MiningChannelService::new(
            pool_config.sv1_mining_channel_host,
            pool_config.sv1_mining_channel_port,
        )
        .await?;
        Ok(Self {
            sv1_mining_channel_service: downstream_service,
        })
    }

    pub async fn serve(self) -> anyhow::Result<tokio::task::JoinHandle<anyhow::Result<()>>> {
        info!("SV2: starting Pool service");

        // let downstream_service_handle = self.sv1_mining_channel_service.serve();

        let handle = tokio::task::spawn(async move {
            // downstream_service_handle.await?;
            Ok(())
        });

        Ok(handle)
    }
}
