pub mod variable_byte_integer;

use std::io::{self, Cursor, Write};

use crate::packet::Encode;

pub use variable_byte_integer::VariableByteInteger;

impl Encode for &str {
	fn encode(&self, w: &mut Cursor<Vec<u8>>) -> io::Result<()> {
		w.write_all(&(self.len() as u16).to_be_bytes())?;
		w.write_all(self.as_bytes())?;

		Ok(())
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
