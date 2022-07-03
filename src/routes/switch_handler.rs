use super::{Injected, ANNOUNCEMENT};
use crate::{
    blinds_service::BlindsService, discord_service::DiscordService, ioc::IocContainer,
    speech_service::SpeechService,
};
use async_trait::async_trait;
use log::*;
use mqtt_router::{RouteHandler, RouterError};
use serde::Deserialize;

pub struct SwitchHandler {
    ioc: IocContainer,
}

impl SwitchHandler {
    pub fn new(ioc: IocContainer) -> Box<Self> {
        Box::new(Self { ioc })
    }
}

impl Injected for SwitchHandler {
    fn ioc(&self) -> &IocContainer {
        &self.ioc
    }
}

#[async_trait]
impl RouteHandler for SwitchHandler {
    async fn call(&mut self, topic: &str, content: &[u8]) -> std::result::Result<(), RouterError> {
        info!("Handling switch data");
        let switch_name = topic.split('/').last().unwrap_or("unknown");
        let switch_data: SwitchPayload =
            serde_json::from_slice(content).map_err(|err| RouterError::HandlerError(err.into()))?;

        let message = match switch_data.action {
            Action::Single => Some(format!(
                "{ANNOUNCEMENT} Switch {switch_name} was clicked once"
            )),
            Action::Long => Some(format!(
                "{ANNOUNCEMENT} Switch {switch_name} was long pressed"
            )),
            Action::Double => None,
        };

        if let Some(message) = message {
            self.get::<SpeechService>()?
                .say_cheerful(&message)
                .await
                .unwrap();
        }

        if let Action::Double = switch_data.action {
            self.get::<BlindsService>()?.close_both().await.unwrap();
            self.get::<SpeechService>()?
                .say_cheerful(&format!("{ANNOUNCEMENT} All blinds are closing"))
                .await
                .unwrap();
            self.get::<DiscordService>()?
                .send_notification(format!("{ANNOUNCEMENT} All blinds are closing"))
                .await
                .unwrap();
        }

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Single,
    Double,
    Long,
}

#[derive(Debug, Deserialize)]
pub struct SwitchPayload {
    pub action: Action,
    #[allow(dead_code)]
    pub battery: f32,
    #[allow(dead_code)]
    pub linkquality: f32,
    #[allow(dead_code)]
    pub voltage: f32,
}
