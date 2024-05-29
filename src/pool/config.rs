use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PoolConfig {
    pub sv1_mining_channel_host: String,
    pub sv1_mining_channel_port: u16,
}
