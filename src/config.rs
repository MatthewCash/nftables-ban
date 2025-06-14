use anyhow::Result;
use serde::Deserialize;
use std::path::Path;
use tokio::fs;

use crate::{database::DatabaseConfig, nftables::NftablesConfig, webhook::WebhookConfig};

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    pub nftables: NftablesConfig,
    pub webhook: WebhookConfig,
}

pub async fn load_config(path: impl AsRef<Path>) -> Result<Config> {
    let config_str = fs::read_to_string(path).await?;
    Ok(toml::from_str(&config_str)?)
}
