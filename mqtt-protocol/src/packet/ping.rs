use crate::packet::{MqttControlPacket, kind::PacketType};

impl MqttControlPacket {
	/// Create a ping request packet.
	pub fn create_ping_req() -> Self {
		Self {
			header: PacketType::PingReq.into(),
			variable_header: None,
			payload: None,
		}
	}

	pub fn create_ping_resp() -> Self {
		Self {
			header: PacketType::PingResp.into(),
			variable_header: None,
			payload: None,
		}
	}
}
