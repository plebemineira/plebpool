use crate::ln::config::LnConfig;
use std::str::FromStr;
use tracing::{error, info};

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

        if let Some(peers) = ln_config.peers {
            for peer in peers {
                let node_id: bitcoin::secp256k1::PublicKey =
                    bitcoin::secp256k1::PublicKey::from_str(peer.node_id.as_str())?;
                let address: ldk_node::lightning::ln::msgs::SocketAddress =
                    ldk_node::lightning::ln::msgs::SocketAddress::from_str(peer.address.as_str())
                        .expect("failed to parse LN peer addressc");

                match ldk_node.connect(node_id, address, true) {
                    Ok(_) => info!(
                        "LDK: connecting to peer: {}, id: {}",
                        peer.address, peer.node_id
                    ),
                    Err(e) => error!(
                        "LDK: failed to connect to peer: {}, id: {}, error: {}",
                        peer.address, peer.node_id, e
                    ),
                }
            }
        }
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
