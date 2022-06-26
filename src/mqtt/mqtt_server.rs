use crate::configuration::AppConfig;

use super::routes::{DoorSensorHandler, MotionSensorHandler, SwitchHandler};
use log::*;
use mqtt_router::Router;
use rumqttc::{AsyncClient, ConnAck, Event, Incoming, MqttOptions, Publish, QoS, SubscribeFilter};
use std::time::Duration;
use tokio::sync::mpsc::unbounded_channel;

enum MqttUpdate {
    Message(Publish),
    Reconnection(ConnAck),
}

pub fn start_mqtt_service(app_config: AppConfig) -> anyhow::Result<AsyncClient> {
    let mut mqttoptions = MqttOptions::new(
        &app_config.mqtt.client_id,
        &app_config.mqtt.broker_host,
        app_config.mqtt.broker_port,
    );
    info!("Starting MQTT server with options {:?}", mqttoptions);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);

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

    tokio::spawn({
        let client = client.clone();
        async move {
            let mut router = Router::default();

            router
                .add_handler("zigbee2mqtt/main_door", DoorSensorHandler::new())
                .unwrap();

            router
                .add_handler("zigbee2mqtt/switch/#", SwitchHandler::new())
                .unwrap();

            router
                .add_handler("zigbee2mqtt/motion/#", MotionSensorHandler::new())
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
                        let topics =
                            router
                                .topics_for_subscription()
                                .map(|topic| SubscribeFilter {
                                    path: topic.to_owned(),
                                    qos: QoS::AtMostOnce,
                                });
                        client.subscribe_many(topics).await.unwrap();
                    }
                }
            }
        }
    });

    Ok(client)
}
