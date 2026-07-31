use std::io::{self, Cursor, Write};

use crate::{
	VariableByteInteger,
	packet::{
		self, ControlPacketParseError, Decode, Encode, MqttControlPacket, QoS,
		fixed_header::MqttFixedHeader, kind::PacketType, property::Properties,
	},
};

impl MqttControlPacket {
	pub fn subscribe(topics: Vec<TopicFilter>) -> Self {
		debug_assert_ne!(topics.len(), 0);

		Self::new_from_parts(
			// Subscribe packets have a fixed header with flags set to 0x02.
			MqttFixedHeader::new(PacketType::Subscribe, 0x02),
			Some(packet::VariableHeader::Subscribe(Header {
				packet_id: 20, // FIXME: hardcoded number
				properties: Some(Properties {
					subscription_identifier: Some(VariableByteInteger::try_from(10).unwrap()), // FIXME: hardcoded number
					..Default::default()
				}),
			})),
			Some(packet::Payload::Subscribe(Payload { topics })),
		)
	}
}

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

/// Payload for a subscribe packet, containing a list of topic filters.
#[derive(Debug, Clone)]
pub struct Payload {
	topics: Vec<TopicFilter>,
}

impl Encode for Payload {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		for topic in &self.topics {
			topic.encode(w)?;
		}

		Ok(())
	}
}

impl Decode<Payload> for Payload {
	fn decode(_data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		todo!()
	}
}

#[derive(Debug, Clone)]
pub struct TopicFilter {
	topic: String,
	options: SubscriptionOptions,
}

impl TopicFilter {
	pub fn new(topic: &str) -> Self {
		Self {
			topic: topic.to_string(),
			options: SubscriptionOptions::default(),
		}
	}
}

impl From<&str> for TopicFilter {
	fn from(topic: &str) -> Self {
		Self::new(topic)
	}
}

impl Encode for TopicFilter {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		self.topic.as_str().encode(w)?;
		self.options.encode(w)?;

		Ok(())
	}
}

#[derive(Debug, Clone, Default)]
pub struct SubscriptionOptions {
	qos: QoS,
}

impl Encode for SubscriptionOptions {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		// TODO: This is missing other flags.
		w.write_all(&[self.qos as u8])?;

		Ok(())
	}
}
