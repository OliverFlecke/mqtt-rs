use std::io::{self, Cursor, Write};

use crate::packet::{ControlPacketParseError, Decode, Encode, QoS};

/// A topic filter that can be used to subscribe to a topic. Can include
/// wildcards.
#[derive(Debug, Clone)]
pub struct TopicFilter {
	topic: String,
	options: SubscriptionOptions,
}

impl TopicFilter {
	pub fn new(topic: &str) -> Self {
		Self {
			topic: topic.to_string(),
			options: SubscriptionOptions::default(),
		}
	}

	pub fn new_with_options(topic: &str, options: SubscriptionOptions) -> Self {
		Self {
			topic: topic.to_string(),
			options,
		}
	}
}

impl From<&str> for TopicFilter {
	fn from(topic: &str) -> Self {
		Self::new(topic)
	}
}

impl Encode for TopicFilter {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		self.topic.as_str().encode(w)?;
		self.options.encode(w)?;

		Ok(())
	}
}

impl Decode<Self> for TopicFilter {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		let (topic, data) = String::decode(data)?;
		let (options, data) = SubscriptionOptions::decode(data)?;

		Ok((Self { topic, options }, data))
	}
}

/// Different options that can be set on a subscription.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SubscriptionOptions {
	/// Quality of service to receive the message with.
	pub qos: QoS,

	/// Represents the No Local option. If the value is 1, Application Messages
	/// MUST NOT be forwarded to a connection with a ClientID equal to the
	/// ClientID of the publishing connection. It is a Protocol Error to set
	/// the No Local bit to 1 on a Shared Subscription
	pub no_local: bool,

	/// Represents the Retain As Published option. If 1, Application Messages
	/// forwarded using this subscription keep the RETAIN flag they were
	/// published with. If 0, Application Messages forwarded using this
	/// subscription have the RETAIN flag set to 0. Retained messages sent when
	/// the subscription is established have the RETAIN flag set to 1.
	pub retain_as_published: bool,

	/// This option specifies whether retained messages are sent when the
	/// subscription is established. This does not affect the sending of
	/// retained messages at any point after the subscribe. If there are no
	/// retained messages matching the Topic Filter, all of these values act
	/// the same
	pub retain_handling: RetainHandling,
}

impl From<&SubscriptionOptions> for u8 {
	fn from(value: &SubscriptionOptions) -> Self {
		let mut ret = 0;
		ret |= value.qos as u8;
		ret |= (value.no_local as u8) << 2;
		ret |= (value.retain_as_published as u8) << 3;
		ret |= (value.retain_handling as u8) << 4;

		ret
	}
}

impl Encode for SubscriptionOptions {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		w.write_all(&[self.into()])?;

		Ok(())
	}
}

impl Decode<Self> for SubscriptionOptions {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		if data.is_empty() {
			return Err(ControlPacketParseError::NotEnoughData);
		}

		let value = data[0];
		Ok((
			Self {
				qos: QoS::from_repr(value & 0x3)
					.ok_or(ControlPacketParseError::UnsupportedQoS(value))?,
				no_local: value & 0x4 != 0,
				retain_as_published: value & 0x8 != 0,
				retain_handling: RetainHandling::from_repr((value & 0x30) >> 4)
					.ok_or(ControlPacketParseError::UnsupportedRetainHandling(value))?,
			},
			&data[1..],
		))
	}
}

#[derive(Debug, Clone, Copy, strum::FromRepr, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum RetainHandling {
	/// Send retained messages at the time of subscribe
	#[default]
	Send = 0,

	/// Send retained messages at subscribe only if the subscription does not currently exist
	SendOnlyIfSubscriptionDoesNotExist = 1,

	/// Do not send retained messages at the time of subscribe
	DoNotSend = 2,
}

#[cfg(test)]
mod tests {
	use rstest::rstest;

	use super::*;

	#[rstest]
	#[case(QoS::AtMostOnce, 0)]
	#[case(QoS::AtLeastOnce, 0b01)]
	#[case(QoS::ExactlyOnce, 0b10)]
	fn subscription_options_qos_to_u8(#[case] qos: QoS, #[case] expected: u8) {
		let value = SubscriptionOptions {
			qos,
			..Default::default()
		};

		assert_eq!(u8::from(&value), expected);
		let (decoded, _) = SubscriptionOptions::decode(&[expected]).unwrap();
		assert_eq!(decoded, value);
	}

	#[rstest]
	#[case(false, 0)]
	#[case(true, 0b100)]
	fn subscription_options_no_local_to_u8(#[case] no_local: bool, #[case] expected: u8) {
		let value = SubscriptionOptions {
			no_local,
			..Default::default()
		};

		assert_eq!(u8::from(&value), expected);
		let (decoded, _) = SubscriptionOptions::decode(&[expected]).unwrap();
		assert_eq!(decoded, value);
	}

	#[rstest]
	#[case(false, 0b0)]
	#[case(true, 0b1000)]
	fn subscription_options_retain_as_published_to_u8(
		#[case] retain_as_published: bool,
		#[case] expected: u8,
	) {
		let value = SubscriptionOptions {
			retain_as_published,
			..Default::default()
		};

		assert_eq!(u8::from(&value), expected);
		let (decoded, _) = SubscriptionOptions::decode(&[expected]).unwrap();
		assert_eq!(decoded, value);
	}

	#[rstest]
	#[case(RetainHandling::Send, 0)]
	#[case(RetainHandling::SendOnlyIfSubscriptionDoesNotExist, 0x10)]
	#[case(RetainHandling::DoNotSend, 0x20)]
	fn subscription_options_retain_handling_to_u8(
		#[case] retain_handling: RetainHandling,
		#[case] expected: u8,
	) {
		let value = SubscriptionOptions {
			retain_handling,
			..Default::default()
		};
		assert_eq!(u8::from(&value), expected);

		let (decoded, _) = SubscriptionOptions::decode(&[expected]).unwrap();
		assert_eq!(decoded, value);
	}
}
