use crate::packet::EncodeMqtt;

impl EncodeMqtt for &str {
	fn encode(&self, data: &mut Vec<u8>) {
		data.extend_from_slice(&(self.len() as u16).to_be_bytes());
		data.extend_from_slice(self.as_bytes());
	}
}
impl EncodeMqtt for &[u8] {
	fn encode(&self, data: &mut Vec<u8>) {
		data.extend_from_slice(&(self.len() as u16).to_be_bytes());
		data.extend_from_slice(self);
	}
}
