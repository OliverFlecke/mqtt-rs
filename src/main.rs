use std::time::Duration;

use anyhow::Context;
use mqtt_cli::packet::{MqttControlPacket, connect, create_disconnect, create_ping_req};
use tokio::{
	io::{self, AsyncRead, AsyncReadExt, AsyncWriteExt},
	net::TcpStream,
	pin, signal,
	sync::mpsc::{self, Sender},
	time::sleep,
};
use tokio_util::sync::CancellationToken;
use tracing::level_filters::LevelFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing::subscriber::set_global_default(
		tracing_subscriber::fmt()
			.with_max_level(LevelFilter::DEBUG)
			.finish(),
	)?;

	tracing::debug!("Starting mqtt-cli");
	let token = CancellationToken::new();
	let socket = TcpStream::connect("127.0.0.1:1883")
		.await
		.context("Failed to connect")?;
	tracing::debug!("Socket connected {:?}", socket.local_addr()?.port());

	let (reader, mut writer) = io::split(socket);

	{
		let (tx, mut rx) = mpsc::channel::<MqttControlPacket>(4);
		let read_token = token.clone();
		tokio::spawn(async move {
			if let Err(err) = read(reader, tx, read_token).await {
				tracing::error!("Error reading from socket: {:?}", err);
			}
		});
		tokio::spawn(async move {
			while let Some(packet) = rx.recv().await {
				tracing::debug!("Packet received: {:x?}", packet.header.kind);
			}
		});
	}

	let (tx, mut rx) = mpsc::channel::<MqttControlPacket>(4);
	tokio::spawn(async move {
		while let Some(packet) = rx.recv().await {
			tracing::debug!("Sending packet type: {:x?}", packet.header.kind);
			tracing::trace!("Sending packet: {:#?}", packet);

			match writer.write_all(&packet.encode()).await {
				Ok(_) => {}
				Err(err) => {
					tracing::error!("Error writing to socket: {:?}", err);
				}
			};
		}
	});

	tx.send(connect(Some(String::from("alice")))).await?;
	sleep(Duration::from_millis(100)).await;

	tx.send(MqttControlPacket::create_publish(
		"test".to_string(),
		b"hello world".to_vec(),
	))
	.await?;

	tracing::trace!("Writer closed");

	tokio::select! {
		_ = health_check(tx.clone()) => {}
		_ = token.cancelled() => {}
		_ = signal::ctrl_c() => {
			tx.send(create_disconnect()).await?;

			tracing::debug!("Shutting down");
			token.cancel();
		},
	}

	// _ = tokio::try_join!(write_task, read_task, sender_task);

	Ok(())
}

async fn health_check(writer: Sender<MqttControlPacket>) -> Result<(), anyhow::Error> {
	loop {
		sleep(Duration::from_secs(5)).await;

		let packet = create_ping_req();
		writer.send(packet).await?;
	}
}

async fn read(
	reader: impl AsyncRead,
	tx: Sender<MqttControlPacket>,
	cancellation_token: CancellationToken,
) -> anyhow::Result<()> {
	pin!(reader);

	let mut buf = [0; 1024];
	loop {
		let data = reader.read(&mut buf).await;
		match data {
			Ok(0) => {
				tracing::warn!("Server disconnected");
				cancellation_token.cancel();
				break;
			}
			Ok(length) => {
				tracing::trace!("Received {} bytes", length);

				let packet = MqttControlPacket::decode(&buf[0..length]);
				match packet {
					Ok(packet) => tx.send(packet).await?,
					Err(err) => tracing::error!("Error parsing packet: {:?}", err),
				}
			}
			Err(err) => {
				tracing::error!("Error reading from socket: {:?}", err);
			}
		}
	}

	tracing::trace!("Reader closed");
	Ok(())
}
