use crate::sec_system::SecSystem;

use super::{Injected, IocContainer};
use async_trait::async_trait;
use log::*;
use mqtt_router::{RouteHandler, RouterError};
use serde::Deserialize;

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
        info!("Handling motion sensor data from topic {}", topic);
        let motion_sensor: MotionSensorData =
            serde_json::from_slice(content).map_err(|err| RouterError::HandlerError(err.into()))?;
        let sec_system = self.get::<SecSystem>()?;
        sec_system
            .handle_motion_sensor_data(&motion_sensor, topic)
            .await
            .map_err(|err| RouterError::HandlerError(err.into()))?;

        Ok(())
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct MotionSensorData {
    #[allow(dead_code)]
    pub battery: f32,
    pub battery_low: bool,
    #[allow(dead_code)]
    pub linkquality: f32,
    pub occupancy: bool,
    pub tamper: bool,
    #[allow(dead_code)]
    pub voltage: f32,
}
