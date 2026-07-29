use std::io::{self, Cursor};

use crate::packet::{self, ControlPacketParseError, Decode, Encode, kind::PacketType};

/// Represents the various variable headers that can be used in a packet.
#[derive(Debug, Clone)]
pub enum VariableHeader {
	Connect(packet::connect::Header),
	ConnAck(packet::connack::Header),
	Disconnect(packet::disconnect::Header),
	Publish(packet::publish::Header),
	Subscribe(packet::subscribe::Header),
	SubAck(packet::suback::Header),
}

impl Encode for VariableHeader {
	fn encode(&self, data: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		match self {
			VariableHeader::Connect(connect) => connect.encode(data),
			VariableHeader::ConnAck(connack) => connack.encode(data),
			VariableHeader::Disconnect(disconnect) => disconnect.encode(data),
			VariableHeader::Publish(publish) => publish.encode(data),
			VariableHeader::Subscribe(subscribe) => subscribe.encode(data),
			VariableHeader::SubAck(suback) => suback.encode(data),
		}
	}
}

impl Encode for Option<VariableHeader> {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		match self {
			Some(variable_header) => variable_header.encode(w),
			None => Ok(()),
		}
	}
}

impl VariableHeader {
	pub fn decode(
		kind: PacketType,
		data: &[u8],
	) -> Result<Option<(Self, &[u8])>, ControlPacketParseError> {
		match kind {
			PacketType::Connect => {
				packet::connect::Header::decode(data).map(|(h, d)| Some((Self::Connect(h), d)))
			}
			PacketType::ConnAck => {
				packet::connack::Header::decode(data).map(|(h, d)| Some((Self::ConnAck(h), d)))
			}
			PacketType::Disconnect => packet::disconnect::Header::decode(data)
				.map(|(h, d)| Some((Self::Disconnect(h), d))),
			PacketType::SubAck => {
				packet::suback::Header::decode(data).map(|(h, d)| Some((Self::SubAck(h), d)))
			}

			PacketType::PingReq | PacketType::PingResp => Ok(None),

			_ => {
				tracing::warn!("Decoding of {:?} is not yet supported", kind);
				Ok(None)
			}
		}
	}
}
