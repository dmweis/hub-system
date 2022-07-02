#![allow(dead_code)]
// TODO(David): Remove this?

mod blinds_service;
mod configuration;
mod discord_service;
mod ioc;
mod mqtt_server;
mod routes;
mod sec_system;
mod speech_service;

use crate::{blinds_service::BlindsService, ioc::IocContainer, mqtt_server::start_mqtt_service};
use configuration::get_configuration;
use discord_service::DiscordService;
use log::*;
use sec_system::SecSystem;
use simplelog::*;
use speech_service::SpeechService;
use std::path::PathBuf;
use structopt::StructOpt;

const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(StructOpt, Debug)]
#[structopt(
    version = VERSION,
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
    info!("Starting Hub System version {}", VERSION);
    let app_config = get_configuration(opts.config)?;

    // build services
    let container = IocContainer::default();

    let speech_service = SpeechService::new(container.clone());
    container.register(speech_service.clone());

    let blinds_service = BlindsService::new(container.clone());
    container.register(blinds_service);

    let discord_service = DiscordService::new(container.clone(), app_config.discord_bot.clone());
    container.register(discord_service.clone());

    let sec_system = SecSystem::new(container.clone());
    container.register(sec_system);

    start_mqtt_service(app_config.clone(), container.clone())?;

    let online_message = format!("Hub system version {} is online", VERSION);
    speech_service.say_cheerful(&online_message).await?;
    discord_service.send_notification(online_message).await?;

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
