use anyhow::Result;
use rumqttc::AsyncClient;

use crate::ioc::IocContainer;

#[derive(Debug, Clone)]
pub struct BlindsService {
    ioc: IocContainer,
}

impl BlindsService {
    pub fn new(ioc: IocContainer) -> Self {
        Self { ioc }
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
        self.ioc
            .service::<AsyncClient>()?
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
        self.ioc
            .service::<AsyncClient>()?
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
        self.ioc
            .service::<AsyncClient>()?
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
        self.ioc
            .service::<AsyncClient>()?
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
