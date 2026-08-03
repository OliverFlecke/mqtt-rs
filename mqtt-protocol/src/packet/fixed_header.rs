use std::io::{self, Cursor, Write};

use crate::{
	VariableByteInteger,
	packet::{ControlPacketParseError, Decode, Encode, kind::PacketType},
};

#[derive(Debug, Clone)]
pub struct MqttFixedHeader {
	kind: PacketType,
	flags: u8,
	remaining_length: VariableByteInteger,
}

impl MqttFixedHeader {
	pub fn new(kind: PacketType, flags: u8) -> Self {
		Self {
			kind,
			flags,
			remaining_length: VariableByteInteger::default(),
		}
	}

	#[inline]
	pub fn kind(&self) -> PacketType {
		self.kind
	}

	pub fn flags(&self) -> u8 {
		self.flags
	}

	pub fn set_length(&mut self, len: VariableByteInteger) {
		self.remaining_length = len;
	}
}

impl From<PacketType> for MqttFixedHeader {
	fn from(kind: PacketType) -> Self {
		Self {
			kind,
			flags: Default::default(),
			remaining_length: VariableByteInteger::default(),
		}
	}
}

impl Encode for MqttFixedHeader {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		w.write_all(&[(self.kind as u8) << 4 | (self.flags & 0xF)])?;
		// self.remaining_length.encode(w)?;
		Ok(())
	}
}

impl Decode<Self> for MqttFixedHeader {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		tracing::trace!(data = format!("{:2x?}", data), "Decoding fixed header");

		if data.len() < 2 {
			return Err(ControlPacketParseError::NotEnoughData);
		}

		let kind = data[0] >> 4;
		let kind =
			PacketType::from_repr(kind).ok_or(ControlPacketParseError::UnknownPacketType(kind))?;
		let flags = data[0] & 0xF;
		let (remaining_length, data) = VariableByteInteger::decode(&data[1..])?;

		tracing::debug!(?kind, ?flags, ?remaining_length, "Decoded fixed header");

		Ok((
			Self {
				kind,
				flags,
				remaining_length,
			},
			data,
		))
	}
}
