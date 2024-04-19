use std::path::PathBuf;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MintConfig {
    pub url: String,
    pub db_path: PathBuf,
    pub last_pay_path: String,
    pub listen_host: String,
    pub listen_port: u16,
    pub mnemonic: String,
    pub derivation_path: String,
    pub max_order: u8,
    pub min_fee_reserve: cdk::Amount,
    pub min_fee_percent: f32,
}

impl Default for MintConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            db_path: PathBuf::from("/tmp/plebpool/mint.redb"),
            last_pay_path: "/tmp/plebpool/last_pay".to_string(),
            listen_host: String::new(),
            listen_port: 0,
            mnemonic: String::new(),
            derivation_path: "0/0/0/0".to_string(),
            max_order: 32,
            min_fee_reserve: cdk::Amount::default(),
            min_fee_percent: 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcashConfig {
    pub mint: MintConfig,
    pub ln: LnConfig,
}
impl EcashConfig {
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

        let ecash_config: EcashConfig = config.try_deserialize()?;

        Ok(ecash_config)
    }
}
