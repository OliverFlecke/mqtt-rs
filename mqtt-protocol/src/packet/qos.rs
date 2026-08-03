/// Represents the various types of quality of service that messages
/// can be sent with.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, strum::FromRepr)]
#[repr(u8)]
pub enum QoS {
	#[default]
	AtMostOnce = 0,
	AtLeastOnce = 1,
	ExactlyOnce = 2,
}
