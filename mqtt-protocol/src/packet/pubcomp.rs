/// QoS 2 delivery part 3
use std::io::Cursor;

use crate::packet::{
	ControlPacketParseError, Decode, Encode, MqttControlPacket, PacketType, ReasonCode,
	VariableHeader, property::Properties,
};

impl MqttControlPacket {
	/// Create a control packet to acknowledge a publish packet.
	pub fn publish_complete(packet_id: u16) -> Self {
		Self::new(
			PacketType::PubComp,
			Some(VariableHeader::PubComp(Header {
				packet_identifier: packet_id,
				reason_code: ReasonCode::Success,
				properties: None,
			})),
			None,
		)
	}
}

#[derive(Debug, Clone)]
pub struct Header {
	pub packet_identifier: u16,
	pub reason_code: ReasonCode,
	pub properties: Option<Properties>,
}

impl Encode for Header {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> std::io::Result<()> {
		self.packet_identifier.encode(w)?;
		self.reason_code.encode(w)?;
		self.properties.encode(w)?;

		Ok(())
	}
}

impl Decode<Self> for Header {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		let (packet_identifier, data) = u16::decode(data)?;

		let (reason_code, data) = if data.is_empty() {
			(ReasonCode::Success, data)
		} else {
			ReasonCode::decode(data)?
		};

		let (properties, data) = if data.is_empty() {
			(None, data)
		} else {
			Option::<Properties>::decode(data)?
		};

		Ok((
			Self {
				packet_identifier,
				reason_code,
				properties,
			},
			data,
		))
	}
}
