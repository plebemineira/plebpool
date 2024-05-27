mod downstream;

use crate::pool::config::PoolConfig;
use crate::pool::service::downstream::DownstreamService;

use tracing::info;

pub struct PoolService {
    downstream_service: DownstreamService
}

impl PoolService {
    pub async fn new(pool_config: PoolConfig) -> anyhow::Result<Self> {
        let downstream_service = DownstreamService::new(pool_config.downstream_listen_host, pool_config.downstream_listen_port).await?;
        Ok(Self {
            downstream_service
        })
    }

    pub async fn serve(self) -> anyhow::Result<tokio::task::JoinHandle<anyhow::Result<()>>> {
        info!("SV2: starting Pool service");

        let downstream_service_handle = self.downstream_service.serve();

        let handle = tokio::task::spawn(async move {
            downstream_service_handle.await?;
            Ok(())
        });

        Ok(handle)
    }
}
