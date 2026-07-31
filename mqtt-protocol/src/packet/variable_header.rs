use std::io::{self, Cursor};

use crate::packet::{
	self, ControlPacketParseError, Decode, DecodeFromType, Encode, connack, connect, disconnect,
	kind::PacketType, puback, publish, suback,
};

/// Represents the various variable headers that can be used in a packet.
#[derive(Debug, Clone)]
pub enum VariableHeader {
	Connect(packet::connect::Header),
	ConnAck(packet::connack::Header),
	Disconnect(packet::disconnect::Header),
	Publish(packet::publish::Header),
	PubAck(packet::puback::Header),
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
			VariableHeader::PubAck(puback) => puback.encode(data),
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

impl DecodeFromType<VariableHeader> for VariableHeader {
	fn decode_from_type(
		kind: PacketType,
		data: &[u8],
	) -> Result<(Option<Self>, &[u8]), ControlPacketParseError> {
		tracing::trace!(
			?kind,
			data = format!("{:2x?}", data),
			"Decoding variable header"
		);

		match kind {
			PacketType::Connect => {
				connect::Header::decode(data).map(|(h, d)| (Some(Self::Connect(h)), d))
			}
			PacketType::ConnAck => {
				connack::Header::decode(data).map(|(h, d)| (Some(Self::ConnAck(h)), d))
			}
			PacketType::Disconnect => {
				disconnect::Header::decode(data).map(|(h, d)| (Some(Self::Disconnect(h)), d))
			}
			PacketType::SubAck => {
				suback::Header::decode(data).map(|(h, d)| (Some(Self::SubAck(h)), d))
			}

			PacketType::Publish => {
				publish::Header::decode(data).map(|(h, d)| (Some(Self::Publish(h)), d))
			}
			PacketType::PubAck => {
				puback::Header::decode(data).map(|(h, d)| (Some(Self::PubAck(h)), d))
			}

			PacketType::PingReq | PacketType::PingResp => Ok((None, data)),

			_ => {
				tracing::warn!(?kind, "Variable header decoding not supported");
				Ok((None, data))
			}
		}
	}
}
