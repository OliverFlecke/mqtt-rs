use std::time::Duration;

use mqtt_client::MqttClient;
use tokio::time::sleep;

use crate::Publish;

pub async fn handler(mut client: MqttClient, args: Publish) -> anyhow::Result<MqttClient> {
	loop {
		if let Err(err) = client
			.publish(
				args.topic.clone().into(),
				args.message.clone().into_bytes(),
				args.retain,
				args.qos.into(),
			)
			.await
		{
			tracing::error!("Error sending packet: {:?}", err);
			break;
		}
		client.flush().await?;
		println!("Published {} -> {}", args.topic, args.message);

		if let Some(repeat_frequency_ms) = args.repeat_frequency_ms {
			sleep(Duration::from_millis(repeat_frequency_ms)).await;
		} else {
			break;
		}
	}

	Ok(client)
}
