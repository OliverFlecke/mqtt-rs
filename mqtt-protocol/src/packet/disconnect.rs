use std::io::{self, Cursor, Write};

use crate::packet::{
	self, ControlPacketParseError, Decode, Encode, MqttControlPacket, MqttFixedHeader,
	kind::PacketType, property::Properties, reason::ReasonCode,
};

pub fn create_disconnect() -> MqttControlPacket {
	MqttControlPacket {
		header: MqttFixedHeader {
			kind: PacketType::Disconnect,
			remaining_length: 0,
		},
		variable_header: Some(packet::VariableHeader::Disconnect(VariableHeader {
			reason_code: ReasonCode::Success,
			properties: None,
		})),
		payload: None,
	}
}

#[derive(Debug, Clone)]
pub struct VariableHeader {
	reason_code: ReasonCode,
	properties: Option<Properties>,
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
