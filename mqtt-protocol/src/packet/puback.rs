use std::io::{self, Cursor};

use crate::packet::{self, ControlPacketParseError, Decode, Encode, MqttControlPacket, PacketType};

impl MqttControlPacket {
	/// Create a control packet to acknowledge a publish packet.
	pub fn puback(packet_id: u16) -> Self {
		Self::new(
			PacketType::PubAck,
			Some(packet::VariableHeader::PubAck(Header {
				packet_identifier: packet_id,
			})),
			None,
		)
	}
}

/// Represents the header of a puback packet.
#[derive(Debug, Clone)]
pub struct Header {
	pub packet_identifier: u16,
}

impl Encode for Header {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		self.packet_identifier.encode(w)?;

		Ok(())
	}
}

impl Decode<Self> for Header {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		let (packet_identifier, data) = u16::decode(data)?;

		Ok((Self { packet_identifier }, data))
	}
}
