use std::io::{self, Cursor, Write};

use crate::packet::{
	ControlPacketParseError, Decode, Encode, property::Properties, reason::ReasonCode,
};

#[derive(Debug, Clone)]
pub struct Header {
	pub session_present: bool,
	pub reason_code: ReasonCode,
	pub properties: Option<Properties>,
}

impl Encode for Header {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		w.write_all(&[self.session_present as u8])?;
		self.reason_code.encode(w)?;
		Ok(())
	}
}

impl Decode<Self> for Header {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		tracing::trace!(data = format!("{:2x?}", data), "Decoding connack header");
		let session_present = data[0] == 1;
		let reason_code = ReasonCode::from_repr(data[1])
			.ok_or(ControlPacketParseError::UnknownReasonCode(data[1]))?;
		let (properties, data) = Option::<Properties>::decode(&data[2..])?;

		Ok((
			Self {
				session_present,
				reason_code,
				properties,
			},
			data,
		))
	}
}
