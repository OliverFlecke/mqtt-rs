use std::io::{self, Cursor};

use crate::packet::{self, Decode, DecodeFromType, Encode, PacketType};

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

impl DecodeFromType<Payload> for Payload {
	fn decode_from_type(
		kind: packet::PacketType,
		data: &[u8],
	) -> Result<(Option<Self>, &[u8]), packet::ControlPacketParseError> {
		tracing::trace!("Decoding payload: {:?} => {:x?}", kind, data);

		match kind {
			PacketType::Publish => {
				packet::publish::Payload::decode(data).map(|(p, d)| (Some(Self::Publish(p)), d))
			}
			PacketType::SubAck => {
				packet::suback::Payload::decode(data).map(|(p, d)| (Some(Self::SubAck(p)), d))
			}

			PacketType::ConnAck | PacketType::PingReq | PacketType::PingResp => Ok((None, data)),

			_ => {
				tracing::warn!(?kind, "Decoding not yet supported");
				Ok((None, data))
			}
		}
	}
}
