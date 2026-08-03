use std::time::Duration;

use anyhow::Context;
use tokio::{
	io::{self, AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf},
	net::{TcpStream, ToSocketAddrs},
	sync::{broadcast, mpsc},
	time::sleep,
};
use tokio_util::{future::FutureExt, sync::CancellationToken};

use mqtt_protocol::packet::{
	Encode, MqttControlPacket, PublishOptions, PublishQoS, QoS, ReasonCode, Topic, TopicFilter,
	VariableHeader,
};
use tracing::instrument;

use crate::session::Session;

/// Options for connecting to a broker.
#[derive(Debug, Default)]
pub struct ConnectOptions {
	client_id: Option<String>,
	publish_retry_interval: Duration,
	health_check_interval: Duration,
}

/// Builder for connection options.
#[derive(Debug, Default)]
pub struct ConnectOptionsBuilder {
	client_id: Option<String>,
	publish_retry_interval: Option<Duration>,
	health_check_interval: Option<Duration>,
}

impl ConnectOptionsBuilder {
	pub fn client_id(mut self, client_id: Option<String>) -> Self {
		self.client_id = client_id;
		self
	}

	pub fn publish_retry_interval(mut self, interval: Duration) -> Self {
		self.publish_retry_interval = Some(interval);
		self
	}

	pub fn health_check_interval(mut self, interval: Duration) -> Self {
		self.health_check_interval = Some(interval);
		self
	}

	pub fn build(self) -> ConnectOptions {
		ConnectOptions {
			client_id: self.client_id,
			publish_retry_interval: self
				.publish_retry_interval
				.unwrap_or(Duration::from_millis(500)),
			health_check_interval: self
				.health_check_interval
				.unwrap_or(Duration::from_secs(30)),
		}
	}
}

/// A client that can send and receive MQTT packets.
#[derive(Debug)]
pub struct MqttClient {
	tx: mpsc::Sender<MqttControlPacket>,
	rx: broadcast::Sender<MqttControlPacket>,
	ct: CancellationToken,
	session: Session,
	publish_retry_interval: Duration,
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
		let health_check_interval = options.health_check_interval;
		let health_chec_ct = ct.clone();
		tokio::spawn(async move {
			health_check(tx_health, health_check_interval, health_chec_ct).await
		});

		// TODO: must have at least one subscriber to the reader, so this is kept
		// around for now. Secondly, this is needed to track the internal state
		// of the client, and reconnect the client if it disconnects.
		let sub = reader_tx.subscribe();
		let sub_tx = writer_tx.clone();
		tokio::spawn(async move {
			let mut sub = sub;
			while let Ok(packet) = sub.recv().await {
				tracing::trace!(?packet, "Received packet");

				let flags = packet.fixed_header().flags();
				match packet.into() {
					(Some(VariableHeader::Disconnect(header)), _) => {
						tracing::info!(reason = ?header.reason_code(), "Disconnected");
					}
					(Some(VariableHeader::Subscribe(_)), _) => {}
					(Some(VariableHeader::Publish(header)), _) => {
						let Ok(flags) = PublishOptions::try_from(flags) else {
							continue;
						};
						let Some(id) = header.packet_identifier() else {
							continue;
						};

						let packet = match flags.qos {
							QoS::AtMostOnce => continue,
							QoS::AtLeastOnce => MqttControlPacket::publish_acknowledged(id),
							QoS::ExactlyOnce => MqttControlPacket::publish_received(id),
						};

						if let Err(err) = sub_tx.send(packet).await {
							tracing::error!(?err, "Error sending puback");
						}
					}
					(Some(VariableHeader::PubRel(header)), _) => {
						if let Err(err) = sub_tx
							.send(MqttControlPacket::publish_complete(
								header.packet_identifier,
							))
							.await
						{
							tracing::error!(?err, "Error sending pubrel");
						}
					}

					_ => {}
				}
			}
		});

		Ok(MqttClient {
			tx: writer_tx,
			rx: reader_tx,
			ct,
			session,
			publish_retry_interval: options.publish_retry_interval,
		})
	}

	pub fn cancellation_token(&self) -> &CancellationToken {
		&self.ct
	}

	/// Disconnect from the broker.
	pub async fn disconnect(self) -> Result<(), ClientError> {
		tracing::debug!("Disconnecting");
		self.send_packet(MqttControlPacket::disconnect()).await?;
		self.ct.cancel();

		Ok(())
	}

	/// Send a packet to the broker.
	///
	/// This is a low-level API to send packet level messages. See `publish` for
	/// a higher-level API to publish messages.
	pub async fn send_packet(&self, packet: MqttControlPacket) -> Result<(), ClientError> {
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
	pub fn subscribe_for_packet(&self) -> broadcast::Receiver<MqttControlPacket> {
		self.rx.subscribe()
	}

	/// Subscribe this client to listen to a topic.
	pub async fn subscribe(&mut self, topic: TopicFilter) -> Result<(), ClientError> {
		let packet_id = self.session.get_next_packet_id();
		self.send_packet(MqttControlPacket::subscribe(packet_id, vec![topic]))
			.await?;

		self.wait_for_packet(|packet| {
			matches!(packet.into(), (Some(VariableHeader::SubAck(header)), _) if header.packet_id() == packet_id)
		})
		.await?;

		Ok(())
	}

	pub async fn unsubscribe(&mut self, topic: Topic) -> Result<(), ClientError> {
		let packet_id = self.session.get_next_packet_id();
		self.send_packet(MqttControlPacket::unsubscribe(packet_id, vec![topic]))
			.await?;

		self.wait_for_packet(|packet| {
			matches!(packet.into(), (Some(VariableHeader::UnsubAck(header)), _) if header.packet_id() == packet_id)
		})
		.await?;

		Ok(())
	}

	async fn wait_for_packet<F>(&mut self, predicate: F) -> Result<(), ClientError>
	where
		F: Fn(MqttControlPacket) -> bool,
	{
		loop {
			let packet = self
				.subscribe_for_packet()
				.recv()
				.with_cancellation_token(self.cancellation_token())
				.await
				.ok_or(ClientError::ReceiveFailed)?
				.map_err(|_| ClientError::ReceiveFailed)?;

			if predicate(packet) {
				break;
			}
		}

		Ok(())
	}

	/// Publish a message to a topic
	#[instrument(skip(self), level = "debug", err)]
	pub async fn publish(
		&mut self,
		topic: Topic,
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
		topic: Topic,
		payload: Vec<u8>,
		retain: bool,
	) -> Result<(), ClientError> {
		tracing::debug!("Publishing packet");
		let packet =
			MqttControlPacket::publish(topic, payload, PublishQoS::AtMostOnce, retain, false);
		self.send_packet(packet).await?;

		Ok(())
	}

	/// Publish a message with at most once delivery.
	#[instrument(skip(self), level = "debug")]
	pub async fn publish_at_least_once(
		&mut self,
		topic: Topic,
		payload: Vec<u8>,
		retain: bool,
	) -> Result<(), ClientError> {
		let packet_id = self.session.get_next_packet_id();
		tracing::debug!(?packet_id, "Publishing packet with at least once delivery");

		let mut sub = self.subscribe_for_packet();
		let sending_ct = self.publish_packet_repeat(move |count| {
			MqttControlPacket::publish(
				topic.clone(),
				payload.clone(),
				PublishQoS::AtLeastOnce(packet_id),
				retain,
				count != 0,
			)
		});

		while let Some(Ok(packet)) = sub
			.recv()
			.with_cancellation_token(self.cancellation_token())
			.await
		{
			if let (Some(VariableHeader::PubAck(header)), _) = packet.into()
				&& header.packet_identifier == packet_id
			{
				tracing::debug!(?packet_id, "Received puback for packet");
				sending_ct.cancel();

				break;
			}
		}

		Ok(())
	}

	/// Publish a message with exactly once delivery.
	pub async fn publish_exactly_once(
		&mut self,
		topic: Topic,
		payload: Vec<u8>,
		retain: bool,
	) -> Result<(), ClientError> {
		let packet_id = self.session.get_next_packet_id();

		tracing::debug!(?packet_id, "Publishing packet with exactly once delivery");

		let mut sub = self.subscribe_for_packet();
		let mut sending_ct = self.publish_packet_repeat(move |count| {
			MqttControlPacket::publish(
				topic.clone(),
				payload.clone(),
				PublishQoS::ExactlyOnce(packet_id),
				retain,
				count != 0,
			)
		});

		let ct = self.cancellation_token().clone();
		while let Some(Ok(packet)) = sub.recv().with_cancellation_token(&ct).await {
			match packet.into() {
				(Some(VariableHeader::PubRec(header)), _)
					if header.packet_identifier == packet_id =>
				{
					tracing::debug!(?packet_id, "QoS 2 - Received pubrec for packet");

					sending_ct.cancel();
					sending_ct = self.publish_packet_repeat(move |_| {
						MqttControlPacket::publish_release(packet_id)
					});
				}
				(Some(VariableHeader::PubComp(h)), _) if h.packet_identifier == packet_id => {
					tracing::debug!(?packet_id, "QoS 2 - Received pubcomp for packet");
					sending_ct.cancel();
					break;
				}
				_ => (),
			}
		}

		Ok(())
	}

	/// Spawn a task that will send a packet repeatedly until the returned
	/// cancellation token is canceled.
	///
	/// The packet is created by the provided closure, which takes the number
	/// of times packet has been published (starting from 0).
	fn publish_packet_repeat<F>(&self, create_packet: F) -> CancellationToken
	where
		F: Fn(u16) -> MqttControlPacket + Send + 'static,
	{
		let retry_interval = self.publish_retry_interval;
		let tx = self.tx.clone();

		let ct = CancellationToken::new();
		let task_ct = ct.clone();
		tokio::spawn(async move {
			let ct = task_ct;
			let mut count = 0;

			loop {
				let packet = create_packet(count);
				if let Err(err) = tx.send(packet.clone()).await {
					tracing::error!(?err, "Error sending packet");
				}

				sleep(retry_interval).with_cancellation_token(&ct).await;
				if ct.is_cancelled() {
					break;
				}

				count += 1;
			}
		});

		ct
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
			.variable_header()
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
	#[error("Failed to subscribe")]
	SubscribeFailed,
}

async fn health_check(
	writer: mpsc::Sender<MqttControlPacket>,
	interval: Duration,
	ct: CancellationToken,
) -> Result<(), anyhow::Error> {
	loop {
		// TODO: this timer should reset ever time a packet is sent from the client,
		// to avoid sending a ping packet.
		sleep(interval).with_cancellation_token(&ct).await;
		if ct.is_cancelled() {
			return Ok(());
		}

		let packet = MqttControlPacket::create_ping_req();
		writer.send(packet).await?;
	}
}
