use std::io::{self, Cursor, Write};

use crate::packet::{
	ControlPacketParseError, Decode, Encode, property::Properties, reason::ReasonCode,
};

#[derive(Debug, Clone)]
pub struct Header {
	pub packet_id: u16,
	pub properties: Option<Properties>,
}

impl Encode for Header {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		w.write_all(&self.packet_id.to_be_bytes())?;
		self.properties.encode(w)?;

		Ok(())
	}
}

impl Decode<Header> for Header {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		let packet_id = u16::from_be_bytes([data[0], data[1]]);
		let (properties, data) = Option::<Properties>::decode(&data[2..])?;

		Ok((
			Self {
				packet_id,
				properties,
			},
			data,
		))
	}
}

#[derive(Debug, Clone)]
pub struct Payload {
	reason_codes: Vec<ReasonCode>,
}

impl Encode for Payload {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		for code in &self.reason_codes {
			code.encode(w)?;
		}

		Ok(())
	}
}

impl Decode<Payload> for Payload {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		let mut reason_codes = Vec::new();
		for code in data {
			reason_codes.push(
				ReasonCode::from_repr(*code)
					.ok_or(ControlPacketParseError::UnknownReasonCode(*code))?,
			);
		}

		Ok((Self { reason_codes }, &[]))
	}
}
