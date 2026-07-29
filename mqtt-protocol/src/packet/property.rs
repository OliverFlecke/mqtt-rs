use std::io::{self, Cursor, Write};

use crate::packet::{ControlPacketParseError, Decode, Encode};

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone, Default)]
pub struct Properties {}

impl Encode for Properties {
	fn encode(&self, _w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		// TODO: implement. Needs to write overall length and each property
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
		Ok((None, data))
	}
}
