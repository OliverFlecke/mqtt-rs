use std::io::{self, Cursor};

use crate::packet::{ControlPacketParseError, Decode, Encode};

/// Represents a topic that can be subscribed to.
#[derive(Debug, Clone)]
pub struct Topic(String);

impl From<&str> for Topic {
	fn from(value: &str) -> Self {
		Self(value.to_string())
	}
}

impl From<String> for Topic {
	fn from(value: String) -> Self {
		Self(value)
	}
}

impl AsRef<str> for Topic {
	fn as_ref(&self) -> &str {
		self.0.as_str()
	}
}

impl Encode for Topic {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		self.0.as_str().encode(w)
	}
}

impl Decode<Self> for Topic {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		let (topic, data) = String::decode(data)?;
		Ok((Self(topic), data))
	}
}
