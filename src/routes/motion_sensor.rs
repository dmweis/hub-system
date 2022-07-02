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
