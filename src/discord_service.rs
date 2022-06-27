use anyhow::Result;
use rumqttc::AsyncClient;
use serde::Serialize;
use std::collections::HashMap;

use crate::configuration::DiscordBotConfig;

#[derive(Debug, Clone)]
pub struct DiscordService {
    mqtt_client: AsyncClient,
    whole_sum_boi_base_topic: String,
    notification_channel: u64,
    spam_channel: u64,
    channel_name_to_id: HashMap<String, u64>,
    channel_id_to_name: HashMap<u64, String>,
}

impl DiscordService {
    pub fn new(mqtt_client: AsyncClient, config: DiscordBotConfig) -> Self {
        let channel_id_to_name: HashMap<u64, String> = config
            .channels
            .iter()
            .map(|(name, id)| (*id, name.clone()))
            .collect();
        Self {
            mqtt_client,
            whole_sum_boi_base_topic: config.whole_sum_boi_base_topic,
            notification_channel: config.notification_discord_channel,
            spam_channel: config.spam_channel_id,
            channel_name_to_id: config.channels,
            channel_id_to_name,
        }
    }

    pub async fn send_notification(&self, content: String) -> Result<()> {
        let message = DiscordMessageToChannel {
            channel_id: self.notification_channel,
            content,
        };
        let json = serde_json::to_string(&message)?;
        let topic = format!("{}/say_channel", self.whole_sum_boi_base_topic);
        self.mqtt_client
            .publish(&topic, rumqttc::QoS::AtMostOnce, false, json)
            .await?;
        Ok(())
    }

    pub async fn send_spam(&self, content: String) -> Result<()> {
        let message = DiscordMessageToChannel {
            channel_id: self.spam_channel,
            content,
        };
        let json = serde_json::to_string(&message)?;
        let topic = format!("{}/say_channel", self.whole_sum_boi_base_topic);
        self.mqtt_client
            .publish(&topic, rumqttc::QoS::AtMostOnce, false, json)
            .await?;
        Ok(())
    }

    pub fn get_id_from_channel(&self, channel_name: &str) -> Option<u64> {
        self.channel_name_to_id.get(channel_name).cloned()
    }

    pub fn get_channel_name_from_id(&self, id: u64) -> Option<&String> {
        self.channel_id_to_name.get(&id)
    }

    pub fn notification_channel(&self) -> u64 {
        self.notification_channel
    }

    pub fn spam_channel(&self) -> u64 {
        self.spam_channel
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct DiscordMessageToChannel {
    channel_id: u64,
    content: String,
}
