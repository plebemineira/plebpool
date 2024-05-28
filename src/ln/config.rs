use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LnConfig {
    pub esplora_server_url: Option<String>,
    pub gossip_source_rgs: Option<String>,
    pub network: Option<String>,
}
