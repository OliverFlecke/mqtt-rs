use std::io::{self, Cursor, Write};

use crate::packet::{ControlPacketParseError, Decode, Encode, QoS};

/// A topic filter that can be used to subscribe to a topic. Can include
/// wildcards.
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

impl Decode<Self> for TopicFilter {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		let (topic, data) = String::decode(data)?;
		let (options, data) = SubscriptionOptions::decode(data)?;

		Ok((Self { topic, options }, data))
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

impl Decode<Self> for SubscriptionOptions {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		if data.is_empty() {
			return Err(ControlPacketParseError::NotEnoughData);
		}

		let qos =
			QoS::from_repr(data[0]).ok_or(ControlPacketParseError::UnsupportedQoS(data[0]))?;

		Ok((Self { qos }, &data[1..]))
	}
}
