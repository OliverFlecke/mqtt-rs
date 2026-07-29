mod connack;
mod connect;
mod disconnect;
pub(crate) mod fixed_header;
pub mod kind;
mod ping;
mod property;
mod publish;
mod reason;
mod subscribe;

use std::io::{self, Cursor, Write};

use crate::packet::{fixed_header::MqttFixedHeader, kind::PacketType};

/// Represents a single MQTT control packet, containing a fixed header,
/// and optionally a variable header and payload.
#[derive(Debug)]
pub struct MqttControlPacket {
	// Packet type and flags together make up the first byte.
	header: MqttFixedHeader,
	variable_header: Option<VariableHeader>,
	payload: Option<Payload>,
}

impl MqttControlPacket {
	#[inline]
	pub fn kind(&self) -> PacketType {
		self.header.kind()
	}

	pub fn header(&self) -> &Option<VariableHeader> {
		&self.variable_header
	}

	/// Parse a packet from the given data.
	pub fn decode(data: &[u8]) -> Result<Self, ControlPacketParseError> {
		tracing::trace!("Parsing packet from bytes {:x?}", data);

		let (header, data) = MqttFixedHeader::decode(data)?;
		let (variable_header, _data) = match VariableHeader::decode(header.kind(), data)? {
			Some((variable_header, rest)) => (Some(variable_header), rest),
			None => (None, data),
		};

		Ok(Self {
			header,
			variable_header,
			payload: None, // TODO: Parse payload
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

pub(crate) trait Decode<T> {
	fn decode(data: &[u8]) -> Result<(T, &[u8]), ControlPacketParseError>;
}

/// Represents the various variable headers that can be used in a packet.
#[derive(Debug, Clone)]
pub enum VariableHeader {
	Connect(connect::VariableHeader),
	ConnAck(connack::VariableHeader),
	Disconnect(disconnect::VariableHeader),
	Publish(publish::VariableHeader),
	Subscribe(subscribe::VariableHeader),
}

impl Encode for VariableHeader {
	fn encode(&self, data: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		match self {
			VariableHeader::Connect(connect) => connect.encode(data),
			VariableHeader::ConnAck(connack) => connack.encode(data),
			VariableHeader::Disconnect(disconnect) => disconnect.encode(data),
			VariableHeader::Publish(publish) => publish.encode(data),
			VariableHeader::Subscribe(subscribe) => subscribe.encode(data),
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
	fn decode(
		kind: PacketType,
		data: &[u8],
	) -> Result<Option<(Self, &[u8])>, ControlPacketParseError> {
		match kind {
			PacketType::Connect => {
				connect::VariableHeader::decode(data).map(|(h, d)| Some((Self::Connect(h), d)))
			}
			PacketType::ConnAck => {
				connack::VariableHeader::decode(data).map(|(h, d)| Some((Self::ConnAck(h), d)))
			}
			PacketType::Disconnect => disconnect::VariableHeader::decode(data)
				.map(|(h, d)| Some((Self::Disconnect(h), d))),

			PacketType::PingReq | PacketType::PingResp => Ok(None),

			_ => {
				tracing::warn!("Decoding of {:?} is not yet supported", kind);
				Ok(None)
			}
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
	Subscribe(subscribe::Payload),
}

impl Encode for Payload {
	fn encode(&self, data: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		match self {
			Payload::Connect(connect) => connect.encode(data),
			Payload::Publish(publish) => publish.encode(data),
			Payload::Subscribe(subscribe) => subscribe.encode(data),
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
	#[error("Invalid variable byte integer {0:x}")]
	InvalidVariableByteInteger(u32),
}
