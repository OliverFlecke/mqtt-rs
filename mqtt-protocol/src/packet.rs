mod connack;
mod connect;
mod control_packet;
mod disconnect;
pub(crate) mod fixed_header;
pub mod kind;
mod payload;
mod ping;
mod property;
mod protocol_version;
mod puback;
mod publish;
mod qos;
mod reason;
mod suback;
mod subscribe;
mod variable_header;

use std::{
	io::{self, Cursor},
	str::Utf8Error,
};

pub use control_packet::MqttControlPacket;
pub use kind::PacketType;
pub use payload::Payload;
pub use protocol_version::ProtocolVersion;
pub use publish::PublishOptions;
pub use qos::QoS;
pub use reason::ReasonCode;
pub use variable_header::VariableHeader;

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

pub(crate) trait DecodeFromType<T> {
	fn decode_from_type(
		kind: PacketType,
		data: &[u8],
	) -> Result<(Option<T>, &[u8]), ControlPacketParseError>;
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
	#[error("Invalid UTF-8: {0}")]
	InvalidUtf8(Utf8Error),
	#[error("Variable Byte Integer is more than 4 bytes")]
	InvalidVariableByteIntegerLength,
	#[error("Unknown property {0:x}")]
	UnknownProperty(u8),
}
