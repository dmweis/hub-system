use super::{Injected, ANNOUNCEMENT};
use crate::{
    blinds_service::BlindsService, discord_service::DiscordService, ioc::IocContainer,
    sec_system::SecSystem, speech_service::SpeechService,
};

use async_trait::async_trait;
use log::*;
use mqtt_router::{RouteHandler, RouterError};
use serde::Deserialize;

pub struct DiscordHandler {
    ioc: IocContainer,
}

impl DiscordHandler {
    pub fn new(ioc: IocContainer) -> Box<Self> {
        Box::new(Self { ioc })
    }
}

impl Injected for DiscordHandler {
    fn ioc(&self) -> &IocContainer {
        &self.ioc
    }
}

#[async_trait]
impl RouteHandler for DiscordHandler {
    async fn call(&mut self, _topic: &str, content: &[u8]) -> std::result::Result<(), RouterError> {
        info!("Handling discord data");
        let received_message: ReceivedDiscordMessage =
            serde_json::from_slice(content).map_err(|err| RouterError::HandlerError(err.into()))?;
        if received_message.is_author_bot {
            info!("Skipping message from bot");
            return Ok(());
        }

        let home_speak_id = self
            .get::<DiscordService>()?
            .get_id_from_channel("home_speak");
        if let Some(id) = home_speak_id {
            if received_message.channel_id == id {
                self.get::<SpeechService>()?
                    .say_cheerful(&received_message.content)
                    .await
                    .unwrap();
            }
        }

        // this is some hacky stuff
        // but fun
        let notification_channel = self.get::<DiscordService>()?.notification_channel();
        if received_message.channel_id == notification_channel {
            match received_message.content.to_ascii_lowercase().as_str() {
                "help" => {
                    let message = "Commands:
`help`: this
`disarm`: disarm Sec System
`arm`: arm Sec System
`close blinds`: close all blinds
`open blinds`: open all blinds
                    ";
                    self.get::<DiscordService>()?
                        .send_notification(message.to_owned())
                        .await
                        .map_err(|err| RouterError::HandlerError(err.into()))?;
                }
                "disarm" => {
                    self.get::<SecSystem>()?
                        .disarm()
                        .await
                        .map_err(|err| RouterError::HandlerError(err.into()))?;
                }
                "arm" => {
                    self.get::<SecSystem>()?
                        .arm()
                        .await
                        .map_err(|err| RouterError::HandlerError(err.into()))?;
                }
                "close blinds" => {
                    self.get::<BlindsService>()?
                        .close_both()
                        .await
                        .map_err(|err| RouterError::HandlerError(err.into()))?;
                    self.get::<SpeechService>()?
                        .say_cheerful(&format!("{ANNOUNCEMENT} All blinds are closing"))
                        .await
                        .map_err(|err| RouterError::HandlerError(err.into()))?;
                }
                "open blinds" => {
                    self.get::<BlindsService>()?
                        .open_both()
                        .await
                        .map_err(|err| RouterError::HandlerError(err.into()))?;
                    self.get::<SpeechService>()?
                        .say_cheerful(&format!("{ANNOUNCEMENT} All blinds are opening"))
                        .await
                        .map_err(|err| RouterError::HandlerError(err.into()))?;
                }
                message => {
                    let message = format!("Unknown command \"{}\"", message);
                    info!("{}", message);
                    self.get::<DiscordService>()?
                        .send_notification(message)
                        .await
                        .map_err(|err| RouterError::HandlerError(err.into()))?;
                }
            }
        }

        Ok(())
    }
}

/// Simplified representation of message for use over mqtt
#[derive(Debug, Clone, Deserialize)]
struct ReceivedDiscordMessage {
    message_id: u64,
    author_id: u64,
    is_author_bot: bool,
    channel_id: u64,
    content: String,
}
