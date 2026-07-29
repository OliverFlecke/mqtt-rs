use std::io::{self, Cursor, Write};

use crate::packet::{
	self, ControlPacketParseError, Decode, Encode, MqttControlPacket, kind::PacketType,
	property::Properties, reason::ReasonCode,
};

impl MqttControlPacket {
	pub fn create_disconnect() -> Self {
		Self {
			header: PacketType::Disconnect.into(),
			variable_header: Some(packet::VariableHeader::Disconnect(VariableHeader {
				reason_code: ReasonCode::Success,
				properties: None,
			})),
			payload: None,
		}
	}
}

#[derive(Debug, Clone)]
pub struct VariableHeader {
	reason_code: ReasonCode,
	properties: Option<Properties>,
}

impl VariableHeader {
	pub fn reason_code(&self) -> ReasonCode {
		self.reason_code
	}

	pub fn properties(&self) -> &Option<Properties> {
		&self.properties
	}
}

impl Encode for VariableHeader {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		w.write_all(&[self.reason_code as u8])?;
		self.properties.encode(w)?;

		Ok(())
	}
}

impl Decode<VariableHeader> for VariableHeader {
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
