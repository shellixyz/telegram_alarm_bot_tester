use rumqttc::{MqttOptions, Client, QoS, Outgoing, Event};
use serde_json::json;
use std::collections::HashMap;
use std::time::Duration;
use clap::{Parser,ValueEnum};

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum, Debug)]
enum Bool {
    True,
    False
}

impl Bool {
    fn to_bool(&self) -> bool {
        match self {
            Bool::True => true,
            Bool::False => false,
        }
    }
}

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(value_parser)]
    mqtt_topic: String,

    #[clap(value_parser)]
    payload_field_name: String,

    /// whether the payload should indicate the sensor has been triggered
    #[clap(arg_enum, value_parser)]
    payload_field_value: Bool,

    /// MQTT broker hostname
    #[clap(group = "mqtt", short, long, default_value_t = String::from("localhost"), value_parser)]
    hostname: String,

    /// MQTT broker port
    #[clap(group = "mqtt", short, long, default_value_t = 1883, value_parser = clap::value_parser!(u16).range(1..))]
    port: u16,

    /// send battery remaining percentage information in payload
    #[clap(short, long, value_parser = clap::value_parser!(u8).range(0..=100))]
    battery: Option<u8>,

    /// send voltage information in payload
    #[clap(short, long, value_parser = clap::value_parser!(u16).range(0..=4200))]
    voltage: Option<u16>,

    /// set sensor ID appearing in sensor name
    #[clap(short = 'n', long, default_value_t = 1, value_parser)]
    sensor_id: u16,
}

fn main() {
    let cli = Cli::parse();

    let mut mqttoptions = MqttOptions::new("rumqtt-sync", cli.hostname, cli.port);
    mqttoptions.set_keep_alive(Duration::from_secs(5));

    let (mut client, mut connection) = Client::new(mqttoptions, 10);

    let mut payload_data = HashMap::from([(cli.payload_field_name.as_str(), json!(cli.payload_field_value.to_bool()))]);

    if let Some(battery) = cli.battery {
        payload_data.insert("battery", json!(battery));
    }

    if let Some(voltage) = cli.voltage {
        payload_data.insert("voltage", json!(voltage));
    }

    let payload = serde_json::to_string(&payload_data).expect("failed to serialize sensor payload");

    client.publish(&cli.mqtt_topic, QoS::AtLeastOnce, false, payload).unwrap();

    for notification in connection.iter() {
        if let Ok(Event::Outgoing(Outgoing::Publish { .. })) = notification {
            break;
        }
    }
}
