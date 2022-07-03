use crate::routes::{
    BlindsStateHandler, DiscordHandler, DoorSensorHandler, MotionSensorHandler, RawJsonHandler,
    SwitchHandler,
};
use crate::{configuration::AppConfig, ioc::IocContainer};
use log::*;
use mqtt_router::Router;
use rumqttc::{AsyncClient, ConnAck, Event, Incoming, MqttOptions, Publish, QoS, SubscribeFilter};
use std::time::Duration;
use tokio::sync::mpsc::unbounded_channel;

enum MqttUpdate {
    Message(Publish),
    Reconnection(ConnAck),
}

pub fn start_mqtt_service(app_config: AppConfig, ioc: IocContainer) -> anyhow::Result<()> {
    let mut mqttoptions = MqttOptions::new(
        &app_config.mqtt.client_id,
        &app_config.mqtt.broker_host,
        app_config.mqtt.broker_port,
    );
    info!("Starting MQTT server with options {:?}", mqttoptions);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

    ioc.register(client.clone());

    let (message_sender, mut message_receiver) = unbounded_channel();

    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(notification) => match notification {
                    Event::Incoming(Incoming::Publish(publish)) => {
                        if let Err(e) = message_sender.send(MqttUpdate::Message(publish)) {
                            eprintln!("Error sending message {}", e);
                        }
                    }
                    Event::Incoming(Incoming::ConnAck(con_ack)) => {
                        if let Err(e) = message_sender.send(MqttUpdate::Reconnection(con_ack)) {
                            eprintln!("Error sending message {}", e);
                        }
                    }
                    _ => (),
                },
                Err(e) => {
                    eprintln!("Error processing eventloop notifications {}", e);
                }
            }
        }
    });

    let whole_sum_boi_base_topic = app_config.discord_bot.whole_sum_boi_base_topic;

    tokio::spawn(async move {
        let mut router = Router::default();

        router
            .add_handler("zigbee2mqtt/main_door", DoorSensorHandler::new(ioc.clone()))
            .unwrap();

        router
            .add_handler("zigbee2mqtt/switch/#", SwitchHandler::new(ioc.clone()))
            .unwrap();

        router
            .add_handler(
                "zigbee2mqtt/motion/#",
                MotionSensorHandler::new(ioc.clone()),
            )
            .unwrap();

        router
            .add_handler(
                &format!("{}/new_message/v1", whole_sum_boi_base_topic),
                DiscordHandler::new(ioc.clone()),
            )
            .unwrap();

        router
            .add_handler("+/blinds/state", BlindsStateHandler::new(ioc.clone()))
            .unwrap();

        // raw json

        let raw_json_handler = RawJsonHandler::new(ioc.clone());
        router
            .add_handler("zigbee2mqtt/motion/#", raw_json_handler.clone())
            .unwrap();
        router
            .add_handler("zigbee2mqtt/switch/#", raw_json_handler.clone())
            .unwrap();
        router
            .add_handler("zigbee2mqtt/main_door", raw_json_handler.clone())
            .unwrap();
        router
            .add_handler("+/blinds/state", raw_json_handler.clone())
            .unwrap();

        let topics = router
            .topics_for_subscription()
            .map(|topic| SubscribeFilter {
                path: topic.to_owned(),
                qos: QoS::AtMostOnce,
            });
        client.subscribe_many(topics).await.unwrap();

        loop {
            let update = message_receiver.recv().await.unwrap();
            match update {
                MqttUpdate::Message(message) => {
                    match router
                        .handle_message_ignore_errors(&message.topic, &message.payload)
                        .await
                    {
                        Ok(false) => error!("No handler for topic: \"{}\"", &message.topic),
                        Ok(true) => (),
                        Err(e) => error!("Failed running handler with {:?}", e),
                    }
                }
                MqttUpdate::Reconnection(_) => {
                    let topics = router
                        .topics_for_subscription()
                        .map(|topic| SubscribeFilter {
                            path: topic.to_owned(),
                            qos: QoS::AtMostOnce,
                        });
                    client.subscribe_many(topics).await.unwrap();
                }
            }
        }
    });

    Ok(())
}
