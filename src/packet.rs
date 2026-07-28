mod connact;
mod connect;
pub mod kind;
mod reason;

use crate::packet::{
	connect::{ConnectPayload, VariableHeaderConnect},
	kind::PacketType,
};

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
	pub fn parse(data: &[u8]) -> Result<Self, ControlPacketParseError> {
		tracing::debug!("Parsing packet from bytes {:x?}", data);

		let header = MqttFixedHeader::parse(data)?;

		Ok(Self {
			header,
			variable_header: None, // TODO: Parse variable header
			payload: None,
		})
	}

	pub fn encode(&self) -> Vec<u8> {
		// OPTIMIZE: can we pre-allocate the length of the vector?
		let mut data = self.header.encode_to_vec();

		if let Some(variable_header) = &self.variable_header {
			variable_header.encode(&mut data);
		}
		if let Some(payload) = &self.payload {
			payload.encode(&mut data);
		}

		// Update remaining length
		data[1] = data.len() as u8 - 2;

		data
	}

	pub fn connect(client_id: Option<String>) -> Self {
		Self {
			header: MqttFixedHeader {
				kind: PacketType::Connect,
				remaining_length: 0,
			},
			variable_header: Some(VariableHeader::Connect(VariableHeaderConnect::default())),
			payload: Some(Payload::Connect(ConnectPayload { client_id })),
		}
	}
}

/// Trait for types that can be encoded into a byte vector following the MQTT
/// specification.
pub(crate) trait EncodeMqtt {
	fn encode(&self, data: &mut Vec<u8>);

	fn encode_to_vec(&self) -> Vec<u8> {
		let mut data = Vec::new();
		self.encode(&mut data);
		data
	}
}

pub(crate) trait DecodeMqtt<T> {
	fn decode(data: &[u8]) -> Result<T, ControlPacketParseError>;
}

#[derive(Debug, Clone)]
pub enum VariableHeader {
	Connect(VariableHeaderConnect),
}

impl EncodeMqtt for VariableHeader {
	fn encode(&self, data: &mut Vec<u8>) {
		match self {
			VariableHeader::Connect(connect) => connect.encode(data),
		}
	}
}

#[derive(Debug, Clone)]
pub enum Payload {
	Connect(ConnectPayload),
}

impl EncodeMqtt for Payload {
	fn encode(&self, data: &mut Vec<u8>) {
		match self {
			Payload::Connect(connect) => connect.encode(data),
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::FromRepr)]
#[repr(u8)]
pub enum ProtocolVersion {
	V5 = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::FromRepr)]
#[repr(u8)]
pub enum WillQoS {
	AtMostOnce = 0,
	AtLeastOnce = 1,
	ExactlyOnce = 2,
}

#[derive(Debug, Clone)]
pub struct MqttFixedHeader {
	pub kind: PacketType,
	pub remaining_length: u8,
}

impl EncodeMqtt for MqttFixedHeader {
	fn encode(&self, data: &mut Vec<u8>) {
		data.push((self.kind as u8) << 4);
		data.push(self.remaining_length);
	}
}

impl MqttFixedHeader {
	pub fn parse(data: &[u8]) -> Result<Self, ControlPacketParseError> {
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
}
