use std::{
	collections::HashMap,
	io::{self, Cursor, Write},
};

use crate::{
	packet::{ControlPacketParseError, Decode, Encode, QoS},
	util::VariableByteInteger,
};

#[derive(Debug, Clone, strum::FromRepr)]
#[repr(u8)]
#[allow(dead_code)]
pub enum Id {
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

	ResponseInformation = 0x1A, // UTF-8 Encoded String -> CONNACK
	ServerReference = 0x1C,     // UTF-8 Encoded String -> CONNACK, DISCONNECT
	ReasonString = 0x1F, // UTF-8 Encoded String -> CONNACK, PUBACK, PUBREC, PUBREL, PUBCOMP, SUBACK, UNSUBACK, DISCONNECT, AUTH

	ReceiveMaximum = 0x21,                  // Two Byte Integer -> CONNECT, CONNACK
	TopicAliasMaximum = 0x22,               // Two Byte Integer -> CONNECT, CONNACK
	TopicAlias = 0x23,                      // Two Byte Integer -> PUBLISH
	MaximumQoS = 0x24,                      // Byte -> CONNACK
	RetainAvailable = 0x25,                 // Byte -> CONNACK
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
	pub payload_format_indicator: Option<u8>,
	pub message_expiry_interval: Option<u32>,
	pub content_type: Option<String>,
	pub response_topic: Option<String>,
	pub correlation_data: Option<Vec<u8>>,
	pub session_expiry_interval: Option<u32>,
	pub server_keep_alive: Option<u16>,
	pub authentication_method: Option<String>,
	pub authentication_data: Option<Vec<u8>>,

	pub request_problem_information: Option<u8>,
	pub response_information: Option<String>,
	pub server_reference: Option<String>,
	pub reason_string: Option<String>,

	pub will_delay_interval: Option<u32>,
	pub request_response_information: Option<u8>,
	pub topic_alias: Option<u16>,
	pub maximum_qos: Option<QoS>,
	pub wildcard_subscription_available: Option<u8>,
	pub subscription_identifier_available: Option<u8>,
	pub shared_subscription_available: Option<u8>,
	pub retain_available: Option<u8>,
	pub user_properties: Option<HashMap<String, String>>,
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
			w.write_all(&[Id::SubscriptionIdentifier as u8])?;
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

		let mut ps = Properties::new(len as u32);
		let mut i = 0;

		while i < len {
			let id =
				Id::from_repr(data[i]).ok_or(ControlPacketParseError::UnknownProperty(data[i]))?;
			i += 1;

			match id {
				Id::PayloadFormatIndicator => {
					ps.payload_format_indicator = Some(u8::decode_property(data, &mut i)?)
				}
				Id::MessageExpiryInterval => {
					ps.message_expiry_interval = Some(u32::decode_property(data, &mut i)?);
				}
				Id::ContentType => {
					ps.content_type = Some(String::decode_property(data, &mut i)?);
				}
				Id::ResponseTopic => {
					ps.response_topic = Some(String::decode_property(data, &mut i)?);
				}
				Id::CorrelationData => {
					ps.correlation_data = Some(Vec::<u8>::decode_property(data, &mut i)?);
				}
				Id::SubscriptionIdentifier => {
					ps.subscription_identifier =
						Some(VariableByteInteger::decode_property(data, &mut i)?);
				}
				Id::SessionExpiryInterval => {
					ps.session_expiry_interval = Some(u32::decode_property(data, &mut i)?);
				}
				Id::AssignedClientIdentifier => {
					ps.assigned_client_identifier = Some(String::decode_property(data, &mut i)?);
				}
				Id::ServerKeepAlive => {
					ps.server_keep_alive = Some(u16::decode_property(data, &mut i)?);
				}
				Id::AuthenticationMethod => {
					ps.authentication_method = Some(String::decode_property(data, &mut i)?);
				}
				Id::AuthenticationData => {
					ps.authentication_data = Some(Vec::<u8>::decode_property(data, &mut i)?);
				}
				Id::RequestProblemInformation => {
					ps.request_problem_information = Some(u8::decode_property(data, &mut i)?);
				}
				Id::WillDelayInterval => {
					ps.will_delay_interval = Some(u32::decode_property(data, &mut i)?);
				}
				Id::RequestResponseInformation => {
					ps.request_response_information = Some(u8::decode_property(data, &mut i)?);
				}
				Id::ResponseInformation => {
					ps.response_information = Some(String::decode_property(data, &mut i)?);
				}
				Id::ServerReference => {
					ps.server_reference = Some(String::decode_property(data, &mut i)?);
				}
				Id::ReasonString => {
					ps.reason_string = Some(String::decode_property(data, &mut i)?);
				}

				Id::ReceiveMaximum => {
					ps.receive_maximum = Some(u16::decode_property(data, &mut i)?);
				}
				Id::TopicAliasMaximum => {
					ps.topic_alias_maximum = Some(u16::decode_property(data, &mut i)?);
				}
				Id::TopicAlias => {
					ps.topic_alias = Some(u16::decode_property(data, &mut i)?);
				}
				Id::MaximumQoS => {
					let qos = u8::decode_property(data, &mut i)?;
					ps.maximum_qos = Some(
						QoS::from_repr(qos).ok_or(ControlPacketParseError::UnsupportedQoS(qos))?,
					);
				}
				Id::RetainAvailable => {
					ps.retain_available = Some(u8::decode_property(data, &mut i)?);
				}
				Id::UserProperty => {
					if ps.user_properties.is_none() {
						ps.user_properties = Some(HashMap::new());
					}

					if let Some(user_properties) = ps.user_properties.as_mut() {
						let key = String::decode_property(data, &mut i)?;
						let value = String::decode_property(data, &mut i)?;
						user_properties.insert(key, value);
					}
				}
				Id::MaximumPacketSize => {
					ps.maximum_packet_size = Some(u32::decode_property(data, &mut i)?);
				}
				Id::WildcardSubscriptionAvailable => {
					ps.wildcard_subscription_available = Some(u8::decode_property(data, &mut i)?);
				}
				Id::SubscriptionIdentifierAvailable => {
					ps.subscription_identifier_available = Some(u8::decode_property(data, &mut i)?);
				}
				Id::SharedSubscriptionAvailable => {
					ps.shared_subscription_available = Some(u8::decode_property(data, &mut i)?);
				}
			}
		}

		Ok((Some(ps), &data[len..]))
	}
}

/// Helper trait to decode properties into their respective types and size.
/// This will read the given type from the start of `data` and advance the index
/// by the consumed amount.
trait DecodeProperty<T> {
	fn decode_property(data: &[u8], index: &mut usize) -> Result<T, ControlPacketParseError>;
}

#[duplicate::duplicate_item(
  kind      size;
  [ u8 ]    [ 1 ];
  [ u16 ]   [ 2 ];
  [ u32 ]   [ 4 ];
)]
impl DecodeProperty<Self> for kind {
	fn decode_property(data: &[u8], index: &mut usize) -> Result<Self, ControlPacketParseError> {
		let (value, _) = Self::decode(&data[*index..*index + size])?;
		*index += size;
		Ok(value)
	}
}

#[duplicate::duplicate_item(
  kind          size;
  [ String ]    [ 2 ];
  [ Vec<u8> ]   [ 2 ];
)]
impl DecodeProperty<Self> for kind {
	fn decode_property(data: &[u8], index: &mut usize) -> Result<Self, ControlPacketParseError> {
		let (value, _) = Self::decode(&data[*index..])?;
		*index += size + value.len(); // length indicator + data length
		Ok(value)
	}
}

impl DecodeProperty<Self> for VariableByteInteger {
	fn decode_property(data: &[u8], index: &mut usize) -> Result<Self, ControlPacketParseError> {
		let (value, _) = Self::decode(&data[*index..])?;
		*index += value.number_of_bytes();
		Ok(value)
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

	#[test]
	fn decode_payload_format_indicator() {
		let data = &[0x02, Id::PayloadFormatIndicator as u8, 0x01];
		let (property, remaining) = Option::<Properties>::decode(data).unwrap();

		assert_eq!(property.unwrap().payload_format_indicator, Some(1));
		assert_eq!(remaining, &[]);
	}
}
