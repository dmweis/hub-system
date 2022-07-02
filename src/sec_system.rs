use std::{collections::HashMap, sync::Mutex};

use crate::{
    discord_service::DiscordService, ioc::IocContainer, routes::MotionSensorData,
    speech_service::SpeechService,
};
use anyhow::Result;
use log::*;

const ANNOUNCEMENT_PREAMBLE: &str = "Security System announcement.";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SecSystemState {
    Disarmed,
    Armed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MotionSensorState {
    Occupied,
    Unoccupied,
}

impl MotionSensorState {
    fn from_data(sensor_data: &MotionSensorData) -> Self {
        if sensor_data.occupancy {
            MotionSensorState::Occupied
        } else {
            MotionSensorState::Unoccupied
        }
    }
}

pub struct SecSystem {
    ioc: IocContainer,
    state: Mutex<SecSystemState>,
    motion_sensor_state: Mutex<HashMap<String, MotionSensorState>>,
}

impl SecSystem {
    pub fn new(ioc: IocContainer) -> Self {
        Self {
            ioc,
            state: Mutex::new(SecSystemState::Disarmed),
            motion_sensor_state: Mutex::new(HashMap::new()),
        }
    }

    fn clear_motion_data(&self) {
        let mut motion_sensor_state = self.motion_sensor_state.lock().unwrap();
        for value in motion_sensor_state.values_mut() {
            *value = MotionSensorState::Unoccupied;
        }
    }

    pub async fn arm(&self) -> Result<()> {
        info!("arming sec system");
        {
            let mut state_handle = self.state.lock().unwrap();
            if *state_handle == SecSystemState::Armed {
                error!("Sec system already armed");
                return Ok(());
            }
            *state_handle = SecSystemState::Armed;
        }

        const MESSAGE: &str = "Arming Security System";

        self.clear_motion_data();

        self.ioc
            .service::<SpeechService>()?
            .say_angry(MESSAGE)
            .await?;
        self.ioc
            .service::<DiscordService>()?
            .send_notification(MESSAGE.to_owned())
            .await?;

        Ok(())
    }

    pub async fn disarm(&self) -> Result<()> {
        info!("disarming sec system");
        {
            let mut state_handle = self.state.lock().unwrap();
            if *state_handle == SecSystemState::Disarmed {
                error!("Sec system already disarmed");
                return Ok(());
            }
            *state_handle = SecSystemState::Disarmed;
        }

        const MESSAGE: &str = "Disarming Security System";

        self.ioc
            .service::<SpeechService>()?
            .say_cheerful(MESSAGE)
            .await?;
        self.ioc
            .service::<DiscordService>()?
            .send_notification(MESSAGE.to_owned())
            .await?;

        Ok(())
    }

    pub async fn handle_motion_sensor_data(
        &self,
        sensor_data: &MotionSensorData,
        sensor_topic: &str,
    ) -> Result<()> {
        let new_state = MotionSensorState::from_data(sensor_data);

        let previous_state = self
            .motion_sensor_state
            .lock()
            .unwrap()
            .insert(sensor_topic.to_owned(), new_state)
            .unwrap_or(MotionSensorState::Unoccupied);

        let sensors_id = sensor_topic.split('/').last().unwrap_or("unknown");

        info!(
            "Motion sensor \"{}\" old state: {:?} new state: {:?}",
            sensors_id, previous_state, new_state
        );

        if sensor_data.battery_low {
            let message = format!(
                "{} Batter low in motion sensor \"{}\"",
                ANNOUNCEMENT_PREAMBLE, sensors_id
            );
            self.ioc
                .service::<SpeechService>()?
                .say_plain(&message)
                .await?;
            self.ioc
                .service::<DiscordService>()?
                .send_notification(message)
                .await?;
        }

        if sensor_data.tamper {
            let message = format!(
                "{} tampering detected in motion sensor \"{}\"",
                ANNOUNCEMENT_PREAMBLE, sensors_id
            );
            self.ioc
                .service::<SpeechService>()?
                .say_angry(&message)
                .await?;
            self.ioc
                .service::<DiscordService>()?
                .send_notification(message)
                .await?;
        }

        if *self.state.lock().unwrap() == SecSystemState::Armed {
            match (previous_state, new_state) {
                (MotionSensorState::Unoccupied, MotionSensorState::Unoccupied) => (),
                (MotionSensorState::Unoccupied, MotionSensorState::Occupied) => {
                    info!("Handling motion sensor transition to occupied");
                    let message = format!(
                        "{} Motion detected from sensor \"{}\"",
                        ANNOUNCEMENT_PREAMBLE, sensors_id
                    );
                    self.ioc
                        .service::<SpeechService>()?
                        .say_angry(&message)
                        .await?;
                    self.ioc
                        .service::<DiscordService>()?
                        .send_notification(message)
                        .await?;
                }
                (MotionSensorState::Occupied, MotionSensorState::Unoccupied) => {
                    let message = format!(
                        "{} Motion sensor \"{}\" cleared",
                        ANNOUNCEMENT_PREAMBLE, sensors_id
                    );
                    self.ioc
                        .service::<SpeechService>()?
                        .say_plain(&message)
                        .await?;
                    self.ioc
                        .service::<DiscordService>()?
                        .send_notification(message)
                        .await?;
                }
                (old, new) => info!("Unhandled motion sensors transition {:?} => {:?}", old, new),
            }
        } else {
            info!("Skipping motion sensor data in disarmed state");
        }

        Ok(())
    }
}
