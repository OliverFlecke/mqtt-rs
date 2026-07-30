pub mod variable_byte_integer;

use std::io::{self, Cursor, Write};

use crate::packet::{ControlPacketParseError, Decode, Encode};

pub use variable_byte_integer::VariableByteInteger;

impl Encode for &str {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		w.write_all(&(self.len() as u16).to_be_bytes())?;
		w.write_all(self.as_bytes())?;

		Ok(())
	}
}

impl Decode<Vec<u8>> for Vec<u8> {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), crate::packet::ControlPacketParseError> {
		let len = u16::from_be_bytes(data[0..2].try_into().unwrap()) as usize;
		let (value, rest) = data[2..].split_at(len);

		Ok((value.to_vec(), rest))
	}
}

impl Decode<String> for String {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), crate::packet::ControlPacketParseError> {
		let len = u16::from_be_bytes(data[0..2].try_into().unwrap()) as usize;
		let (value, rest) = data[2..].split_at(len);

		let s = str::from_utf8(value)
			.map_err(ControlPacketParseError::InvalidUtf8)?
			.to_string();

		Ok((s, rest))
	}
}

impl Encode for Option<&str> {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		match self {
			Some(value) => value.encode(w),
			None => w.write_all(&[0, 0]),
		}
	}
}

impl Encode for &[u8] {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		w.write_all(&(self.len() as u16).to_be_bytes())?;
		w.write_all(self)?;

		Ok(())
	}
}

#[duplicate::duplicate_item(
  int_type  size;
  [ u8 ]    [ 1 ];
  [ u16 ]   [ 2 ];
  [ u32 ]   [ 4 ];
)]
impl Decode<Self> for int_type {
	fn decode(data: &[u8]) -> Result<(Self, &[u8]), ControlPacketParseError> {
		#[allow(clippy::len_zero)]
		if data.len() < size {
			return Err(ControlPacketParseError::NotEnoughData);
		}

		let value = Self::from_be_bytes(data[0..size].try_into().expect("sizeasserted above"));
		Ok((value, &data[size..]))
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn encode_str() {
		let mut cursor = Cursor::new(Vec::new());
		"Hello World".encode(&mut cursor).unwrap();

		assert_eq!(
			cursor.into_inner(),
			vec![
				0x00, 0x0b, b'H', b'e', b'l', b'l', b'o', b' ', b'W', b'o', b'r', b'l', b'd'
			]
		);
	}

	#[test]
	fn encode_option_str() {
		let mut cursor = Cursor::new(Vec::new());
		Some("Hello World").encode(&mut cursor).unwrap();

		assert_eq!(
			cursor.into_inner(),
			vec![
				0x00, 0x0b, b'H', b'e', b'l', b'l', b'o', b' ', b'W', b'o', b'r', b'l', b'd'
			]
		);

		let mut cursor = Cursor::new(Vec::new());
		None::<&str>.encode(&mut cursor).unwrap();

		assert_eq!(cursor.into_inner(), vec![0x00, 0x00]);
	}
}
