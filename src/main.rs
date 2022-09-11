extern crate core;
extern crate serde;

use std::time::Duration;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use rumqttc::Event::Incoming;
use rumqttc::Packet::Publish;
use serde::Deserialize;
use tokio::task;
use warp::Filter;

#[allow(dead_code)]
#[derive(Deserialize, Debug)]
struct ShellyAnnouncement {
    pub id: String,
    pub model: String,
    pub mac: String,
    pub ip: String,
    pub new_fw: bool,
    pub fw_ver: String,
}

#[tokio::main]
async fn main() {
    let host = std::env::var("MQTT_BROKER_HOST").expect("Missing MQTT_BROKER_HOST env var!");
    let mut options = MqttOptions::new("energy_monitor", host, 1883);
    options.set_keep_alive(Duration::from_secs(5));

    let mqtt_receiver = task::spawn(async {

        let (client, mut eventloop) = AsyncClient::new(options, 10);

        client.subscribe("shellies/announce", QoS::AtMostOnce).await.unwrap();

        while let Ok(notification) = eventloop.poll().await {
            match notification {
                Incoming(Publish(publish)) => {
                    let payload = String::from_utf8(publish.payload.to_vec()).unwrap();
                    match publish.topic.as_str() {
                        "shellies/announce" => {
                            let announcement = serde_json::from_str::<ShellyAnnouncement>(payload.as_str()).unwrap();
                            client.subscribe(format!("shellies/{}/online", announcement.id), QoS::AtMostOnce).await.unwrap();
                            client.subscribe(format!("shellies/{}/relay/+", announcement.id), QoS::AtMostOnce).await.unwrap();
                            client.subscribe(format!("shellies/{}/relay/+/power", announcement.id), QoS::AtMostOnce).await.unwrap();
                            client.subscribe(format!("shellies/{}/relay/+/energy", announcement.id), QoS::AtMostOnce).await.unwrap();
                            println!("{:?}", announcement);
                        }
                        _ => {
                            println!("{} = {}", publish.topic, payload);
                        }
                    }
                }
                _ => {}
            }
        }
    });

    let exporter = task::spawn(async {
        let hello = warp::path!("metrics")
            .map(|| "test_metrics{} 0\n");

        warp::serve(hello)
            .run(([0, 0, 0, 0], 9000))
            .await;
    });

    mqtt_receiver.await.expect("MQTT receiver failed!");
    exporter.await.expect("Exporter failed!");
}
