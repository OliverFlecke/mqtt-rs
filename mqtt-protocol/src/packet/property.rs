use std::io::{self, Cursor, Write};

use crate::{
	packet::{ControlPacketParseError, Decode, Encode},
	util::VariableByteInteger,
};

#[derive(Debug, Clone, strum::FromRepr)]
#[repr(u8)]
#[allow(dead_code)]
pub enum PropertyId {
	PayloadFormatIndicator = 0x01,     // Byte -> PUBLISH, Will Properties
	MessageExpiryInterval = 0x02,      // Four Byte Integer -> PUBLISH, Will Properties
	ContentType = 0x03,                // UTF-8 Encoded String -> PUBLISH, Will Properties
	ResponseTopic = 0x08,              // UTF-8 Encoded String -> PUBLISH, Will Properties
	CorrelationData = 0x09,            // Binary Data -> PUBLISH, Will Properties
	SubscriptionIdentifier = 0x0B,     // Variable Byte Integer -> PUBLISH, SUBSCRIBE
	SessionExpiryInterval = 0x11,      // Four Byte Integer -> CONNECT, CONNACK, DISCONNECT
	AssignedClientIdentifier = 0x12,   // UTF-8 Encoded String -> CONNACK
	ServerKeepAlive = 0x13,            // Two Byte Integer -> CONNACK
	AuthenticationMethod = 0x15,       // UTF-8 Encoded String -> CONNECT, CONNACK, AUTH
	AuthenticationData = 0x16,         // Binary Data -> CONNECT, CONNACK, AUTH
	RequestProblemInformation = 0x17,  // Byte -> CONNECT
	WillDelayInterval = 0x18,          // Four Byte Integer -> Will Properties
	RequestResponseInformation = 0x19, // Byte -> CONNECT
	ResponseInformation = 0x1A,        // UTF-8 Encoded String -> CONNACK
	ServerReference = 0x1C,            // UTF-8 Encoded String -> CONNACK, DISCONNECT
	ReasonString = 0x1F, // UTF-8 Encoded String -> CONNACK, PUBACK, PUBREC, PUBREL, PUBCOMP, SUBACK, UNSUBACK, DISCONNECT, AUTH
	ReceiveMaximum = 0x21, // Two Byte Integer -> CONNECT, CONNACK
	TopicAliasMaximum = 0x22, // Two Byte Integer -> CONNECT, CONNACK
	TopicAlias = 0x23,   // Two Byte Integer -> PUBLISH
	MaximumQoS = 0x24,   // Byte -> CONNACK
	RetainAvailable = 0x25, // Byte -> CONNACK
	UserProperty = 0x26, // UTF-8 String Pair -> CONNECT, CONNACK, PUBLISH, Will Properties, PUBACK, PUBREC, PUBREL, PUBCOMP, SUBSCRIBE, SUBACK, UNSUBSCRIBE, UNSUBACK, DISCONNECT, AUTH
	MaximumPacketSize = 0x27, // Four Byte Integer -> CONNECT, CONNACK
	WildcardSubscriptionAvailable = 0x28, // Byte -> CONNACK
	SubscriptionIdentifierAvailable = 0x29, // Byte -> CONNACK
	SharedSubscriptionAvailable = 0x2A, // Byte -> CONNACK
}

// TODO: write encoder and decoder for properties

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Properties {
	pub length: u32,
	pub subscription_identifier: Option<VariableByteInteger>,
	pub topic_alias_maximum: Option<u16>,
	pub assigned_client_identifier: Option<String>,
	pub maximum_packet_size: Option<u32>,
	pub receive_maximum: Option<u16>,
}

impl Properties {
	pub fn new(length: u32) -> Self {
		Self {
			length,
			..Self::default()
		}
	}
}

impl Encode for Properties {
	fn encode(&self, final_writer: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		// TODO: implement. Needs to write overall length and each property
		let mut w = Cursor::new(Vec::new());

		if let Some(subscription_identifier) = &self.subscription_identifier {
			w.write_all(&[PropertyId::SubscriptionIdentifier as u8])?;
			subscription_identifier.encode(&mut w)?;
		}

		let data = w.into_inner();
		let length = VariableByteInteger::try_from(data.len() as u32).unwrap(); // TODO: handle error
		length.encode(final_writer)?;
		final_writer.write_all(&data)?;

		Ok(())
	}
}

impl Encode for Option<Properties> {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		match self {
			Some(properties) => properties.encode(w),
			None => w.write_all(&[0]),
		}
	}
}

impl Decode<Option<Properties>> for Option<Properties> {
	fn decode(data: &[u8]) -> Result<(Option<Properties>, &[u8]), ControlPacketParseError> {
		let (property_length, data) = VariableByteInteger::decode(data)?;
		let len: usize = property_length.into();

		if len == 0 {
			return Ok((None, &data[len..]));
		}

		let mut properties = Properties::new(len as u32);
		let mut i = 0;

		while i < len {
			match PropertyId::from_repr(data[i]) {
				Some(PropertyId::SubscriptionIdentifier) => {
					let (sub_id, _) = VariableByteInteger::decode(&data[i + 1..])?;
					i += 1 + sub_id.num_of_bytes();
					properties.subscription_identifier = Some(sub_id);
				}

				Some(PropertyId::TopicAliasMaximum) => {
					let (value, _) = u16::decode(&data[i + 1..])?;
					i += 3;
					properties.topic_alias_maximum = Some(value);
				}
				Some(PropertyId::AssignedClientIdentifier) => {
					let (value, _) = String::decode(&data[i + 1..])?;
					i += 3 + value.len(); // Property id + 2 byte length + string length
					properties.assigned_client_identifier = Some(value);
				}
				Some(PropertyId::MaximumPacketSize) => {
					let (value, _) = u32::decode(&data[i + 1..])?;
					i += 5;
					properties.maximum_packet_size = Some(value);
				}
				Some(PropertyId::ReceiveMaximum) => {
					let (value, _) = u16::decode(&data[i + 1..])?;
					properties.receive_maximum = Some(value);
					i += 3;
				}

				// TODO: implement remaining properties
				Some(id) => {
					tracing::warn!("Decoding of property id not yet implemented: {:x?}", id);
					i += 1; // Just to read through the properties that are not yet implemented.
					// Not perfect, as it might hit already supported values.
				}
				None => return Err(ControlPacketParseError::UnknownProperty(data[i])),
			}
		}

		Ok((Some(properties), &data[len..]))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	use pretty_assertions::assert_eq;

	#[test]
	fn decode_subscription_identifier() {
		let data = &[0x02, 0x0b, 0x0a];
		let (property, remaining) = Option::<Properties>::decode(data).unwrap();

		assert_eq!(
			property.unwrap().subscription_identifier,
			Some(10.try_into().unwrap())
		);
		assert_eq!(remaining, &[]);
	}

	#[test]
	fn decode_properties_from_connack() {
		// Data taken from actual response from broker.
		let data = hex::decode(
			"3722000a1200296175746f2d33303743303245432d443545432d334434302d314246302d32343846354541463346333227001e8480210014",
		).expect("valid hex data");

		let (property, remaining) = Option::<Properties>::decode(&data).expect("valid data");

		assert_eq!(remaining, &[]);

		let expected = Properties {
			length: 55,
			topic_alias_maximum: Some(10),
			assigned_client_identifier: Some(
				"auto-307C02EC-D5EC-3D40-1BF0-248F5EAF3F32".to_string(),
			),
			maximum_packet_size: Some(2000000),
			receive_maximum: Some(20),
			..Default::default()
		};
		assert_eq!(property.unwrap(), expected);
	}
}
