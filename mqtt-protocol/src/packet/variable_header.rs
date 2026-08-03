use std::io::{self, Cursor};

use crate::packet::{
	ControlPacketParseError, Decode, DecodeFromType, Encode, MqttFixedHeader, connack, connect,
	disconnect, kind::PacketType, puback, pubcomp, publish, pubrec, pubrel, suback, subscribe,
	unsuback, unsubscribe,
};

/// Represents the various variable headers that can be used in a packet.
#[derive(Debug, Clone)]
pub enum VariableHeader {
	Connect(connect::Header),
	ConnAck(connack::Header),
	Disconnect(disconnect::Header),
	Publish(publish::Header),
	PubAck(puback::Header),
	PubRec(pubrec::Header),
	PubRel(pubrel::Header),
	PubComp(pubcomp::Header),

	Subscribe(subscribe::Header),
	SubAck(suback::Header),
	Unsubscribe(unsubscribe::Header),
	UnsubAck(unsuback::Header),
}

impl Encode for VariableHeader {
	fn encode(&self, data: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		match self {
			VariableHeader::Connect(h) => h.encode(data),
			VariableHeader::ConnAck(h) => h.encode(data),
			VariableHeader::Disconnect(h) => h.encode(data),

			VariableHeader::Publish(h) => h.encode(data),
			VariableHeader::PubAck(h) => h.encode(data),
			VariableHeader::PubRec(h) => h.encode(data),
			VariableHeader::PubRel(h) => h.encode(data),
			VariableHeader::PubComp(w) => w.encode(data),

			VariableHeader::Subscribe(h) => h.encode(data),
			VariableHeader::SubAck(h) => h.encode(data),
			VariableHeader::Unsubscribe(h) => h.encode(data),
			VariableHeader::UnsubAck(h) => h.encode(data),
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

impl<'a> DecodeFromType<'a, Self> for VariableHeader {
	fn decode_from_type(
		header: &MqttFixedHeader,
		data: &'a [u8],
	) -> Result<(Option<Self>, &'a [u8]), ControlPacketParseError> {
		tracing::trace!(
			?header,
			data = format!("{:2x?}", data),
			"Decoding variable header"
		);

		match header.kind() {
			PacketType::Connect => {
				connect::Header::decode(data).map(|(h, d)| (Some(Self::Connect(h)), d))
			}
			PacketType::ConnAck => {
				connack::Header::decode(data).map(|(h, d)| (Some(Self::ConnAck(h)), d))
			}
			PacketType::Disconnect => {
				disconnect::Header::decode(data).map(|(h, d)| (Some(Self::Disconnect(h)), d))
			}

			PacketType::Publish => publish::Header::decode_from_type(header, data)
				.map(|(h, d)| (h.map(Self::Publish), d)),
			PacketType::PubAck => {
				puback::Header::decode(data).map(|(h, d)| (Some(Self::PubAck(h)), d))
			}
			PacketType::PubRec => {
				pubrec::Header::decode(data).map(|(h, d)| (Some(Self::PubRec(h)), d))
			}
			PacketType::PubRel => {
				pubrel::Header::decode(data).map(|(h, d)| (Some(Self::PubRel(h)), d))
			}
			PacketType::PubComp => {
				pubcomp::Header::decode(data).map(|(h, d)| (Some(Self::PubComp(h)), d))
			}

			PacketType::Subscribe => {
				subscribe::Header::decode(data).map(|(h, d)| (Some(Self::Subscribe(h)), d))
			}

			PacketType::SubAck => {
				suback::Header::decode(data).map(|(h, d)| (Some(Self::SubAck(h)), d))
			}
			PacketType::Unsubscribe => {
				unsubscribe::Header::decode(data).map(|(h, d)| (Some(Self::Unsubscribe(h)), d))
			}
			PacketType::UnsubAck => {
				unsuback::Header::decode(data).map(|(h, d)| (Some(Self::UnsubAck(h)), d))
			}

			PacketType::PingReq | PacketType::PingResp => Ok((None, data)),

			PacketType::Auth => {
				tracing::warn!(?header, "Variable header decoding not supported");
				Ok((None, data))
			}
		}
	}
}
