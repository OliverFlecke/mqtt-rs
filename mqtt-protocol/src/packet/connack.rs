use std::io::{self, Cursor, Write};

use crate::packet::{
	ControlPacketParseError, DecodeMqtt, Encode, property::Properties, reason::ReasonCode,
};

#[derive(Debug, Clone)]
pub struct VariableHeader {
	pub session_present: bool,
	pub reason_code: ReasonCode,
	pub properties: Option<Properties>,
}

impl Encode for VariableHeader {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		w.write_all(&[self.session_present as u8, self.reason_code as u8])
	}
}

impl DecodeMqtt<VariableHeader> for VariableHeader {
	fn try_decode(data: &[u8]) -> Result<Self, ControlPacketParseError> {
		Ok(Self {
			session_present: data[0] == 1,
			reason_code: ReasonCode::from_repr(data[1])
				.ok_or(ControlPacketParseError::UnknownReasonCode(data[1]))?,
			properties: Option::<Properties>::try_decode(&data[2..])?,
		})
	}
}
