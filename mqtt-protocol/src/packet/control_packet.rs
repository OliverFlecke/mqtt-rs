use std::io::{self, Cursor, Write};

use crate::{
	VariableByteInteger,
	packet::{
		ControlPacketParseError, Decode, DecodeFromType, Encode, Payload, VariableHeader,
		fixed_header::MqttFixedHeader, kind::PacketType,
	},
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
	/// Create a new control packet from its parts.
	///
	/// Generally `new` should be preferred, unless a specific `MqttFixedHeader`
	/// is required.
	pub fn new_from_parts(
		header: MqttFixedHeader,
		variable_header: Option<VariableHeader>,
		payload: Option<Payload>,
	) -> Self {
		Self {
			header,
			variable_header,
			payload,
		}
	}

	/// Create a new control packet for a given packet type.
	///
	/// Header and payload are optional and dependent on the packet type.
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
		tracing::trace!(data = format!("{:2x?}", data), "Decoding control packet");

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
		// Need to know the length of the packet before the header can
		// be encoded.
		let mut cursor = Cursor::new(Vec::new());
		self.variable_header.encode(&mut cursor)?;
		self.payload.encode(&mut cursor)?;

		self.header.encode(w)?;
		VariableByteInteger::try_from(cursor.get_ref().len() as u32)
			.unwrap() // TODO: handle error
			.encode(w)?;
		w.write_all(&cursor.into_inner())?;

		Ok(())
	}
}
