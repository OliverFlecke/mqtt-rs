use std::io::{self, Cursor};

use crate::packet::{
	self, ControlPacketParseError, Decode, Encode, MqttControlPacket, QoS,
	fixed_header::MqttFixedHeader, kind::PacketType, property::Properties,
};

/// Represents options for a publish packet.
#[derive(Debug, Clone, Copy, Default)]
pub struct PublishOptions {
	/// Indicates whether this is a duplicate message. It should be 'false' by default,
	/// indicating to the server that this is the first time the message has been
	/// attempted to be sent to the server.
	/// `true` indicates that this message is likely a retry.
	pub duplicate: bool,

	/// Quality of service to publish the message with.
	pub qos: QoS,

	/// Whether the message should be retained by the server.
	pub retain: bool,
}

impl From<PublishOptions> for u8 {
	fn from(val: PublishOptions) -> Self {
		let mut value = 0;
		value |= (val.duplicate as u8) << 3;
		value |= (val.qos as u8) << 1;
		value |= val.retain as u8;
		value
	}
}

impl MqttControlPacket {
	/// Create a packet to publish a message to a topic.
	pub fn publish(topic: String, payload: Vec<u8>, options: PublishOptions) -> Self {
		Self::new_from_parts(
			MqttFixedHeader::new(PacketType::Publish, options.into()),
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

	#[test]
	fn publish_options_to_u8_default() {
		let options = PublishOptions::default();
		let value: u8 = options.into();
		assert_eq!(value, 0b0000);
	}

	#[test]
	fn publish_options_to_u8() {
		let options = PublishOptions {
			duplicate: true,
			qos: QoS::ExactlyOnce,
			retain: true,
		};

		let value: u8 = options.into();
		assert_eq!(value, 0b1101);
	}
}
