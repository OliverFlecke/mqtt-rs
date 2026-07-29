/// Represents the different protocol versions of MQTT supported.
///
/// Currently the goal is only to support MQTT v5.
#[derive(Debug, Clone, Copy, PartialEq, Eq, strum::FromRepr)]
#[repr(u8)]
pub enum ProtocolVersion {
	V5 = 5,
}
