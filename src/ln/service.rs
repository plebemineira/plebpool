use crate::ln::config::LnConfig;

pub struct LnService {
  #[allow(dead_code)]
  ldk_node: ldk_node::Node<ldk_node::io::sqlite_store::SqliteStore>
}

impl LnService {
  pub fn new(ln_config: LnConfig) -> anyhow::Result<Self> {
    // init ldk node builder
    let mut ldk_node_builder = ldk_node::Builder::new();

    // set network
    match ln_config.network.as_str() {
        "testnet" => ldk_node_builder.set_network(ldk_node::Network::Testnet),
        "regtest" => ldk_node_builder.set_network(ldk_node::Network::Regtest),
        "mainnet" => ldk_node_builder.set_network(ldk_node::Network::Bitcoin),
        "signet" => ldk_node_builder.set_network(ldk_node::Network::Signet),
        _ => return Err(anyhow::anyhow!("Impossible value on ln.network config"))
    };

    // set log level
    match ln_config.log_level.as_str() {
        "gossip" => ldk_node_builder.set_log_level(ldk_node::LogLevel::Gossip),
        "trace" => ldk_node_builder.set_log_level(ldk_node::LogLevel::Trace),
        "debug" => ldk_node_builder.set_log_level(ldk_node::LogLevel::Debug),
        "info" => ldk_node_builder.set_log_level(ldk_node::LogLevel::Info),
        "warn" => ldk_node_builder.set_log_level(ldk_node::LogLevel::Warn),
        "error" => ldk_node_builder.set_log_level(ldk_node::LogLevel::Error),
        _ => return Err(anyhow::anyhow!("Impossible value on ln.log_level config"))
    };

    // set esplora server
    ldk_node_builder.set_esplora_server(ln_config.esplora_server_url);

    // set gossip source (RapidGossipSync)
    ldk_node_builder.set_gossip_source_rgs(ln_config.gossip_source_rgs);

    // build ldk_node
    let ldk_node = ldk_node_builder.build()?;

    // start ldk_node
    ldk_node.start()?;

    Ok(Self {
      ldk_node
    })
  }
}