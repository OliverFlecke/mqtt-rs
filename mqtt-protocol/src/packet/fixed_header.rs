use std::io::{self, Cursor, Write};

use crate::packet::{ControlPacketParseError, Decode, Encode, kind::PacketType};

#[derive(Debug, Clone)]
pub struct MqttFixedHeader {
	kind: PacketType,
	remaining_length: u8,
}

impl MqttFixedHeader {
	#[inline]
	pub fn kind(&self) -> PacketType {
		self.kind
	}
}

impl From<PacketType> for MqttFixedHeader {
	fn from(kind: PacketType) -> Self {
		Self {
			kind,
			remaining_length: 0,
		}
	}
}

impl Encode for MqttFixedHeader {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		let reserved: u8 = match self.kind {
			PacketType::Subscribe => 0x2,
			_ => 0,
		};
		w.write_all(&[(self.kind as u8) << 4 | reserved, self.remaining_length])
	}
}

impl Decode<MqttFixedHeader> for MqttFixedHeader {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		if data.len() < 2 {
			return Err(ControlPacketParseError::NotEnoughData);
		}

		let kind = data[0] >> 4;
		let kind =
			PacketType::from_repr(kind).ok_or(ControlPacketParseError::UnknownPacketType(kind))?;

		Ok((
			Self {
				kind,
				remaining_length: data[1],
			},
			&data[2..],
		))
	}
}
