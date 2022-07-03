mod discord_handler;
mod door_sensor;
mod motion_sensor;
mod raw_json_handler;
mod switch_handler;

pub use discord_handler::DiscordHandler;
pub use door_sensor::{DoorSensorData, DoorSensorHandler};
pub use motion_sensor::{MotionSensorData, MotionSensorHandler};
pub use raw_json_handler::RawJsonHandler;
pub use switch_handler::SwitchHandler;

use crate::ioc::IocContainer;
use log::*;
use mqtt_router::RouterError;
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
