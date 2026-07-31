use std::time::Duration;

use mqtt_client::MqttClient;
use tokio::time::sleep;

use crate::Publish;

pub async fn handler(client: MqttClient, args: Publish) -> anyhow::Result<()> {
	loop {
		if let Err(err) = client
			.publish(
				args.topic.clone(),
				args.message.clone().into_bytes(),
				args.qos.into(),
				args.retain,
			)
			.await
		{
			tracing::error!("Error sending packet: {:?}", err);
			break;
		}
		client.flush().await?;
		tracing::info!("Message published");

		if let Some(repeat_frequency_ms) = args.repeat_frequency_ms {
			sleep(Duration::from_millis(repeat_frequency_ms)).await;
		} else {
			break;
		}
	}

	Ok(())
}
