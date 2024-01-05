use std::{path::{Path, PathBuf}, fs};
use anyhow::Context;
use serde::Deserialize;
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
}

#[derive(Debug, Clone, Deserialize)]
pub struct Telegram {
    pub bot_token: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Database {
    pub path: PathBuf,
}

pub fn read_from<P: AsRef<Path>>(path: P) -> anyhow::Result<Config> {
    let path = path.as_ref();

    let raw: String = fs::read_to_string(path)
        .with_context(|| format!("reading file from {}", path.display()))?;

    let config: Config = serde_json::from_str(&raw)
        .context("context")
        .with_context(|| format!("deserialize json from {}", path.display()))?;

    Ok(config)
}
