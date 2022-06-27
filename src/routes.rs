use crate::{
    blinds_service::BlindsService, discord_service::DiscordService, ioc::IocContainer,
    speech_service::SpeechService,
};
use async_trait::async_trait;
use log::*;
use mqtt_router::{RouteHandler, RouterError};
use serde::Deserialize;
use std::{any::Any, sync::Arc};

trait Injected {
    fn ioc(&self) -> &IocContainer;

    fn get<T: Any + Send + Sync>(&self) -> std::result::Result<Arc<T>, RouterError> {
        // this is just playing
        // Do this correctly
        if let Some(service) = self.ioc().get() {
            Ok(service)
        } else {
            error!(
                "Service {} not available in {:?}",
                std::any::type_name::<T>(),
                self.ioc()
            );
            Err(RouterError::HandlerError("Service not available".into()))
        }
    }
}

const ANNOUNCEMENT: &str = "This is Hub system announcement.";

#[derive(Debug)]
pub struct MotionSensorHandler {
    ioc: IocContainer,
}

impl MotionSensorHandler {
    pub fn new(ioc: IocContainer) -> Box<Self> {
        Box::new(Self { ioc })
    }
}

impl Injected for MotionSensorHandler {
    fn ioc(&self) -> &IocContainer {
        &self.ioc
    }
}

#[async_trait]
impl RouteHandler for MotionSensorHandler {
    async fn call(
        &mut self,
        _topic: &str,
        _content: &[u8],
    ) -> std::result::Result<(), RouterError> {
        info!("Motion sensor silenced");
        Ok(())

        // let motion_sensor_id = topic.split('/').last().unwrap_or("Not available");
        // info!("Handling motion sensor data");
        // let motion_sensor: MotionSensorData =
        //     serde_json::from_slice(content).map_err(|err| RouterError::HandlerError(err.into()))?;

        // let message = if motion_sensor.occupancy {
        //     format!("{ANNOUNCEMENT} Motion sensor {motion_sensor_id} triggered")
        // } else {
        //     format!("{ANNOUNCEMENT} Motion sensor {motion_sensor_id} detects no movement")
        // };

        // self.get::<SpeechService>()?
        //     .say_cheerful(&message)
        //     .await
        //     .unwrap();

        // Ok(())
    }
}

#[derive(Debug, Deserialize)]
struct MotionSensorData {
    #[allow(dead_code)]
    pub battery: f32,
    #[allow(dead_code)]
    pub battery_low: bool,
    #[allow(dead_code)]
    pub linkquality: f32,
    pub occupancy: bool,
    #[allow(dead_code)]
    pub tamper: bool,
    #[allow(dead_code)]
    pub voltage: f32,
}

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
            Action::Single => format!("{ANNOUNCEMENT} Switch {switch_name} was clicked once"),
            Action::Long => format!("{ANNOUNCEMENT} Switch {switch_name} was long pressed"),
            Action::Double => format!("{ANNOUNCEMENT} Switch {switch_name} was double clicked"),
        };

        self.get::<SpeechService>()?
            .say_cheerful(&message)
            .await
            .unwrap();

        if let Action::Double = switch_data.action {
            self.get::<BlindsService>()?.close_both().await.unwrap();
            self.get::<SpeechService>()?
                .say_cheerful(&format!("{ANNOUNCEMENT} All blinds are closing"))
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
            let message = match received_message.content.to_ascii_lowercase().as_str() {
                "arm" => Some(String::from("Sec System is unavailable.")),
                "close blinds" => {
                    self.get::<BlindsService>()?.close_both().await.unwrap();
                    Some(format!("{ANNOUNCEMENT} All blinds are closing"))
                }
                "open blinds" => {
                    self.get::<BlindsService>()?.open_both().await.unwrap();
                    Some(format!("{ANNOUNCEMENT} All blinds are opening"))
                }
                _ => None,
            };
            if let Some(message) = message {
                self.get::<SpeechService>()?
                    .say_cheerful(&message)
                    .await
                    .unwrap();
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
