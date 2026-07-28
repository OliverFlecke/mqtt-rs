use crate::packet_type::PacketType;

#[derive(Debug)]
pub struct MqttControlPacket {
	// Packet type and flags together make up the first byte.
	header: MqttFixedHeader,
}

impl MqttControlPacket {
	/// Parse a packet from the given data.
	pub fn parse(data: &[u8]) -> Result<Self, ControlPacketParseError> {
		tracing::debug!("Parsing packet from bytes {:x?}", data);

		let header = MqttFixedHeader::parse(data)?;

		Ok(Self { header })
	}
}

#[derive(Debug, Clone)]
pub struct MqttFixedHeader {
	kind: PacketType,
	remaining_length: u8,
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

#[derive(Debug, thiserror::Error)]
pub enum ControlPacketParseError {
	#[error("Unknown packet type {0:x}")]
	UnknownPacketType(u8),
	#[error("Not enough data")]
	NotEnoughData,
}
