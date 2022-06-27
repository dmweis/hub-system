#![allow(dead_code)]
// TODO(David): Remove this?

mod configuration;
mod mqtt_server;
mod routes;
mod speech_service;

use configuration::get_configuration;
use log::*;
use simplelog::*;
use speech_service::SpeechService;
use std::{io::Read, path::PathBuf};
use structopt::StructOpt;

use crate::mqtt_server::start_mqtt_service;

#[derive(StructOpt, Debug)]
#[structopt(
    version = "0.1.0",
    author = "David M. Weis <dweis7@gmail.com>",
    about = "Hub System"
)]
struct Opts {
    #[structopt(long)]
    config: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    setup_logging();
    let opts = Opts::from_args();
    info!("Starting Hub System");
    let app_config = get_configuration(opts.config)?;
    let mqtt_client = start_mqtt_service(app_config)?;
    let speech_service = SpeechService::new(mqtt_client);
    speech_service
        .say(
            "Hello world! This is hub system reporting",
            speech_service::AzureVoiceStyle::Cheerful,
        )
        .await?;

    tokio::task::spawn_blocking(move || {
        info!("Press Enter to exit...");
        let _ = std::io::stdin().read(&mut [0]).unwrap();
    })
    .await?;
    Ok(())
}

fn setup_logging() {
    if TermLogger::init(
        LevelFilter::Info,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )
    .is_err()
    {
        eprintln!("Failed to create term logger");
        if SimpleLogger::init(LevelFilter::Info, Config::default()).is_err() {
            eprintln!("Failed to create simple logger");
        }
    }
}
