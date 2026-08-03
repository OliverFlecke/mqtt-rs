use crate::packet::{Decode, Encode};

#[derive(Debug, Clone)]
pub struct Header {
	packet_id: u16,
}

impl Encode for Header {
	fn encode(&self, w: &mut std::io::Cursor<Vec<u8>>) -> std::io::Result<()> {
		self.packet_id.encode(w)
	}
}

impl Decode<Self> for Header {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), super::ControlPacketParseError> {
		let packet_id = u16::from_be_bytes([data[0], data[1]]);
		Ok((Self { packet_id }, &data[2..]))
	}
}

#[derive(Debug, Clone)]
pub struct Payload {
	reason_codes: Vec<u8>,
}

impl Encode for Payload {
	fn encode(&self, w: &mut std::io::Cursor<Vec<u8>>) -> std::io::Result<()> {
		for reason_code in &self.reason_codes {
			reason_code.encode(w)?;
		}

		Ok(())
	}
}

impl Decode<Self> for Payload {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), super::ControlPacketParseError> {
		let mut data = data;
		let mut reason_codes = Vec::new();
		while !data.is_empty() {
			let reason_code = u8::decode(data)?;
			data = reason_code.1;
			reason_codes.push(reason_code.0);
		}

		Ok((Self { reason_codes }, data))
	}
}
