use mqtt_client::MqttClient;
use mqtt_protocol::packet::{MqttControlPacket, Payload, VariableHeader};
use tokio::signal;

use crate::Subscribe;

fn on_message(packet: MqttControlPacket) {
	tracing::debug!(kind = ?packet.kind(), "Packet received");

	match packet.into() {
		(Some(VariableHeader::Disconnect(header)), _) => {
			tracing::info!(reason = ?header.reason_code(), "Disconnected");
		}
		(Some(VariableHeader::SubAck(_)), _) => {
			// TODO: track the subscription and related topic to print
			// the topic that we have subscribed to.
			tracing::info!("Subscribed!");
		}
		(Some(VariableHeader::Publish(header)), Some(Payload::Publish(payload))) => {
			let msg: String = payload.try_into().expect("always to be valid UTF-8");
			tracing::info!(
				topic = header.topic(),
				msg,
				"Received message on topic: {:} -> {:?}",
				header.topic(),
				msg
			);
		}
		_ => (),
	}
}

pub async fn handler(client: MqttClient, args: Subscribe) -> anyhow::Result<()> {
	tracing::debug!(topic = ?args.topic, "Subscribing to topic: {:?}", args.topic);

	let ct = client.on_message(on_message);

	client
		.send_packet(MqttControlPacket::subscribe(vec![
			args.topic.as_str().into(),
		]))
		.await?;

	tokio::select! {
		_ = signal::ctrl_c() => {
			ct.cancel();
			tracing::debug!("Shutting down");
		},
	}

	Ok(())
}
