use super::{Injected, ANNOUNCEMENT};
use crate::{discord_service::DiscordService, ioc::IocContainer, speech_service::SpeechService};
use async_trait::async_trait;
use log::*;
use mqtt_router::{RouteHandler, RouterError};
use serde::Deserialize;

pub struct DoorSensorHandler {
    ioc: IocContainer,
    door_contact: Option<bool>,
}

impl DoorSensorHandler {
    pub fn new(ioc: IocContainer) -> Box<Self> {
        Box::new(Self {
            ioc,
            door_contact: None,
        })
    }
}

impl Injected for DoorSensorHandler {
    fn ioc(&self) -> &IocContainer {
        &self.ioc
    }
}

#[async_trait]
impl RouteHandler for DoorSensorHandler {
    async fn call(&mut self, _topic: &str, content: &[u8]) -> std::result::Result<(), RouterError> {
        info!("Handling door sensor data");
        let door_sensors_data: DoorSensor =
            serde_json::from_slice(content).map_err(|err| RouterError::HandlerError(err.into()))?;

        let state_changed = match (self.door_contact, door_sensors_data.contact) {
            (Some(a), b) if a == b => false,
            (Some(_), _) => true,
            (None, _) => true,
        };
        self.door_contact = Some(door_sensors_data.contact);

        if !state_changed {
            info!("Ignoring door data because it didn't change");
            return Ok(());
        }

        let message = if door_sensors_data.contact {
            format!("{ANNOUNCEMENT} Front door closed")
        } else {
            format!("{ANNOUNCEMENT} Front door opened")
        };

        self.get::<SpeechService>()?
            .say_cheerful(&message)
            .await
            .map_err(|err| RouterError::HandlerError(err.into()))?;

        self.get::<DiscordService>()?
            .send_notification(message)
            .await
            .map_err(|err| RouterError::HandlerError(err.into()))?;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct DoorSensor {
    #[allow(dead_code)]
    pub battery: f32,
    #[allow(dead_code)]
    pub battery_low: bool,
    pub contact: bool,
    #[allow(dead_code)]
    pub linkquality: f32,
    #[allow(dead_code)]
    pub tamper: bool,
    #[allow(dead_code)]
    pub voltage: f32,
}
