use std::path::PathBuf;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LnConfig {
    pub cln_path: Option<PathBuf>,
    pub invoice_description: Option<String>,
    pub fee_percent: f64,
    pub reserve_fee_min: cdk::Amount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LnMessage {
    PaymentReceived,
}

// assumes /tmp/plebpool exists
// todo: unify all paths into one single root path
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MintConfig {
    pub url: String,
    #[serde(default = "path_default")]
    pub db_path: PathBuf,
    #[serde(default = "last_pay_path")]
    pub last_pay_path: String,
    pub listen_host: String,
    pub listen_port: u16,
    pub mnemonic: String,
    #[serde(default = "derivation_path_default")]
    pub derivation_path: String,
    #[serde(default = "max_order_default")]
    pub max_order: u8,
    pub min_fee_reserve: cdk::Amount,
    pub min_fee_percent: f32,
    pub ln: LnConfig,
}

fn path_default() -> PathBuf {
    PathBuf::from_str("/tmp/plebpool/mint.redb").unwrap()
}

fn derivation_path_default() -> String {
    "0/0/0/0".to_string()
}

fn max_order_default() -> u8 {
    32
}

fn last_pay_path() -> String {
    "/tmp/plebpool/last_pay".to_string()
}

impl MintConfig {
    #[must_use]
    pub fn new(config_file_name: &Option<String>) -> Result<Self, config::ConfigError> {
        let config_path: String = match config_file_name {
            Some(value) => value.clone(),
            None => {
                return Err(config::ConfigError::NotFound(
                    "no config file provided".to_string(),
                ))
            }
        };

        let builder = config::Config::builder();
        let config: config::Config = builder
            .add_source(config::File::with_name(&config_path))
            .build()?;

        let mint_config: MintConfig = config.try_deserialize()?;

        Ok(mint_config)
    }
}
