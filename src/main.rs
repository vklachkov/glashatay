use anyhow::Context;
use std::process;
use tokio::fs;
use url::Url;

#[tokio::main]
async fn main() {
    simple_logger::init().unwrap();
    hello();

    match run().await {
        Ok(()) => {
            process::exit(0);
        }
        Err(err) => {
            log::error!("Fatal error: {err:#}");
            process::exit(1);
        }
    }
}

fn hello() {
    log::info!(
        "{name} version {version}",
        name = env!("CARGO_BIN_NAME"),
        version = env!("CARGO_PKG_VERSION")
    );
}

async fn run() -> anyhow::Result<()> {
    const SERVER: &str = "api.vk.com";
    const VERSION: &str = "5.137";
    const LANGUAGE: &str = "ru";
    const METHOD: &str = "wall.get";
    const SERVICE_KEY: &str =
        "d9312976d9312976d931297609da27e21fdd931d9312976bca7810da8f13e38180454b8";

    let url: Url = Url::parse_with_params(
        &format!("https://{SERVER}/method/{METHOD}"),
        &[
            ("v", VERSION),
            ("lang", LANGUAGE),
            ("domain", "dar.viardo"),
            ("offset", "0"),
            ("count", "5"),
        ],
    )
    .expect("url should be valid");

    let client = reqwest::Client::new();

    let response = client
        .get(url)
        .bearer_auth(SERVICE_KEY)
        .send()
        .await
        .context("executing wall.get")?;

    let response = response
        .text()
        .await
        .context("parsing response from wall.get")?;

    fs::write("response.json", response)
        .await
        .context("writing response to the file")?;

    Ok(())
}
