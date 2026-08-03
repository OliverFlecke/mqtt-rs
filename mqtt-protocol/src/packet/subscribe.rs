use std::io::{self, Cursor, Write};

use crate::packet::{
	self, ControlPacketParseError, Decode, Encode, MqttControlPacket, MqttFixedHeader, PacketType,
	Properties, TopicFilter,
};

impl MqttControlPacket {
	/// Create a control packet to subscribe to one or more topics (most be at
	/// least one topic).
	pub fn subscribe(packet_id: u16, topics: Vec<TopicFilter>) -> Self {
		debug_assert_ne!(topics.len(), 0);

		Self::new_from_parts(
			// Subscribe packets have a fixed header with flags set to 0x02.
			MqttFixedHeader::new(PacketType::Subscribe, 0x02),
			Some(packet::VariableHeader::Subscribe(Header {
				packet_id,
				properties: Some(Properties {
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

impl Decode<Self> for Header {
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

impl Decode<Self> for Payload {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		let mut data = data;
		let mut topics = Vec::new();
		while !data.is_empty() {
			let (topic, rest) = TopicFilter::decode(data)?;
			data = rest;
			topics.push(topic);
		}

		Ok((Self { topics }, data))
	}
}
