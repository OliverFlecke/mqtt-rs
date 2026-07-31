use mqtt_protocol::packet::{MqttControlPacket, PublishOptions, QoS};

use crate::{ClientError, MqttClient};

impl MqttClient {
	/// Publish a message to a topic
	pub async fn publish(
		&self,
		topic: String,
		payload: Vec<u8>,
		qos: QoS,
		retain: bool,
	) -> Result<(), ClientError> {
		let options = PublishOptions {
			qos,
			retain,
			..Default::default()
		};
		let packet = MqttControlPacket::publish(topic, payload, options);
		self.send(packet).await?;

		// TODO: If QoS > 0

		Ok(())
	}
}
