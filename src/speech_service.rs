use anyhow::Result;
use rumqttc::AsyncClient;
use serde::Serialize;

use crate::ioc::IocContainer;

#[derive(Debug, Clone)]
pub struct SpeechService {
    ioc: IocContainer,
}

impl SpeechService {
    pub fn new(ioc: IocContainer) -> Self {
        Self { ioc }
    }

    pub async fn say(&self, message: &str, style: AzureVoiceStyle) -> Result<()> {
        let message = SayCommand {
            content: message.to_owned(),
            style,
            template: false,
        };
        let json = serde_json::to_string(&message)?;
        self.ioc
            .service::<AsyncClient>()?
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
