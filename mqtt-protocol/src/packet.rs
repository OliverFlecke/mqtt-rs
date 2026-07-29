mod connack;
mod connect;
mod disconnect;
pub mod kind;
mod ping;
mod property;
mod publish;
mod reason;
pub(crate) mod util;

use std::io::{self, Cursor, Write};

use crate::packet::kind::PacketType;

pub use connect::connect;
pub use disconnect::create_disconnect;
pub use ping::{create_ping_req, create_ping_resp};

/// Represents a single MQTT control packet, containing a fixed header,
/// and optionally a variable header and payload.
#[derive(Debug)]
pub struct MqttControlPacket {
	// Packet type and flags together make up the first byte.
	pub header: MqttFixedHeader,
	pub variable_header: Option<VariableHeader>,
	pub payload: Option<Payload>,
}

impl MqttControlPacket {
	/// Parse a packet from the given data.
	pub fn decode(data: &[u8]) -> Result<Self, ControlPacketParseError> {
		tracing::debug!("Parsing packet from bytes {:x?}", data);

		let header = MqttFixedHeader::try_decode(data)?;

		Ok(Self {
			variable_header: VariableHeader::decode(header.kind, &data[2..])?,
			payload: None, // TODO: Parse payload
			header,
		})
	}
}

impl Encode for MqttControlPacket {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		self.header.encode(w)?;
		self.variable_header.encode(w)?;
		self.payload.encode(w)?;

		// Update remaining length
		let pos = w.position();
		w.set_position(1);
		w.write_all(&[pos as u8 - 2])?;
		w.set_position(pos);

		Ok(())
	}
}

/// Trait for types that can be encoded into a byte vector following the MQTT
/// specification.
pub trait Encode {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()>;

	fn encode_to_vec(&self) -> io::Result<Vec<u8>> {
		let mut data = Cursor::new(Vec::new());
		self.encode(&mut data)?;
		Ok(data.into_inner())
	}
}

pub(crate) trait DecodeMqtt<T> {
	// TODO: we properly need the length of the decoded data here for further decoding.
	fn try_decode(data: &[u8]) -> Result<T, ControlPacketParseError>;
}

/// Represents the various variable headers that can be used in a packet.
#[derive(Debug, Clone)]
pub enum VariableHeader {
	Connect(connect::VariableHeader),
	ConnAck(connack::VariableHeader),
	Disconnect(disconnect::VariableHeader),
	Publish(publish::VariableHeader),
}

impl Encode for VariableHeader {
	fn encode(&self, data: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		match self {
			VariableHeader::Connect(connect) => connect.encode(data),
			VariableHeader::ConnAck(connack) => connack.encode(data),
			VariableHeader::Disconnect(disconnect) => disconnect.encode(data),
			VariableHeader::Publish(publish) => publish.encode(data),
		}
	}
}

impl Encode for Option<VariableHeader> {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		match self {
			Some(variable_header) => variable_header.encode(w),
			None => Ok(()),
		}
	}
}

impl VariableHeader {
	fn decode(kind: PacketType, data: &[u8]) -> Result<Option<Self>, ControlPacketParseError> {
		match kind {
			PacketType::Connect => Ok(Some(Self::Connect(connect::VariableHeader::try_decode(
				data,
			)?))),
			PacketType::ConnAck => Ok(Some(Self::ConnAck(connack::VariableHeader::try_decode(
				data,
			)?))),

			PacketType::PingReq | PacketType::PingResp => Ok(None),

			_ => unimplemented!("Decoding of {:?} is not yet supported", kind),
		}
	}
}

/// Payload for a packet.
///
/// This represents the various payloads that each packet can have. Note
/// that not all packet types has a payload.
#[derive(Debug, Clone)]
pub enum Payload {
	Connect(connect::Payload),
	Publish(publish::Payload),
}

impl Encode for Payload {
	fn encode(&self, data: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		match self {
			Payload::Connect(connect) => connect.encode(data),
			Payload::Publish(publish) => publish.encode(data),
		}
	}
}

impl Encode for Option<Payload> {
	fn encode(&self, data: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		match self {
			Some(payload) => payload.encode(data),
			None => Ok(()),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::FromRepr)]
#[repr(u8)]
pub enum ProtocolVersion {
	V5 = 5,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, strum::FromRepr)]
#[repr(u8)]
pub enum QoS {
	#[default]
	AtMostOnce = 0,
	AtLeastOnce = 1,
	ExactlyOnce = 2,
}

#[derive(Debug, Clone)]
pub struct MqttFixedHeader {
	pub kind: PacketType,
	pub remaining_length: u8,
}

impl MqttFixedHeader {
	pub fn new(kind: PacketType) -> Self {
		Self {
			kind,
			remaining_length: 0,
		}
	}
}

impl Encode for MqttFixedHeader {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		w.write_all(&[(self.kind as u8) << 4, self.remaining_length])
	}
}

impl DecodeMqtt<MqttFixedHeader> for MqttFixedHeader {
	fn try_decode(data: &[u8]) -> Result<Self, ControlPacketParseError> {
		if data.len() < 2 {
			return Err(ControlPacketParseError::NotEnoughData);
		}

		let kind = data[0] >> 4;

		Ok(Self {
			kind: PacketType::from_repr(kind)
				.ok_or(ControlPacketParseError::UnknownPacketType(kind))?,
			remaining_length: data[1],
		})
	}
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ControlPacketParseError {
	#[error("Unknown packet type {0:x}")]
	UnknownPacketType(u8),
	#[error("Unknown reason code {0:x}")]
	UnknownReasonCode(u8),
	#[error("Not enough data")]
	NotEnoughData,
	#[error("Unsupported QoS {0:x}")]
	UnsupportedQoS(u8),
	#[error("Incorrect protocol, must be MQTT")]
	IncorrectProtocol,
	#[error("Unsupported protocol version {0:x}")]
	UnsupportedProtocol(u8),
}
