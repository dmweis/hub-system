#![allow(dead_code)]
// TODO(David): Remove this?

mod blinds_service;
mod configuration;
mod discord_service;
mod ioc;
mod mqtt_server;
mod routes;
mod speech_service;

use crate::{blinds_service::BlindsService, ioc::IocContainer, mqtt_server::start_mqtt_service};
use configuration::get_configuration;
use discord_service::DiscordService;
use log::*;
use simplelog::*;
use speech_service::SpeechService;
use std::path::PathBuf;
use structopt::StructOpt;

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

    let container = IocContainer::default();

    let mqtt_client = start_mqtt_service(app_config.clone(), container.clone())?;

    let speech_service = SpeechService::new(mqtt_client.clone());
    speech_service.say_cheerful("Hub system online").await?;
    container.register(speech_service);

    let blinds_service = BlindsService::new(mqtt_client.clone());
    container.register(blinds_service);

    let discord_service = DiscordService::new(mqtt_client.clone(), app_config.discord_bot.clone());
    container.register(discord_service);

    std::future::pending::<()>().await;
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