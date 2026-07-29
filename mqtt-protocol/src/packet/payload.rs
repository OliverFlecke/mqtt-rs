use std::io::{self, Cursor};

use crate::packet::{self, Encode};

/// Payload for a packet.
///
/// This represents the various payloads that each packet can have. Note
/// that not all packet types has a payload.
#[derive(Debug, Clone)]
pub enum Payload {
	Connect(packet::connect::Payload),
	Publish(packet::publish::Payload),
	Subscribe(packet::subscribe::Payload),
	SubAck(packet::suback::Payload),
}

impl Encode for Payload {
	fn encode(&self, data: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		match self {
			Payload::Connect(connect) => connect.encode(data),
			Payload::Publish(publish) => publish.encode(data),
			Payload::Subscribe(subscribe) => subscribe.encode(data),
			Payload::SubAck(suback) => suback.encode(data),
		}
	}
}

impl Encode for Option<Payload> {
	fn encode(&self, data: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		match self {
			Some(payload) => payload.encode(data),
			None => Ok(()),
		}
	}
}
