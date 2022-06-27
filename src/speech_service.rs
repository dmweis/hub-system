use anyhow::Result;
use rumqttc::AsyncClient;
use serde::Serialize;

#[derive(Debug, Clone)]
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

    pub async fn say_plain(&self, message: &str) -> Result<()> {
        self.say(message, AzureVoiceStyle::Plain).await
    }

    pub async fn say_angry(&self, message: &str) -> Result<()> {
        self.say(message, AzureVoiceStyle::Angry).await
    }

    pub async fn say_cheerful(&self, message: &str) -> Result<()> {
        self.say(message, AzureVoiceStyle::Cheerful).await
    }

    pub async fn say_sad(&self, message: &str) -> Result<()> {
        self.say(message, AzureVoiceStyle::Sad).await
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
