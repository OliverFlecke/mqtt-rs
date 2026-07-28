use crate::packet::{
	self, ControlPacketParseError, DecodeMqtt, EncodeMqtt, MqttControlPacket, MqttFixedHeader,
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

impl EncodeMqtt for VariableHeader {
	fn encode(&self, data: &mut Vec<u8>) {
		data.push(self.reason_code as u8);
		self.properties.encode(data);
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
