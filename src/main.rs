mod bot;
mod config;
mod db;
mod domain;
mod vk_api;
mod vk_poller;

use anyhow::Context;
use argh::FromArgs;
use std::{env, path::PathBuf, process, sync::Arc};
use tokio::signal::unix::{signal, SignalKind};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

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
    let config = config::Config::read_from(args.config)
        .map(Arc::new)
        .context("reading config")?;

    let db = db::Db::new(&config.database.path).context("connecting to database")?;

    let tracker = TaskTracker::new();
    let cancellation_token = CancellationToken::new();

    let bot = teloxide::Bot::new(&config.telegram.bot_token);

    let poller_manager = vk_poller::VkPollManager::new(
        config,
        db,
        bot.clone(),
        tracker.clone(),
        cancellation_token.clone(),
    );

    tracker.spawn(bot::run_dialogue(
        bot.clone(),
        poller_manager.clone(),
        cancellation_token.clone(),
    ));

    tracker.spawn(poller_manager.run());

    tracker.spawn(signal_handler(tracker.clone(), cancellation_token.clone()));

    tracker.wait().await;

    Ok(())
}

async fn signal_handler(tracker: TaskTracker, token: CancellationToken) {
    let mut sigint = signal(SignalKind::interrupt()).unwrap();
    let mut sigterm = signal(SignalKind::terminate()).unwrap();

    tokio::select! {
        _ = sigint.recv() => {
            log::debug!("SIGINT received");
        },
        _ = sigterm.recv() => {
            log::debug!("SIGTERM received");
        },
    }

    tracker.close();
    token.cancel();
}
