mod bot;
mod config;
mod db;
mod domain;
mod vk_api;
mod vk_poller;

use anyhow::Context;
use argh::FromArgs;
use config::Config;
use std::{env, path::PathBuf, process, sync::Arc};

#[derive(FromArgs)]
#[argh(description = "")]
pub struct Args {
    /// path to config
    #[argh(option)]
    config: PathBuf,

    /// enable extra logs
    #[argh(switch)]
    verbose: bool,

    /// enable trace logs
    #[argh(switch)]
    trace: bool,
}

pub struct GlobalState {
    pub config: Config,
}

#[tokio::main]
async fn main() {
    let args = argh::from_env::<Args>();

    setup_logger(&args);
    hello(&args);

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

fn setup_logger(args: &Args) {
    let level = if args.trace {
        log::LevelFilter::Trace
    } else if args.verbose {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    simple_logger::SimpleLogger::new()
        .with_module_level("reqwest", log::LevelFilter::Off)
        .with_level(level)
        .init()
        .unwrap()
}

fn hello(args: &Args) {
    log::info!(
        "{bin} version {version}, commit {commit}, config from {config_path}, {verbose}",
        bin = env!("CARGO_PKG_NAME"),
        version = env!("CARGO_PKG_VERSION"),
        commit = env!("GIT_COMMIT_HASH"),
        config_path = args.config.display(),
        verbose = if args.trace {
            "trace enabled"
        } else if args.verbose {
            "verbose enabled"
        } else {
            "additional logs disabled"
        },
    );
}

async fn run(args: Args) -> anyhow::Result<()> {
    let config = config::read_from(args.config).context("reading config")?;
    let global_state = Arc::new(GlobalState { config });

    _ = tokio::join!(
        tokio::spawn(bot::run(global_state.clone())),
        // tokio::spawn(vk_poll::run(global_state)),
    );

    Ok(())
}
