use std::io::{self, Cursor, Write};

use crate::packet::{
	ControlPacketParseError, Decode, DecodeFromType, Encode, Payload, VariableHeader,
	fixed_header::MqttFixedHeader, kind::PacketType,
};

/// Represents a single MQTT control packet, containing a fixed header,
/// and optionally a variable header and payload.
#[derive(Debug, Clone)]
pub struct MqttControlPacket {
	// Packet type and flags together make up the first byte.
	header: MqttFixedHeader,
	variable_header: Option<VariableHeader>,
	payload: Option<Payload>,
}

impl MqttControlPacket {
	pub fn new(
		packet_type: PacketType,
		header: Option<VariableHeader>,
		payload: Option<Payload>,
	) -> Self {
		Self {
			header: packet_type.into(),
			variable_header: header,
			payload,
		}
	}

	#[inline]
	pub fn kind(&self) -> PacketType {
		self.header.kind()
	}

	pub fn header(&self) -> &Option<VariableHeader> {
		&self.variable_header
	}

	/// Parse a packet from the given data.
	pub fn decode(data: &[u8]) -> Result<Self, ControlPacketParseError> {
		tracing::trace!("Parsing packet from bytes {:x?}", data);

		let (header, data) = MqttFixedHeader::decode(data)?;
		let (variable_header, data) = VariableHeader::decode_from_type(header.kind(), data)?;
		let (payload, _data) = Payload::decode_from_type(header.kind(), data)?;

		// TODO: This would be good to have, as we should not expect to have any data
		// left after parsing the packet.
		// debug_assert!(data.is_empty()); // There should be no data left

		Ok(Self {
			header,
			variable_header,
			payload,
		})
	}
}

impl From<MqttControlPacket> for (Option<VariableHeader>, Option<Payload>) {
	fn from(val: MqttControlPacket) -> Self {
		(val.variable_header, val.payload)
	}
}

impl Encode for MqttControlPacket {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		self.header.encode(w)?;
		self.variable_header.encode(w)?;
		self.payload.encode(w)?;

		// Update remaining length
		let pos = w.position();
		w.set_position(1);
		w.write_all(&[pos as u8 - 2])?;
		w.set_position(pos);

		Ok(())
	}
}
