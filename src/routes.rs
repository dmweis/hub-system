use crate::{ioc::IocContainer, speech_service::SpeechService};
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
        self.ioc()
            .get()
            .ok_or_else(|| RouterError::HandlerError("Speech service not available".into()))
    }
}

const ANNOUNCEMENT: &str = "This is Hub system announcement. ";

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
    async fn call(&mut self, topic: &str, content: &[u8]) -> std::result::Result<(), RouterError> {
        let motion_sensor_id = topic.split('/').last().unwrap_or("Not available");
        info!("Handling motion sensor data");
        let motion_sensor: MotionSensorData =
            serde_json::from_slice(content).map_err(|err| RouterError::HandlerError(err.into()))?;

        let message = if motion_sensor.occupancy {
            format!("{ANNOUNCEMENT} Motion sensor {motion_sensor_id} triggered")
        } else {
            format!("{ANNOUNCEMENT} Motion sensor {motion_sensor_id} detects no movement")
        };

        self.get::<SpeechService>()?
            .say_cheerful(&message)
            .await
            .unwrap();

        Ok(())
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
}

impl DoorSensorHandler {
    pub fn new(ioc: IocContainer) -> Box<Self> {
        Box::new(Self { ioc })
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
        let motion_sensor: DoorSensor =
            serde_json::from_slice(content).map_err(|err| RouterError::HandlerError(err.into()))?;

        let message = if motion_sensor.contact {
            format!("{ANNOUNCEMENT} Front door closed")
        } else {
            format!("{ANNOUNCEMENT} Front door opened")
        };

        self.get::<SpeechService>()?
            .say_cheerful(&message)
            .await
            .unwrap();

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
