use anyhow::Result;
use rumqttc::AsyncClient;
use serde::Serialize;

pub struct SpeechService {
    mqtt_client: AsyncClient,
}

impl SpeechService {
    pub fn new(mqtt_client: AsyncClient) -> Self {
        Self { mqtt_client }
    }

    pub async fn say(&self, message: &str, style: AzureVoiceStyle) -> Result<()> {
        let message = SayCommand {
            content: message.to_owned(),
            style,
            template: false,
        };
        let json = serde_json::to_string(&message)?;
        self.mqtt_client
            .publish("home_speak/say", rumqttc::QoS::AtMostOnce, false, json)
            .await?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize)]
struct SayCommand {
    content: String,
    style: AzureVoiceStyle,
    template: bool,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum AzureVoiceStyle {
    Plain,
    Angry,
    Cheerful,
    Sad,
}
