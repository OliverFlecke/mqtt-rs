use crate::packet::{MqttControlPacket, kind::PacketType};

impl MqttControlPacket {
	/// Create a ping request packet.
	pub fn create_ping_req() -> Self {
		Self::new(PacketType::PingReq, None, None)
	}

	pub fn create_ping_resp() -> Self {
		Self::new(PacketType::PingResp, None, None)
	}
}
