use std::time::Duration;

use anyhow::Context;
use derive_builder::Builder;
use tokio::{
	io::{self, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
	net::{TcpStream, ToSocketAddrs},
	sync::{broadcast, mpsc, oneshot},
	time::sleep,
};
use tokio_util::{future::FutureExt, sync::CancellationToken};

use mqtt_protocol::packet::{
	Encode, MqttControlPacket, PublishQoS, QoS, ReasonCode, VariableHeader,
};
use tracing::instrument;

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
	session: Session,
}

impl MqttClient {
	/// Connect to an MQTT broker.
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

		let tx_health = writer_tx.clone();
		tokio::spawn(async { health_check(tx_health, Duration::from_secs(5)).await });

		// TODO: must have at least one subscriber to the reader, so this is kept
		// around for now. Secondly, this is needed to track the internal state
		// of the client, and reconnect the client if it disconnects.
		let sub = reader_tx.subscribe();
		tokio::spawn(async {
			let mut sub = sub;
			while let Ok(packet) = sub.recv().await {
				tracing::trace!(?packet, "Received packet");

				match packet.into() {
					(Some(VariableHeader::Disconnect(header)), _) => {
						tracing::info!(reason = ?header.reason_code(), "Disconnected");
					}
					(Some(VariableHeader::Subscribe(_)), _) => {}

					_ => {}
				}
			}
		});

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
	///
	/// This is a low-level API to send packet level messages. See `publish` for
	/// a higher-level API to publish messages.
	pub async fn send(&self, packet: MqttControlPacket) -> Result<(), ClientError> {
		self.tx.send(packet).await.map_err(ClientError::SendFailed)
	}

	/// Flush all packets to the broker.
	pub async fn flush(&self) -> Result<(), ClientError> {
		// TODO: need a way to flush the messages out through the client so the
		// bytes has actually been sent over the network.
		sleep(Duration::from_millis(300)).await;

		Ok(())
	}

	/// Subscribe to receive packets from the broker.
	pub fn subscribe(&self) -> broadcast::Receiver<MqttControlPacket> {
		self.rx.subscribe()
	}

	/// Publish a message to a topic
	#[instrument(skip(self), level = "debug", err)]
	pub async fn publish(
		&mut self,
		topic: String,
		payload: Vec<u8>,
		retain: bool,
		qos: QoS,
	) -> Result<(), ClientError> {
		match qos {
			QoS::AtMostOnce => self.publish_at_most_once(topic, payload, retain).await,
			QoS::AtLeastOnce => self.publish_at_least_once(topic, payload, retain).await,
			QoS::ExactlyOnce => self.publish_exactly_once(topic, payload, retain).await,
		}
	}

	/// Publish a message to a topic with at most once delivery.
	#[instrument(skip(self), level = "debug")]
	pub async fn publish_at_most_once(
		&self,
		topic: String,
		payload: Vec<u8>,
		retain: bool,
	) -> Result<(), ClientError> {
		tracing::debug!("Publishing packet");
		let packet =
			MqttControlPacket::publish(topic, payload, PublishQoS::AtMostOnce, retain, false);
		self.send(packet).await?;

		Ok(())
	}

	/// Publish a message with at most once delivery.
	#[instrument(skip(self), level = "debug")]
	pub async fn publish_at_least_once(
		&mut self,
		topic: String,
		payload: Vec<u8>,
		retain: bool,
	) -> Result<(), ClientError> {
		let packet_id = self.session.get_next_packet_id();
		let rx = self.listen_for_puback(packet_id);

		tracing::debug!(?packet_id, "Publishing packet");
		let packet = MqttControlPacket::publish(
			topic,
			payload,
			PublishQoS::AtLeastOnce(packet_id),
			retain,
			false,
		);
		self.send(packet).await?;

		// TODO: must continue to send until puback is received

		if let Err(err) = rx.await {
			tracing::error!(?err, "Error waiting for ack");
		}

		Ok(())
	}

	/// Listen for a puback packet with the given packet id.
	fn listen_for_puback(&self, packet_id: u16) -> oneshot::Receiver<()> {
		let mut sub = self.subscribe();
		let ct = self.cancellation_token().clone();
		let (tx, rx) = oneshot::channel::<()>();

		tokio::spawn(async move {
			while let Some(Ok(packet)) = sub.recv().with_cancellation_token(&ct).await {
				if let (Some(VariableHeader::PubAck(header)), _) = packet.into()
					&& header.packet_identifier == packet_id
				{
					tracing::debug!(?packet_id, "Received puback for packet");
					if let Err(err) = tx.send(()) {
						tracing::error!(?err, "Error sending ack");
					}

					break;
				}
			}
		});

		rx
	}

	/// Publish a message with exactly once delivery.
	pub async fn publish_exactly_once(
		&mut self,
		topic: String,
		payload: Vec<u8>,
		retain: bool,
	) -> Result<(), ClientError> {
		let packet_id = self.session.get_next_packet_id();

		tracing::debug!(?packet_id, "Publishing packet with exactly once delivery");
		let packet = MqttControlPacket::publish(
			topic,
			payload,
			PublishQoS::ExactlyOnce(packet_id),
			retain,
			false,
		);

		let mut sub = self.subscribe();
		// TODO: must continue to send until pubrec is received
		self.send(packet).await?;

		let ct = self.cancellation_token().clone();
		while let Some(Ok(packet)) = sub.recv().with_cancellation_token(&ct).await {
			match packet.into() {
				(Some(VariableHeader::PubRec(header)), _)
					if header.packet_identifier == packet_id =>
				{
					tracing::debug!(?packet_id, "QoS 2 - Received pubrec for packet");
					self.send(MqttControlPacket::pubrel(packet_id)).await?;
				}
				(Some(VariableHeader::PubComp(h)), _) if h.packet_identifier == packet_id => {
					tracing::debug!(?packet_id, "QoS 2 - Received pubcomp for packet");
					break;
				}
				_ => (),
			}
		}

		Ok(())
	}

	/// Spawn a task to read the data from the TCP socket. This will decode the
	/// data into MQTT control packets and send them to the internal queue for
	/// subscribers to handle.
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
						tracing::error!(?err, "Error parsing packet");
						continue;
					}
					Ok(packet) => packet,
				};
				if let Err(err) = tx.send(packet) {
					tracing::error!(
						receiver_count = tx.receiver_count(),
						?err,
						"Error sending packet to client queue"
					);
				};
			}

			tracing::trace!("Reader closed");
		});
		(tx, rx)
	}

	/// Spawn a task that will write packets to the TCP socket.
	fn spawn_writer(
		mut writer: WriteHalf<TcpStream>,
		ct: CancellationToken,
	) -> mpsc::Sender<MqttControlPacket> {
		let (tx, mut rx) = mpsc::channel::<MqttControlPacket>(4);
		tokio::spawn(async move {
			while let Some(packet) = rx.recv().with_cancellation_token(&ct).await.flatten() {
				tracing::debug!(kind = ?packet.kind(), "Sending packet");
				tracing::trace!("Sending packet: {:#?}", packet);

				let encoded = match packet.encode_to_vec() {
					Ok(encoded) => encoded,
					Err(err) => {
						tracing::error!("Error encoding packet: {:?}", err);
						continue;
					}
				};

				tracing::trace!(
					data = format!("{:2x?}", encoded),
					"Sending data over socket"
				);
				match writer.write_all(&encoded).await {
					Ok(()) => tracing::trace!("Packet sent"),
					Err(err) => tracing::error!("Error writing to socket: {:?}", err),
				};
			}

			tracing::debug!("Writer closed");
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
				let client_id = header
					.properties
					.to_owned()
					.and_then(|p| p.assigned_client_identifier)
					.or(client_id)
					.ok_or(ClientError::MissingClientId)?;

				Ok(Session::new(client_id))
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

async fn health_check(
	writer: mpsc::Sender<MqttControlPacket>,
	interval: Duration,
) -> Result<(), anyhow::Error> {
	loop {
		// TODO: this timer should reset ever time a packet is sent from the client,
		// to avoid sending a ping packet.
		sleep(interval).await;

		let packet = MqttControlPacket::create_ping_req();
		writer.send(packet).await?;
	}
}
