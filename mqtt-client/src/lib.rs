use std::time::Duration;

use anyhow::Context;
use tokio::{
	io::{self, AsyncRead, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
	net::{TcpStream, ToSocketAddrs},
	pin,
	sync::mpsc,
	time::sleep,
};
use tokio_util::sync::CancellationToken;

use mqtt_protocol::packet::{Encode, MqttControlPacket, create_ping_req};

pub struct MqttClient {
	cancellation_token: CancellationToken,
	reader: ReadHalf<TcpStream>,
	writer: WriteHalf<TcpStream>,
}

impl MqttClient {
	pub async fn connect<A>(address: A) -> anyhow::Result<Self>
	where
		A: ToSocketAddrs,
	{
		let socket = TcpStream::connect(address)
			.await
			.context("failed to connect")?;
		let (reader, writer) = io::split(socket);

		Ok(Self {
			cancellation_token: CancellationToken::new(),
			reader,
			writer,
		})
	}

	pub fn listen_and_wait(mut self) -> anyhow::Result<Client> {
		let (tx_read, rx_read) = mpsc::channel::<MqttControlPacket>(4);
		let read_token = self.cancellation_token.clone();
		tokio::spawn(async move {
			if let Err(err) = read(self.reader, tx_read, read_token).await {
				tracing::error!("Error reading from socket: {:?}", err);
			}
		});

		let (tx_write, mut rx_write) = mpsc::channel::<MqttControlPacket>(4);
		tokio::spawn(async move {
			while let Some(packet) = rx_write.recv().await {
				tracing::debug!("Sending packet type: {:x?}", packet.header.kind);
				tracing::trace!("Sending packet: {:#?}", packet);

				let encoded = match packet.encode_to_vec() {
					Ok(encoded) => encoded,
					Err(err) => {
						tracing::error!("Error encoding packet: {:?}", err);
						continue;
					}
				};

				match self.writer.write_all(&encoded).await {
					Ok(_) => {
						tracing::debug!("Packet sent");
					}
					Err(err) => {
						tracing::error!("Error writing to socket: {:?}", err);
					}
				};
			}
		});

		let tx_health = tx_write.clone();
		tokio::spawn(async { health_check(tx_health).await });

		Ok((tx_write, rx_read))
	}
}

type Client = (
	mpsc::Sender<MqttControlPacket>,
	mpsc::Receiver<MqttControlPacket>,
);

async fn health_check(writer: mpsc::Sender<MqttControlPacket>) -> Result<(), anyhow::Error> {
	loop {
		sleep(Duration::from_secs(5)).await;

		let packet = create_ping_req();
		writer.send(packet).await?;
	}
}

async fn read(
	reader: impl AsyncRead,
	tx: mpsc::Sender<MqttControlPacket>,
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
					Ok(packet) => {
						tx.send(packet).await?;
					}
					Err(err) => tracing::error!("Error parsing packet: {:?}", err),
				};
			}
			Err(err) => {
				tracing::error!("Error reading from socket: {:?}", err);
			}
		}
	}

	tracing::trace!("Reader closed");
	Ok(())
}
