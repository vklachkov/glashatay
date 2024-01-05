mod bot;
mod config;
mod vk_poll;

use anyhow::Context;
use argh::FromArgs;
use config::Config;
use std::{env, path::PathBuf, process, sync::Arc};

#[derive(FromArgs)]
#[argh(description = "")]
pub struct Args {
    #[argh(option, description = "path to the config")]
    config: PathBuf,
}

pub struct GlobalState {
    pub config: Config,
}

#[tokio::main]
async fn main() {
    let args = argh::from_env::<Args>();

    simple_logger::init().unwrap();
    hello();

    match run(args).await {
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

async fn run(args: Args) -> anyhow::Result<()> {
    let config = config::read_from(args.config).context("reading config")?;
    let global_state = Arc::new(GlobalState { config });

    _ = tokio::join!(
        tokio::spawn(bot::run(global_state.clone())),
        tokio::spawn(vk_poll::run(global_state)),
    );

    Ok(())
}
