extern crate core;
extern crate serde;

use std::time::Duration;
use rumqttc::{Client, MqttOptions, QoS};
use rumqttc::Event::Incoming;
use rumqttc::Packet::Publish;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
struct ShellyAnnouncement {
    pub id: String,
    pub model: String,
    pub mac: String,
    pub ip: String,
    pub new_fw: bool,
    pub fw_ver: String,
}

fn main() {
    let host = std::env::var("MQTT_BROKER_HOST").expect("Missing MQTT_BROKER_HOST env var!");
    let mut options = MqttOptions::new("energy_monitor", host, 1883);
    options.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut connection) = Client::new(options, 10);

    client.subscribe("shellies/announce", QoS::AtMostOnce).unwrap();

    for (_, notification) in connection.iter().enumerate() {
        match notification {
            Ok(Incoming(Publish(publish))) => {
                let payload = String::from_utf8(publish.payload.to_vec()).unwrap();
                match publish.topic.as_str() {
                    "shellies/announce" => {
                        let announcement = serde_json::from_str::<ShellyAnnouncement>(payload.as_str()).unwrap();
                        client.subscribe(format!("shellies/{}/online", announcement.id), QoS::AtMostOnce).unwrap();
                        client.subscribe(format!("shellies/{}/relay/+", announcement.id), QoS::AtMostOnce).unwrap();
                        client.subscribe(format!("shellies/{}/relay/+/power", announcement.id), QoS::AtMostOnce).unwrap();
                        client.subscribe(format!("shellies/{}/relay/+/energy", announcement.id), QoS::AtMostOnce).unwrap();
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
}
