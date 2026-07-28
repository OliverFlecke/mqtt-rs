use crate::packet::{
	ControlPacketParseError, DecodeMqtt, EncodeMqtt, property::Properties, reason::ReasonCode,
};

#[derive(Debug, Clone)]
pub struct VariableHeader {
	pub session_present: bool,
	pub reason_code: ReasonCode,
	pub properties: Option<Properties>,
}

impl EncodeMqtt for VariableHeader {
	fn encode(&self, data: &mut Vec<u8>) {
		data.push(self.session_present as u8);
		data.push(self.reason_code as u8);
	}
}

impl DecodeMqtt<VariableHeader> for VariableHeader {
	fn decode(data: &[u8]) -> Result<Self, ControlPacketParseError> {
		Ok(Self {
			session_present: data[0] == 1,
			reason_code: ReasonCode::from_repr(data[1])
				.ok_or(ControlPacketParseError::UnknownReasonCode(data[1]))?,
			properties: Some(Properties::decode(&data[2..])?),
		})
	}
}
