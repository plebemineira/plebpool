use crate::ln::config::LnConfig;
use tracing::info;

pub struct LnService {
    #[allow(dead_code)]
    ldk_node: ldk_node::Node<ldk_node::io::sqlite_store::SqliteStore>,
}

impl LnService {
    pub fn new(ln_config: LnConfig) -> anyhow::Result<Self> {
        // init ldk node builder
        let mut ldk_node_builder = ldk_node::Builder::new();

        // set network
        if let Some(network) = ln_config.network {
            match network.as_str() {
                "testnet" => ldk_node_builder.set_network(ldk_node::Network::Testnet),
                "regtest" => ldk_node_builder.set_network(ldk_node::Network::Regtest),
                "mainnet" => ldk_node_builder.set_network(ldk_node::Network::Bitcoin),
                "signet" => ldk_node_builder.set_network(ldk_node::Network::Signet),
                _ => return Err(anyhow::anyhow!("Impossible value on ln.network config")),
            };
        } else {
            // mainnet if no network is specified
            ldk_node_builder.set_network(ldk_node::Network::Bitcoin);
        }

        // set log level
        if let Some(log_level) = ln_config.log_level {
            match log_level.as_str() {
                "gossip" => {
                    tracing_subscriber::fmt()
                        .with_max_level(tracing::Level::INFO)
                        .init();
                    ldk_node_builder.set_log_level(ldk_node::LogLevel::Gossip)
                }
                "trace" => {
                    tracing_subscriber::fmt()
                        .with_max_level(tracing::Level::TRACE)
                        .init();
                    ldk_node_builder.set_log_level(ldk_node::LogLevel::Trace)
                }
                "debug" => {
                    tracing_subscriber::fmt()
                        .with_max_level(tracing::Level::DEBUG)
                        .init();
                    ldk_node_builder.set_log_level(ldk_node::LogLevel::Debug)
                }
                "info" => {
                    tracing_subscriber::fmt()
                        .with_max_level(tracing::Level::INFO)
                        .init();
                    ldk_node_builder.set_log_level(ldk_node::LogLevel::Info)
                }
                "warn" => {
                    tracing_subscriber::fmt()
                        .with_max_level(tracing::Level::ERROR)
                        .init();
                    ldk_node_builder.set_log_level(ldk_node::LogLevel::Warn)
                }
                "error" => {
                    tracing_subscriber::fmt()
                        .with_max_level(tracing::Level::ERROR)
                        .init();
                    ldk_node_builder.set_log_level(ldk_node::LogLevel::Error)
                }
                _ => return Err(anyhow::anyhow!("Impossible value on ln.log_level config")),
            };
        }

        // set esplora server
        if let Some(esplora_server_url) = ln_config.esplora_server_url {
            ldk_node_builder.set_esplora_server(esplora_server_url);
        }

        // set gossip source (RapidGossipSync)
        if let Some(gossip_source_rgs) = ln_config.gossip_source_rgs {
            ldk_node_builder.set_gossip_source_rgs(gossip_source_rgs);
        }

        // build ldk_node
        let ldk_node = ldk_node_builder.build()?;

        Ok(Self { ldk_node })
    }

    pub async fn serve(self) -> anyhow::Result<tokio::task::JoinHandle<anyhow::Result<()>>> {
        info!("LDK: starting LN service");

        let handle = tokio::task::spawn(async move {
            self.ldk_node.start()?;
            Ok(())
        });

        Ok(handle)
    }
}
