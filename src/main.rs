use anyhow::Context;
use mqtt_cli::packet::MqttControlPacket;
use tokio::{
	io::{self, AsyncReadExt, AsyncWriteExt},
	net::TcpStream,
	signal,
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

	let (mut reader, mut writer) = io::split(socket);

	let read_token = token.clone();
	let read_task = tokio::spawn(async move {
		let mut buf = [0; 1024];

		loop {
			let data = reader.read(&mut buf).await;
			match data {
				Ok(0) => {
					tracing::warn!("Received 0 bytes, likely protocol error.");
					read_token.cancel();
					break;
				}
				Ok(length) => {
					tracing::debug!("Received {} bytes", length);

					let packet = MqttControlPacket::decode(&buf[0..length]);
					match packet {
						Ok(packet) => tracing::debug!("Packet: {:#?}", packet),
						Err(err) => tracing::error!("Error parsing packet: {:?}", err),
					}
				}
				Err(err) => {
					tracing::error!("Error reading from socket: {:?}", err);
				}
			}
		}

		tracing::trace!("Reader closed");
		Ok::<_, anyhow::Error>(())
	});

	let write_task = tokio::spawn(async move {
		let packet = MqttControlPacket::connect(Some(String::from("alice")));
		let data = packet.encode();
		tracing::debug!("Encoded data (length: {}): {:2x?}", data.len(), data);

		let written = writer.write(&data).await?;
		writer.flush().await?;
		tracing::debug!("Wrote {} bytes", written);

		tokio::select! {
			_ = token.cancelled() => {}
			_ = signal::ctrl_c() => {
				tracing::debug!("Shutting down");
				writer.shutdown().await?;
				token.cancel();
			},
		}

		tracing::trace!("Writer closed");
		Ok::<_, anyhow::Error>(())
	});

	_ = tokio::try_join!(write_task, read_task);

	Ok(())
}
