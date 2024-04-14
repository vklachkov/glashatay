use crate::config_validators as validators;
use anyhow::Context;
use garde::Validate;
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};
use url::Url;

#[derive(Clone, Debug, Deserialize, Validate)]
pub struct Config {
    #[garde(dive)]
    pub vk: Vk,

    #[garde(dive)]
    pub telegram: Telegram,

    #[garde(dive)]
    pub database: Database,
}

#[derive(Clone, Debug, Deserialize, Validate)]
pub struct Vk {
    #[garde(custom(validators::is_base_url))]
    pub server: Url,

    #[garde(length(min = 1))]
    pub language: String,

    #[garde(length(min = 1))]
    pub service_key: String,

    #[garde(dive)]
    pub debug: Option<VkDebug>,
}

#[derive(Clone, Debug, Deserialize, Validate)]
pub struct VkDebug {
    #[garde(skip)]
    pub save_responses: bool,

    #[garde(custom(validators::is_directory_and_exists))]
    pub responses_dir_path: PathBuf,
}

#[derive(Clone, Debug, Deserialize, Validate)]
pub struct Telegram {
    #[garde(length(min = 1))]
    pub bot_token: String,
}

#[derive(Clone, Debug, Deserialize, Validate)]
pub struct Database {
    #[garde(custom(validators::is_file_directory_exists))]
    pub path: PathBuf,
}

impl Config {
    pub fn read_from<P: AsRef<Path>>(path: P) -> anyhow::Result<Config> {
        let path = path.as_ref();

        let raw: String = fs::read_to_string(path)
            .with_context(|| format!("reading file from {}", path.display()))?;

        let config: Config = toml::from_str(&raw)
            .with_context(|| format!("deserializing file from {}", path.display()))?;

        config.validate(&()).map_err(|errors| {
            anyhow::anyhow!(
                "invalid values in config '{path}':\n{errors}",
                path = path.display()
            )
        })?;

        Ok(config)
    }
}
