use std::time::Duration;

use clap::Parser;
use mqtt_cli::{Cli, Command};
use mqtt_client::MqttClient;
use mqtt_protocol::packet::{MqttControlPacket, Payload, VariableHeader};
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

			match packet.into() {
				(Some(VariableHeader::Disconnect(header)), _) => {
					tracing::info!("Disconnected with reason code: {:?}", header.reason_code());
				}
				(Some(VariableHeader::SubAck(_)), _) => {
					// TODO: track the subscription and related topic to print
					// the topic that we have subscribed to.
					tracing::info!("Subscribed!");
				}
				(Some(VariableHeader::Publish(header)), Some(Payload::Publish(payload))) => {
					let msg: String = payload.try_into().expect("always to be valid UTF-8");
					tracing::info!(
						"Received message on topic: {:} -> {:?}",
						header.topic(),
						msg
					);
				}
				_ => (),
			}
		}
	});

	match args.command {
		Command::Connect => {
			tx.send(MqttControlPacket::connect(args.client_id)).await?;
		}
		Command::Publish {
			topic,
			message,
			repeat_frequency_ms,
		} => {
			tx.send(MqttControlPacket::connect(args.client_id)).await?;

			let packet = MqttControlPacket::publish(topic, message.into_bytes());
			let tx_publish = tx.clone();
			tokio::spawn(async move {
				loop {
					if let Err(err) = tx_publish.send(packet.clone()).await {
						tracing::error!("Error sending packet: {:?}", err);
						break;
					}

					if let Some(repeat_frequency_ms) = repeat_frequency_ms {
						sleep(Duration::from_millis(repeat_frequency_ms)).await;
					} else {
						break;
					}
				}
			});
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
			tx.send(MqttControlPacket::disconnect()).await?;
			tracing::debug!("Shutting down");
		},
	}

	// TODO: need a way to flush the messages out through the client so the
	// bytes has actually been sent over the network.

	Ok(())
}
