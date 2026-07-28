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
		tokio::select! {
			data = reader.read(&mut buf) => {
				match data {
					Ok(0) => {
						tracing::warn!("Received 0 bytes, likely protocol error.");
						read_token.cancel();
					}
					Ok(length) => {
						tracing::debug!("Received {} bytes", length);

						let packet = MqttControlPacket::parse(&buf[0..length]);
						tracing::debug!("Packet: {:?}", packet);

					}
					Err(err) => {
						tracing::error!("Error reading from socket: {:?}", err);
					}
				}
			}
			_ = read_token.cancelled() => {}
		}

		tracing::debug!("Reader closed");
		Ok::<_, anyhow::Error>(())
	});

	let write_task = tokio::spawn(async move {
		// Trying to manually write a connect packet here, then refactor it
		let mut data: Vec<u8> = vec![
			0x10,
			0, // Fixed header
			// Variable header
			0x00,
			0x04, // Packet type, flags, remaining length
			b'M',
			b'Q',
			b'T',
			b'T',        // Protocol name
			0x05,        // Protocol version, byte 7
			0b0000_0010, // Connect flags
			0x00,
			0x3c, // Keep alive
			// Properties - don't think these are required
			// 0x05,
			// 0x11,
			// 0x00,
			// 0x00,
			// 0x00,
			// 0x0A,
			// Payload
			0,
			0,
			6,
		];
		data.extend_from_slice(b"oliver");
		data[1] = data.len() as u8 - 2;

		tracing::debug!("Writing data (length: {}): {:x?}", data.len(), data);

		let written = writer.write(&data).await?;
		writer.flush().await?;
		tracing::debug!("Wrote {} bytes", written);

		tokio::select! {
			_ = token.cancelled() => {}
			_ = signal::ctrl_c() => {
				tracing::debug!("Shutting down");
				token.cancel();
			},
		}

		tracing::debug!("Writer closed");
		Ok::<_, anyhow::Error>(())
	});

	_ = tokio::try_join!(write_task, read_task);

	Ok(())
}
