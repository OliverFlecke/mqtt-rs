use std::io::{self, Cursor, Write};

use crate::packet::{
	self, ControlPacketParseError, DecodeMqtt, Encode, MqttControlPacket, MqttFixedHeader,
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

impl DecodeMqtt<VariableHeader> for VariableHeader {
	fn try_decode(data: &[u8]) -> Result<Self, ControlPacketParseError> {
		Ok(Self {
			reason_code: ReasonCode::from_repr(data[0])
				.ok_or(ControlPacketParseError::UnknownReasonCode(data[0]))?,
			properties: Option::<Properties>::try_decode(&data[1..])?,
		})
	}
}
