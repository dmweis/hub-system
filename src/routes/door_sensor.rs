use super::Injected;
use crate::{ioc::IocContainer, sec_system::SecSystem};
use async_trait::async_trait;
use log::*;
use mqtt_router::{RouteHandler, RouterError};
use serde::Deserialize;

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
    async fn call(&mut self, topic: &str, content: &[u8]) -> std::result::Result<(), RouterError> {
        info!("Handling door sensor data");
        let door_sensors_data: DoorSensorData =
            serde_json::from_slice(content).map_err(|err| RouterError::HandlerError(err.into()))?;

        let sec_system = self.get::<SecSystem>()?;
        sec_system
            .handle_door_sensor_data(&door_sensors_data, topic)
            .await
            .map_err(|err| RouterError::HandlerError(err.into()))?;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct DoorSensorData {
    #[allow(dead_code)]
    pub battery: f32,
    pub battery_low: bool,
    pub contact: bool,
    #[allow(dead_code)]
    pub linkquality: f32,
    pub tamper: bool,
    #[allow(dead_code)]
    pub voltage: f32,
}
