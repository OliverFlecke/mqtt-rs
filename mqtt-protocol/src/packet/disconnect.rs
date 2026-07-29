use std::io::{self, Cursor, Write};

use crate::packet::{
	self, ControlPacketParseError, Decode, Encode, MqttControlPacket, kind::PacketType,
	property::Properties, reason::ReasonCode,
};

impl MqttControlPacket {
	pub fn disconnect() -> Self {
		Self::new(
			PacketType::Disconnect,
			Some(packet::VariableHeader::Disconnect(Header {
				reason_code: ReasonCode::Success,
				properties: None,
			})),
			None,
		)
	}
}

#[derive(Debug, Clone)]
pub struct Header {
	reason_code: ReasonCode,
	properties: Option<Properties>,
}

impl Header {
	pub fn reason_code(&self) -> ReasonCode {
		self.reason_code
	}

	pub fn properties(&self) -> &Option<Properties> {
		&self.properties
	}
}

impl Encode for Header {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		w.write_all(&[self.reason_code as u8])?;
		self.properties.encode(w)?;

		Ok(())
	}
}

impl Decode<Header> for Header {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		let reason_code = ReasonCode::from_repr(data[0])
			.ok_or(ControlPacketParseError::UnknownReasonCode(data[0]))?;
		let (properties, data) = Option::<Properties>::decode(&data[1..])?;

		Ok((
			Self {
				reason_code,
				properties,
			},
			data,
		))
	}
}
