use std::io::{self, Cursor};

use crate::packet::{
	self, Encode, MqttControlPacket, MqttFixedHeader, QoS, kind::PacketType, property::Properties,
};

impl MqttControlPacket {
	pub fn create_publish(topic: String, payload: Vec<u8>) -> Self {
		Self {
			header: MqttFixedHeader::new(PacketType::Publish),
			variable_header: Some(packet::VariableHeader::Publish(VariableHeader {
				topic,
				packet_identifier: None,
				properties: None,
			})),
			payload: Some(packet::Payload::Publish(Payload(payload))),
		}
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
pub struct VariableHeader {
	pub topic: String,
	pub packet_identifier: Option<String>,
	pub properties: Option<Properties>,
}

impl Encode for VariableHeader {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		self.topic.as_str().encode(w)?;
		self.properties.encode(w)?;

		Ok(())
	}
}

#[derive(Debug, Clone)]
pub struct Payload(Vec<u8>);

impl Encode for Payload {
	fn encode(&self, data: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		self.0.as_slice().encode(data)
	}
}
