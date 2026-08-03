use crate::packet::{
	self, ControlPacketParseError, Decode, Encode, MqttControlPacket, MqttFixedHeader, PacketType,
	Properties, TopicFilter,
};

impl MqttControlPacket {
	pub fn unsubscribe(packet_id: u16, topics: Vec<TopicFilter>) -> Self {
		debug_assert_ne!(topics.len(), 0);

		Self::new_from_parts(
			// Unsubscribe packets have a fixed header with flags set to 0x02.
			MqttFixedHeader::new(PacketType::Unsubscribe, 0x02),
			Some(packet::VariableHeader::Unsubscribe(Header {
				packet_id,
				properties: None,
			})),
			Some(packet::Payload::Unsubscribe(Payload { topics })),
		)
	}
}

#[derive(Debug, Clone)]
pub struct Header {
	packet_id: u16,
	properties: Option<Properties>,
}

impl Encode for Header {
	fn encode(&self, w: &mut std::io::Cursor<Vec<u8>>) -> std::io::Result<()> {
		self.packet_id.encode(w)?;
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

#[derive(Debug, Clone)]
pub struct Payload {
	topics: Vec<TopicFilter>,
}

impl Encode for Payload {
	fn encode(&self, w: &mut std::io::Cursor<Vec<u8>>) -> std::io::Result<()> {
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
