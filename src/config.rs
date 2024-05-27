use serde::{Deserialize, Serialize};
use crate::{ln, pool};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PlebPoolConfig {
    pub pool: pool::config::PoolConfig,
    pub ln: ln::config::LnConfig,
}

impl PlebPoolConfig {
    pub fn new(config_path: String) -> Result<Self, config::ConfigError> {
        let builder = config::Config::builder();
        let config: config::Config = builder
            .add_source(config::File::with_name(&config_path))
            .build()?;

        let plebpool_config: PlebPoolConfig = config.try_deserialize()?;

        Ok(plebpool_config)
    }
}
