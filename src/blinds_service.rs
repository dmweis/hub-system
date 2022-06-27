use anyhow::Result;
use rumqttc::AsyncClient;

#[derive(Debug, Clone)]
pub struct BlindsService {
    mqtt_client: AsyncClient,
}

impl BlindsService {
    pub fn new(mqtt_client: AsyncClient) -> Self {
        Self { mqtt_client }
    }

    pub async fn open_both(&self) -> Result<()> {
        self.open_bedroom().await?;
        self.open_living_room().await?;
        Ok(())
    }

    pub async fn close_both(&self) -> Result<()> {
        self.close_bedroom().await?;
        self.close_living_room().await?;
        Ok(())
    }

    pub async fn open_bedroom(&self) -> Result<()> {
        self.mqtt_client
            .publish(
                "bedroom/blinds/open",
                rumqttc::QoS::AtMostOnce,
                false,
                vec![],
            )
            .await?;
        Ok(())
    }

    pub async fn open_living_room(&self) -> Result<()> {
        self.mqtt_client
            .publish(
                "living_room/blinds/open",
                rumqttc::QoS::AtMostOnce,
                false,
                vec![],
            )
            .await?;
        Ok(())
    }

    pub async fn close_bedroom(&self) -> Result<()> {
        self.mqtt_client
            .publish(
                "bedroom/blinds/close",
                rumqttc::QoS::AtMostOnce,
                false,
                vec![],
            )
            .await?;
        Ok(())
    }

    pub async fn close_living_room(&self) -> Result<()> {
        self.mqtt_client
            .publish(
                "living_room/blinds/close",
                rumqttc::QoS::AtMostOnce,
                false,
                vec![],
            )
            .await?;
        Ok(())
    }
}
