use mqtt_protocol::packet::MqttControlPacket;
use tokio_util::{future::FutureExt, sync::CancellationToken};

use crate::MqttClient;

impl MqttClient {
	pub fn on_message(&self, f: fn(MqttControlPacket)) -> CancellationToken {
		let mut rx = self.subscribe();
		let ct_client = self.cancellation_token().clone();

		let ct = CancellationToken::new();
		let ct_task = ct.clone();
		tokio::spawn(async move {
			while let Some(Ok(packet)) = rx
				.recv()
				.with_cancellation_token(&ct_task)
				.with_cancellation_token(&ct_client)
				.await
				.flatten()
			{
				f(packet)
			}
		});

		ct
	}
}
