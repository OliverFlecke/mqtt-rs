use crate::packet::{
	self, ControlPacketParseError, DecodeMqtt, EncodeMqtt, MqttControlPacket, MqttFixedHeader,
	ProtocolVersion, WillQoS, kind::PacketType, property::Properties,
};

/// Create a new connect packet
pub fn connect(client_id: Option<String>) -> MqttControlPacket {
	MqttControlPacket {
		header: MqttFixedHeader {
			kind: PacketType::Connect,
			remaining_length: 0,
		},
		variable_header: Some(packet::VariableHeader::Connect(VariableHeader::default())),
		payload: Some(packet::Payload::Connect(Payload { client_id })),
	}
}

#[derive(Debug, Clone)]
pub struct VariableHeader {
	version: ProtocolVersion,
	connect_flags: ConnectFlags,
	keep_alive: u16,
	#[allow(dead_code)]
	properties: Option<Properties>,
}

impl Default for VariableHeader {
	fn default() -> Self {
		Self {
			version: ProtocolVersion::V5,
			connect_flags: ConnectFlags::default(),
			keep_alive: 10,
			properties: None,
		}
	}
}

impl VariableHeader {
	pub fn new(flags: Option<ConnectFlags>) -> Self {
		Self {
			version: ProtocolVersion::V5,
			connect_flags: flags.unwrap_or_default(),
			keep_alive: 60,
			properties: None,
		}
	}
}

impl EncodeMqtt for VariableHeader {
	fn encode(&self, data: &mut Vec<u8>) {
		data.extend_from_slice(&[0x00, 0x04]);
		data.extend_from_slice(b"MQTT");
		data.push(self.version as u8);
		data.push(self.connect_flags.clone().into());
		data.extend_from_slice(&self.keep_alive.to_be_bytes());

		self.properties.encode(data);
	}
}

impl DecodeMqtt<VariableHeader> for VariableHeader {
	fn try_decode(data: &[u8]) -> Result<Self, ControlPacketParseError> {
		if [0x00, 0x04, b'M', b'Q', b'T', b'T'] != data[0..6] {
			return Err(ControlPacketParseError::IncorrectProtocol);
		}

		Ok(Self {
			version: ProtocolVersion::from_repr(data[7])
				.ok_or(ControlPacketParseError::UnsupportedProtocol(data[7]))?,
			connect_flags: ConnectFlags::try_from(data[8])?,
			keep_alive: u16::from_be_bytes([data[9], data[10]]),
			properties: Option::<Properties>::try_decode(&data[11..])?,
		})
	}
}

/// Payload for a connect packet.
#[derive(Debug, Clone)]
pub struct Payload {
	pub client_id: Option<String>,
}

impl EncodeMqtt for Payload {
	fn encode(&self, data: &mut Vec<u8>) {
		let len: u16 = self.client_id.as_ref().map(|s| s.len() as u16).unwrap_or(0);
		tracing::debug!(
			"Client ID length: {} -> {:x?}",
			len,
			self.client_id.as_ref().map(|s| s.as_bytes())
		);

		data.extend_from_slice(&len.to_be_bytes());

		if let Some(client_id) = &self.client_id {
			data.extend_from_slice(client_id.as_bytes());
		}
	}
}

/// Contains flags that are required for a connection request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConnectFlags {
	username: bool,
	password: bool,
	will_retain: bool,
	will_qos: WillQoS,
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
			will_qos: WillQoS::AtMostOnce,
			will: false,
			clean_session: true,
		}
	}
}

impl EncodeMqtt for ConnectFlags {
	fn encode(&self, data: &mut Vec<u8>) {
		data.push(self.clone().into());
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
			will_qos: WillQoS::from_repr((value >> 3) & 0x03)
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
			will_qos: WillQoS::ExactlyOnce,
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
			will_qos: WillQoS::ExactlyOnce,
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
}
