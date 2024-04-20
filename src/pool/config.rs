use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PoolConfig {
    pub downstream_listen_host: String,
    pub downstream_listen_port: u16,
}