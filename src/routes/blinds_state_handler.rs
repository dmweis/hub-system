use super::{Injected, IocContainer};
use crate::{discord_service::DiscordService, speech_service::SpeechService};
use async_trait::async_trait;
use log::*;
use mqtt_router::{RouteHandler, RouterError};
use serde::Deserialize;

#[derive(Debug)]
pub struct BlindsStateHandler {
    ioc: IocContainer,
}

impl BlindsStateHandler {
    pub fn new(ioc: IocContainer) -> Box<Self> {
        Box::new(Self { ioc })
    }
}

impl Injected for BlindsStateHandler {
    fn ioc(&self) -> &IocContainer {
        &self.ioc
    }
}

#[async_trait]
impl RouteHandler for BlindsStateHandler {
    async fn call(&mut self, topic: &str, content: &[u8]) -> std::result::Result<(), RouterError> {
        info!("Handling blinds state data from topic {}", topic);

        let room_name = topic
            .split('/')
            .next()
            .unwrap_or("unknown")
            .replace('_', " ");

        let blinds_data: BlindsStateUpdate =
            serde_json::from_slice(content).map_err(|err| RouterError::HandlerError(err.into()))?;

        let message = match blinds_data.state {
            BlindsState::Closed => Some(format!("{} blinds have been closed", room_name)),
            BlindsState::Open => Some(format!("{} blinds have been opened", room_name)),
            _ => None,
        };

        if let Some(message) = message {
            self.get::<SpeechService>()?
                .say_cheerful(&message)
                .await
                .map_err(|err| RouterError::HandlerError(err.into()))?;
            self.get::<DiscordService>()?
                .send_notification(message)
                .await
                .map_err(|err| RouterError::HandlerError(err.into()))?;
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct BlindsStateUpdate {
    pub state: BlindsState,
}

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum BlindsState {
    Open,
    Closed,
    Opening,
    Closing,
    Other,
}
