mod bot;
mod vk_poll;

use std::{env, process, sync::Arc};

pub struct GlobalState {}

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
    let global_state = Arc::new(GlobalState {});

    _ = tokio::join!(
        tokio::spawn(bot::run(global_state.clone())),
        tokio::spawn(vk_poll::run(global_state)),
    );

    Ok(())
}
