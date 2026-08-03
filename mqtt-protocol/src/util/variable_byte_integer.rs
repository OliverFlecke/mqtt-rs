use std::io::{self, Cursor, Write};

use crate::packet::{ControlPacketParseError, Decode, Encode};

/// Represents a variable byte integer.
///
/// Maximum value is the maximum value of a `u32`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VariableByteInteger(u32);

impl VariableByteInteger {
	pub fn number_of_bytes(&self) -> usize {
		match self.0 {
			0..=127 => 1,
			128..=16_383 => 2,
			16_384..=2_097_151 => 3,
			2_097_152..=268_435_455 => 4,
			_ => panic!("Invalid value for variable byte integer"),
		}
	}
}

impl From<VariableByteInteger> for u32 {
	fn from(val: VariableByteInteger) -> Self {
		val.0
	}
}

impl From<VariableByteInteger> for usize {
	fn from(val: VariableByteInteger) -> Self {
		val.0 as usize
	}
}

impl TryFrom<u32> for VariableByteInteger {
	type Error = ControlPacketParseError;

	fn try_from(value: u32) -> Result<Self, Self::Error> {
		Ok(Self(value))
	}
}

impl Encode for VariableByteInteger {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		if self.0 == 0 {
			w.write_all(&[0])?;
			return Ok(());
		}

		let mut value = self.0;
		while 0 < value {
			let mut byte = (value % 128) as u8;
			value /= 128;
			if value > 0 {
				byte |= 128;
			}
			w.write_all(&[byte])?;
		}

		Ok(())
	}
}

impl Decode<Self> for VariableByteInteger {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		let mut multiplier = 1u32;
		let mut value = 0u32;
		let mut len = 0usize;

		for byte in data {
			value += ((*byte as u32) & 127) * multiplier;
			if multiplier > 128 * 128 * 128 {
				return Err(ControlPacketParseError::InvalidVariableByteIntegerLength);
			}
			multiplier *= 128;
			len += 1;

			if ((*byte as u32) & 128) == 0 {
				break;
			}
		}

		Ok((Self(value), &data[len..]))
	}
}

#[cfg(test)]
mod tests {
	use rstest::rstest;

	use super::*;

	#[rstest]
	#[case(1, &[0x01])]
	#[case(127, &[0x7F])]
	#[case(128, &[0x80, 0x01])]
	#[case(16_383, &[0xFF, 0x7F])]
	#[case(16_384, &[0x80, 0x80, 0x01])]
	#[case(2_097_151, &[0xFF, 0xFF, 0x7F])]
	#[case(2_097_152, &[0x80, 0x80, 0x80, 0x01])]
	#[case(268_435_455, &[0xFF, 0xFF, 0xFF, 0x7F])]
	fn encode_variable_byte_integer(#[case] value: u32, #[case] expected: &[u8]) {
		let mut cursor = Cursor::new(Vec::new());
		VariableByteInteger(value).encode(&mut cursor).unwrap();
		assert_eq!(cursor.into_inner(), expected);
	}
}
