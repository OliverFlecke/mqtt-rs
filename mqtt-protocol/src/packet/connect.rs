use std::io;
use std::io::Cursor;
use std::io::Write;

use crate::packet::{
	self, ControlPacketParseError, Decode, Encode, MqttControlPacket, ProtocolVersion, QoS,
	kind::PacketType, property::Properties,
};

const DEFAULT_KEEP_ALIVE: u16 = 60;

impl MqttControlPacket {
	/// Create a new connect packet
	pub fn connect(
		client_id: Option<String>,
		flags: Option<ConnectFlags>,
		keep_alive: Option<u16>,
	) -> Self {
		Self::new(
			PacketType::Connect,
			Some(packet::VariableHeader::Connect(Header::new(
				flags, keep_alive,
			))),
			Some(packet::Payload::Connect(Payload { client_id })),
		)
	}
}

#[derive(Debug, Clone)]
pub struct Header {
	version: ProtocolVersion,
	connect_flags: ConnectFlags,
	keep_alive: u16,
	properties: Option<Properties>,
}

impl Default for Header {
	fn default() -> Self {
		Self {
			version: ProtocolVersion::V5,
			connect_flags: ConnectFlags::default(),
			keep_alive: DEFAULT_KEEP_ALIVE,
			properties: None,
		}
	}
}

impl Header {
	pub fn new(flags: Option<ConnectFlags>, keep_alive: Option<u16>) -> Self {
		Self {
			version: ProtocolVersion::V5,
			connect_flags: flags.unwrap_or_default(),
			keep_alive: keep_alive.unwrap_or(DEFAULT_KEEP_ALIVE),
			properties: None,
		}
	}
}

impl Encode for Header {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		"MQTT".encode(w)?;
		w.write_all(&[self.version as u8])?;
		self.connect_flags.encode(w)?;
		w.write_all(&self.keep_alive.to_be_bytes())?;
		self.properties.encode(w)?;

		Ok(())
	}
}

impl Decode<Self> for Header {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		if [0x00, 0x04, b'M', b'Q', b'T', b'T'] != data[0..6] {
			return Err(ControlPacketParseError::IncorrectProtocol);
		}

		let (properties, data) = Option::<Properties>::decode(&data[11..])?;

		Ok((
			Self {
				version: ProtocolVersion::from_repr(data[7])
					.ok_or(ControlPacketParseError::UnsupportedProtocol(data[7]))?,
				connect_flags: ConnectFlags::try_from(data[8])?,
				keep_alive: u16::from_be_bytes([data[9], data[10]]),
				properties,
			},
			data,
		))
	}
}

/// Payload for a connect packet.
#[derive(Debug, Clone)]
pub struct Payload {
	pub client_id: Option<String>,
}

impl Encode for Payload {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		self.client_id.as_deref().encode(w)?;

		Ok(())
	}
}

impl Decode<Self> for Payload {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		if data.is_empty() {
			Ok((Self { client_id: None }, data))
		} else {
			let (client_id, data) = String::decode(data)?;
			Ok((
				Self {
					client_id: Some(client_id),
				},
				data,
			))
		}
	}
}

/// Contains flags that are required for a connection request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectFlags {
	username: bool,
	password: bool,
	will_retain: bool,
	will_qos: QoS,
	will: bool,
	clean_session: bool,
}

/// Implements default for the connect flags. All flags will be set to false,
/// except the `clean_session` flag, which assumes the connection is new.
impl Default for ConnectFlags {
	fn default() -> Self {
		Self {
			username: false,
			password: false,
			will_retain: false,
			will_qos: QoS::AtMostOnce,
			will: false,
			clean_session: true,
		}
	}
}

impl Encode for ConnectFlags {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		w.write_all(&[self.clone().into()])
	}
}

impl From<ConnectFlags> for u8 {
	fn from(val: ConnectFlags) -> Self {
		let mut value = 0;
		value |= (val.username as u8) << 7;
		value |= (val.password as u8) << 6;
		value |= (val.will_retain as u8) << 5;
		value |= (val.will_qos as u8) << 3;
		value |= (val.will as u8) << 2;
		value |= (val.clean_session as u8) << 1;
		value
	}
}

impl TryFrom<u8> for ConnectFlags {
	type Error = ControlPacketParseError;

	fn try_from(value: u8) -> Result<Self, Self::Error> {
		Ok(Self {
			username: (value & 0x80) != 0,
			password: (value & 0x40) != 0,
			will_retain: (value & 0x20) != 0,
			will_qos: QoS::from_repr((value >> 3) & 0x03)
				.ok_or(ControlPacketParseError::UnsupportedQoS((value >> 3) & 0x03))?,
			will: (value & 0x04) != 0,
			clean_session: (value & 0x02) != 0,
		})
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn serialize_connect_flags_default() {
		let flags = ConnectFlags::default();

		let encoded: u8 = flags.into();
		assert_eq!(encoded, 0b0000_0010);
	}

	#[test]
	fn deserialize_connect_flags_default() {
		let flags: ConnectFlags = 0b0000_0010.try_into().unwrap();

		assert_eq!(ConnectFlags::default(), flags);
	}

	#[test]
	fn serialize_connect_flags() {
		let flags = ConnectFlags {
			username: true,
			password: true,
			will_retain: true,
			will_qos: QoS::ExactlyOnce,
			will: true,
			clean_session: false,
		};

		let encoded: u8 = flags.clone().into();
		assert_eq!(encoded, 0b1111_0100);

		let decoded: ConnectFlags = encoded.try_into().unwrap();
		assert_eq!(flags, decoded);
	}

	#[test]
	fn deserialize_connect_flags() {
		let flags = ConnectFlags {
			username: true,
			password: true,
			will_retain: true,
			will_qos: QoS::ExactlyOnce,
			will: true,
			clean_session: false,
		};

		let encoded = 0b1111_0100;
		let decoded: ConnectFlags = encoded.try_into().unwrap();

		assert_eq!(flags, decoded);
	}

	#[test]
	fn deserialize_connect_flags_with_incorrect_will_qos() {
		let encoded = 0b0001_1000;
		let decoded: Result<ConnectFlags, ControlPacketParseError> = encoded.try_into();

		assert_eq!(Err(ControlPacketParseError::UnsupportedQoS(3)), decoded);
	}

	#[test]
	fn encode_variable_header() {
		let header = Header::default();
		let mut cursor = Cursor::new(Vec::new());
		header.encode(&mut cursor).unwrap();

		assert_eq!(
			cursor.into_inner(),
			vec![
				0x00,
				0x04,
				b'M',
				b'Q',
				b'T',
				b'T',
				0x05,        // version
				0b0000_0010, // flags
				0x00,
				60,   // keep alive, 2 bytes
				0x00  // properties, empty
			]
		);
	}
}
