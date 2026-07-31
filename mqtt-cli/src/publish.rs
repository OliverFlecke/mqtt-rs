use std::time::Duration;

use mqtt_client::MqttClient;
use mqtt_protocol::packet::MqttControlPacket;
use tokio::time::sleep;

use crate::Publish;

pub async fn handler(client: MqttClient, args: Publish) -> anyhow::Result<()> {
	let packet = MqttControlPacket::publish(args.topic, args.message.into_bytes());

	loop {
		if let Err(err) = client.send(packet.clone()).await {
			tracing::error!("Error sending packet: {:?}", err);
			break;
		}
		tracing::info!("Message published");

		if let Some(repeat_frequency_ms) = args.repeat_frequency_ms {
			sleep(Duration::from_millis(repeat_frequency_ms)).await;
		} else {
			break;
		}
	}

	Ok(())
}
