use super::Injected;
use crate::{discord_service::DiscordService, ioc::IocContainer};
use async_trait::async_trait;
use log::*;
use mqtt_router::{RouteHandler, RouterError};
use serde_json::Value;

#[derive(Debug, Clone)]
pub struct RawJsonHandler {
    ioc: IocContainer,
}

impl RawJsonHandler {
    pub fn new(ioc: IocContainer) -> Box<Self> {
        Box::new(Self { ioc })
    }
}

impl Injected for RawJsonHandler {
    fn ioc(&self) -> &IocContainer {
        &self.ioc
    }
}

#[async_trait]
impl RouteHandler for RawJsonHandler {
    async fn call(&mut self, topic: &str, content: &[u8]) -> std::result::Result<(), RouterError> {
        info!("Handling raw json data");
        let data: Value =
            serde_json::from_slice(content).map_err(|err| RouterError::HandlerError(err.into()))?;
        let json = serde_json::to_string_pretty(&data)
            .map_err(|err| RouterError::HandlerError(err.into()))?;

        let message = format!(
            "----------
_`{}`_ =>
```json
{}
```",
            topic, json
        );

        self.get::<DiscordService>()?
            .send_json(message)
            .await
            .map_err(|err| RouterError::HandlerError(err.into()))?;

        Ok(())
    }
}
