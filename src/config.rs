use anyhow::Context;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use url::Url;

#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    pub vk: Vk,
    pub telegram: Telegram,
    pub database: Database,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Vk {
    pub server: Url,
    pub language: String,
    pub service_key: String,
    pub debug: Option<VkDebug>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct VkDebug {
    pub save_responses: bool,
    pub responses_dir_path: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Telegram {
    pub bot_token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Database {
    pub path: PathBuf,
}

impl Config {
    pub fn read_from<P: AsRef<Path>>(path: P) -> anyhow::Result<Config> {
        let path = path.as_ref();

        let raw: String = fs::read_to_string(path)
            .with_context(|| format!("reading file from {}", path.display()))?;

        let config: Config = toml::from_str(&raw)
            .with_context(|| format!("deserializing file from {}", path.display()))?;

        Ok(config)
    }
}
