use std::time::Duration;

use clap::Parser;
use mqtt_cli::{Cli, Command};
use mqtt_client::MqttClient;
use mqtt_protocol::packet::{MqttControlPacket, Payload, VariableHeader};
use tokio::{signal, time::sleep};
use tokio_util::future::FutureExt;
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

	let mut rx = client.subscribe();
	let ct = client.cancellation_token().clone();
	let reader = tokio::spawn(async move {
		while let Some(Ok(packet)) = rx.recv().with_cancellation_token(&ct).await {
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
			// Should we do anything here? The client would already be connected above
			// client
			// 	.send(MqttControlPacket::connect(args.client_id))
			// 	.await?;
		}
		Command::Publish {
			topic,
			message,
			repeat_frequency_ms,
		} => {
			let packet = MqttControlPacket::publish(topic, message.into_bytes());
			tokio::spawn(async move {
				loop {
					if let Err(err) = client.send(packet.clone()).await {
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
			client
				.send(MqttControlPacket::subscribe(vec![topic.as_str().into()]))
				.await?;
		}
	}

	tokio::select! {
		_ = reader => {}
		_ = signal::ctrl_c() => {
			tracing::debug!("Shutting down");
		},
	}

	// TODO: need a way to flush the messages out through the client so the
	// bytes has actually been sent over the network.

	Ok(())
}
