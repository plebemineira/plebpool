use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LnConfig {
  pub esplora_server_url: String,
  pub gossip_source_rgs: String,
  pub network: String,
  pub log_level: String
}
