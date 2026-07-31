use std::time::Duration;

use anyhow::Context;
use derive_builder::Builder;
use tokio::{
	io::{self, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
	net::{TcpStream, ToSocketAddrs},
	sync::{broadcast, mpsc},
	time::sleep,
};
use tokio_util::{future::FutureExt, sync::CancellationToken};

use mqtt_protocol::packet::{Encode, MqttControlPacket, ReasonCode, VariableHeader};

use crate::session::Session;

#[derive(Debug, Default, Builder)]
pub struct ConnectOptions {
	client_id: Option<String>,
}

/// A client that can send and receive MQTT packets.
#[derive(Debug)]
pub struct MqttClient {
	tx: mpsc::Sender<MqttControlPacket>,
	rx: broadcast::Sender<MqttControlPacket>,
	ct: CancellationToken,
	#[allow(dead_code)]
	session: Session,
}

impl MqttClient {
	/// Connect to a MQTT broker.
	///
	/// This will open a TCP connection to the broker and send a connect packet,
	/// and wait for the connack before returning to ensure the connection is
	/// established.
	pub async fn connect<A>(address: A, options: ConnectOptions) -> Result<Self, ClientError>
	where
		A: ToSocketAddrs,
	{
		let socket = TcpStream::connect(address)
			.await
			.context("failed to connect")
			.map_err(|_| ClientError::ConnectFailed)?;
		let (reader, writer) = io::split(socket);
		let ct = CancellationToken::new();

		let (reader_tx, reader_rx) = Self::spawn_reader(reader, ct.clone());
		let writer_tx = Self::spawn_writer(writer, ct.clone());
		let session = Self::send_connect_and_wait(reader_rx, &writer_tx, options.client_id).await?;

		tracing::debug!(?session, "Connected");

		// let tx_health = tx_write.clone();
		// tokio::spawn(async { health_check(tx_health).await });

		Ok(MqttClient {
			tx: writer_tx,
			rx: reader_tx,
			ct,
			session,
		})
	}

	pub fn cancellation_token(&self) -> &CancellationToken {
		&self.ct
	}

	/// Disconnect from the broker.
	pub async fn disconnect(self) -> Result<(), ClientError> {
		tracing::debug!("Disconnecting");
		self.send(MqttControlPacket::disconnect()).await?;
		self.ct.cancel();

		Ok(())
	}

	/// Send a packet to the broker.
	pub async fn send(&self, packet: MqttControlPacket) -> Result<(), ClientError> {
		self.tx.send(packet).await.map_err(ClientError::SendFailed)
	}

	/// Subscribe to receive packets from the broker.
	pub fn subscribe(&self) -> broadcast::Receiver<MqttControlPacket> {
		self.rx.subscribe()
	}

	fn spawn_reader(
		mut reader: ReadHalf<TcpStream>,
		ct: CancellationToken,
	) -> (
		broadcast::Sender<MqttControlPacket>,
		broadcast::Receiver<MqttControlPacket>,
	) {
		let (tx, rx) = broadcast::channel::<MqttControlPacket>(4);
		let tx_task = tx.clone();
		tokio::spawn(async move {
			let tx = tx_task;
			let mut buf = [0; 1024 * 1024]; // TODO: make this configurable

			loop {
				let data = reader.read(&mut buf).with_cancellation_token(&ct).await;
				let Some(data) = data else {
					break;
				};

				let length = match data {
					Ok(0) => {
						tracing::warn!("Server disconnected");
						ct.cancel();
						break;
					}
					Err(err) => {
						tracing::error!("Error reading from socket: {:?}", err);
						continue;
					}
					Ok(length) => {
						tracing::trace!("Received {} bytes", length);
						length
					}
				};
				// TODO: should we clear the buffer afterward here?
				let packet = match MqttControlPacket::decode(&buf[0..length]) {
					Err(err) => {
						tracing::error!("Error parsing packet: {:?}", err);
						continue;
					}
					Ok(packet) => packet,
				};
				if let Err(err) = tx.send(packet) {
					tracing::error!("Error sending packet: {:?}", err)
				};
			}

			tracing::trace!("Reader closed");
		});
		(tx, rx)
	}

	/// Spawn a task that will write packets to the TPC socket.
	fn spawn_writer(
		mut writer: WriteHalf<TcpStream>,
		ct: CancellationToken,
	) -> mpsc::Sender<MqttControlPacket> {
		let (tx, mut rx) = mpsc::channel::<MqttControlPacket>(4);
		tokio::spawn(async move {
			while let Some(packet) = rx.recv().with_cancellation_token(&ct).await.flatten() {
				tracing::debug!("Sending packet type: {:x?}", packet.kind());
				tracing::trace!("Sending packet: {:#?}", packet);

				let encoded = match packet.encode_to_vec() {
					Ok(encoded) => encoded,
					Err(err) => {
						tracing::error!("Error encoding packet: {:?}", err);
						continue;
					}
				};

				match writer.write_all(&encoded).await {
					Ok(_) => tracing::trace!("Packet sent"),
					Err(err) => tracing::error!("Error writing to socket: {:?}", err),
				};
			}
			tracing::trace!("Writer closed");
		});

		tx
	}

	/// Send a connect packet and wait for the connack.
	async fn send_connect_and_wait(
		mut rx_read: broadcast::Receiver<MqttControlPacket>,
		tx_write: &mpsc::Sender<MqttControlPacket>,
		client_id: Option<String>,
	) -> Result<Session, ClientError> {
		tx_write
			.send(MqttControlPacket::connect(None, None, None))
			.await
			.map_err(ClientError::SendFailed)?;
		match rx_read
			.recv()
			.await
			.map_err(|_| ClientError::ReceiveFailed)?
			.header()
		{
			Some(VariableHeader::ConnAck(header)) if header.reason_code == ReasonCode::Success => {
				tracing::debug!("Connected");

				let session = Session {
					client_id: header
						.properties
						.to_owned()
						.and_then(|p| p.assigned_client_identifier)
						.or(client_id)
						.ok_or(ClientError::MissingClientId)?,
				};

				Ok(session)
			}
			_ => Err(ClientError::ConnectFailed),
		}
	}
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, thiserror::Error)]
pub enum ClientError {
	#[error("Failed to connect")]
	ConnectFailed,
	#[error("Failed to send packet: {0}")]
	SendFailed(#[source] mpsc::error::SendError<MqttControlPacket>),
	#[error("Failed to receive packet")]
	ReceiveFailed,
	#[error("Missing client id")]
	MissingClientId,
}

#[allow(dead_code)]
async fn health_check(writer: mpsc::Sender<MqttControlPacket>) -> Result<(), anyhow::Error> {
	loop {
		sleep(Duration::from_secs(5)).await;

		let packet = MqttControlPacket::create_ping_req();
		writer.send(packet).await?;
	}
}
