use std::io::{self, Cursor};

use crate::packet::{
	self, Decode, DecodeFromType, Encode, PacketType, connect, publish, suback, subscribe,
	unsuback, unsubscribe,
};

/// Payload for a packet.
///
/// This represents the various payloads that each packet can have. Note
/// that not all packet types has a payload.
#[derive(Debug, Clone)]
pub enum Payload {
	Connect(connect::Payload),
	Publish(publish::Payload),
	Subscribe(subscribe::Payload),
	SubAck(suback::Payload),
	Unsubscribe(unsubscribe::Payload),
	UnsubAck(unsuback::Payload),
}

impl Encode for Payload {
	fn encode(&self, data: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		match self {
			Payload::Connect(x) => x.encode(data),
			Payload::Publish(x) => x.encode(data),
			Payload::Subscribe(x) => x.encode(data),
			Payload::SubAck(x) => x.encode(data),
			Payload::Unsubscribe(x) => x.encode(data),
			Payload::UnsubAck(x) => x.encode(data),
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

impl DecodeFromType<Payload> for Payload {
	fn decode_from_type(
		kind: packet::PacketType,
		data: &[u8],
	) -> Result<(Option<Self>, &[u8]), packet::ControlPacketParseError> {
		tracing::trace!(?kind, data = format!("{:2x?}", data), "Decoding payload");

		match kind {
			PacketType::Connect => {
				connect::Payload::decode(data).map(|(p, d)| (Some(Self::Connect(p)), d))
			}
			PacketType::Publish => {
				publish::Payload::decode(data).map(|(p, d)| (Some(Self::Publish(p)), d))
			}
			PacketType::Subscribe => {
				subscribe::Payload::decode(data).map(|(p, d)| (Some(Self::Subscribe(p)), d))
			}
			PacketType::SubAck => {
				suback::Payload::decode(data).map(|(p, d)| (Some(Self::SubAck(p)), d))
			}
			PacketType::Unsubscribe => {
				unsubscribe::Payload::decode(data).map(|(p, d)| (Some(Self::Unsubscribe(p)), d))
			}
			PacketType::UnsubAck => {
				unsuback::Payload::decode(data).map(|(p, d)| (Some(Self::UnsubAck(p)), d))
			}

			PacketType::ConnAck
			| PacketType::Disconnect
			| PacketType::PubAck
			| PacketType::PubRel
			| PacketType::PubRec
			| PacketType::PubComp
			| PacketType::PingReq
			| PacketType::PingResp => Ok((None, data)),

			PacketType::Auth => {
				tracing::warn!(?kind, "Decoding not yet supported");
				Ok((None, data))
			}
		}
	}
}
