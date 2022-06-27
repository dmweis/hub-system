use async_trait::async_trait;
use log::*;
use mqtt_router::{RouteHandler, RouterError};
use serde::Deserialize;

#[derive(Debug)]
pub struct MotionSensorHandler {}

impl MotionSensorHandler {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

#[async_trait]
impl RouteHandler for MotionSensorHandler {
    async fn call(&mut self, _topic: &str, content: &[u8]) -> std::result::Result<(), RouterError> {
        info!("Handling motion sensor data");
        let motion_sensor: MotionSensorData =
            serde_json::from_slice(content).map_err(|err| RouterError::HandlerError(err.into()))?;

        let _message = if motion_sensor.occupancy {
            "Motion sensor triggered"
        } else {
            "Motion sensor detects no movement"
        };

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

pub struct DoorSensorHandler {}

impl DoorSensorHandler {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

#[async_trait]
impl RouteHandler for DoorSensorHandler {
    async fn call(&mut self, _topic: &str, content: &[u8]) -> std::result::Result<(), RouterError> {
        info!("Handling door sensor data");
        let motion_sensor: DoorSensor =
            serde_json::from_slice(content).map_err(|err| RouterError::HandlerError(err.into()))?;

        let _message = if motion_sensor.contact {
            "Front door closed"
        } else {
            "Front door opened"
        };

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

pub struct SwitchHandler {}

impl SwitchHandler {
    pub fn new() -> Box<Self> {
        Box::new(Self {})
    }
}

#[async_trait]
impl RouteHandler for SwitchHandler {
    async fn call(&mut self, topic: &str, content: &[u8]) -> std::result::Result<(), RouterError> {
        info!("Handling switch data");
        let switch_name = topic.split('/').last().unwrap_or("unknown");
        let switch_data: SwitchPayload =
            serde_json::from_slice(content).map_err(|err| RouterError::HandlerError(err.into()))?;

        let _message = match switch_data.action {
            Action::Single => format!("switch {switch_name} was clicked once"),
            Action::Long => format!("switch {switch_name} was long pressed"),
            Action::Double => format!("switch {switch_name} was double clicked"),
        };

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