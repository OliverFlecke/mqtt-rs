use std::io::{self, Cursor};

use crate::packet::{
	self, ControlPacketParseError, Decode, Encode, MqttControlPacket, QoS, kind::PacketType,
	property::Properties,
};

impl MqttControlPacket {
	pub fn publish(topic: String, payload: Vec<u8>) -> Self {
		Self::new(
			PacketType::Publish,
			Some(packet::VariableHeader::Publish(Header {
				topic,
				// packet_identifier: None,
				properties: None,
			})),
			Some(packet::Payload::Publish(Payload(payload))),
		)
	}
}

#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct Flags {
	pub duplicate: bool,
	pub qos: QoS,
	pub retain: bool,
}

#[derive(Debug, Clone)]
pub struct Header {
	topic: String,
	// packet_identifier: Option<u16>,
	properties: Option<Properties>,
}

impl Header {
	pub fn topic(&self) -> &str {
		self.topic.as_str()
	}
}

impl Encode for Header {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		self.topic.as_str().encode(w)?;
		// self.packet_identifier.encode(w)?;
		self.properties.encode(w)?;

		Ok(())
	}
}

impl Decode<Header> for Header {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		let (topic, data) = String::decode(data)?;
		// let (packet_identifier, data) = Option::<String>::decode(data)?;
		let (properties, data) = Option::<Properties>::decode(data)?;

		Ok((
			Self {
				topic,
				// packet_identifier: None,
				properties,
			},
			data,
		))
	}
}

#[derive(Debug, Clone)]
pub struct Payload(Vec<u8>);

impl Encode for Payload {
	fn encode(&self, data: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		self.0.as_slice().encode(data)
	}
}

impl Decode<Payload> for Payload {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		let (payload, data) = Vec::<u8>::decode(data)?;
		Ok((Self(payload), data))
	}
}

impl TryInto<String> for Payload {
	type Error = std::string::FromUtf8Error;

	fn try_into(self) -> Result<String, Self::Error> {
		String::from_utf8(self.0)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn decode_payload() {
		let data = hex::decode("000b68656c6c6f20776f726c64").unwrap();
		let (payload, remaining) = Payload::decode(&data).unwrap();

		assert_eq!(payload.0, b"hello world".to_vec());
		assert_eq!(remaining, &[]);
	}
}
