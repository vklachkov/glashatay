use crate::{config, vk_api, GlobalState};
use anyhow::Context;
use std::{sync::Arc, fs};
use url::Url;

pub async fn run(global_state: Arc<GlobalState>) {
    log::info!(
        "Post: {:?}",
        get_post(&global_state.config.vk, "dar.viardo").await
    );
}

async fn get_post(config: &config::Vk, vk_id: &str) -> anyhow::Result<vk_api::Posts> {
    const VERSION: &str = "5.137";
    const METHOD: &str = "wall.get";

    let url = Url::parse_with_params(
        &format!("{base}method/{METHOD}", base = &config.server),
        &[
            ("v", VERSION),
            ("lang", &config.language),
            ("domain", vk_id),
            ("offset", "0"),
            ("count", "5"),
        ],
    )
    .expect("url should be valid");

    log::debug!("Url: {url}");

    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .bearer_auth(&config.service_key)
        .send()
        .await
        .context("executing wall.get")?;

    let response = response
        .text()
        .await
        .context("reading response from wall.get")?;

    // log::debug!("Vk response: {response}");
    _ = fs::write("vk-response.json", &response);

    let response = serde_json::from_str::<vk_api::Response<vk_api::Posts>>(&response)
        .context("parsing response from wall.get")?;

    Ok(response.response)
}