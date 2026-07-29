use std::time::Duration;

use clap::Parser;
use mqtt_cli::{Cli, Command};
use mqtt_client::MqttClient;
use mqtt_protocol::packet::{MqttControlPacket, VariableHeader};
use tokio::{signal, time::sleep};
use tracing::level_filters::LevelFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing::subscriber::set_global_default(
		tracing_subscriber::fmt()
			.with_max_level(LevelFilter::DEBUG)
			.finish(),
	)?;

	let args = Cli::parse();

	tracing::debug!("Starting mqtt-cli");

	let client = MqttClient::connect(format!("{}:{}", args.host, args.port)).await?;
	let (tx, mut rx) = client.listen_and_wait()?;
	let reader = tokio::spawn(async move {
		while let Some(packet) = rx.recv().await {
			tracing::debug!("Packet received: {:x?}", packet.kind());
			if let Some(VariableHeader::Disconnect(header)) = packet.header() {
				tracing::info!("Disconnected with reason code: {:?}", header.reason_code());
			}
		}
	});

	match args.command {
		Command::Connect => {
			tx.send(MqttControlPacket::connect(args.client_id)).await?;
		}
		Command::Publish { topic, message } => {
			tx.send(MqttControlPacket::connect(args.client_id)).await?;
			tx.send(MqttControlPacket::create_publish(
				topic,
				message.into_bytes(),
			))
			.await?;
		}
		Command::Subscribe { topic } => {
			tracing::debug!("Subscribing to topic: {:?}", topic);
			tx.send(MqttControlPacket::connect(args.client_id)).await?;

			// TODO: we don't want to sleep here, but need to wait until the
			// connection is established
			sleep(Duration::from_millis(200)).await;

			tx.send(MqttControlPacket::subscribe(vec![topic.as_str().into()]))
				.await?;
		}
	}

	tokio::select! {
		_ = reader => {}
		_ = signal::ctrl_c() => {
			tx.send(MqttControlPacket::create_disconnect()).await?;

			tracing::debug!("Shutting down");
		},
	}

	Ok(())
}
