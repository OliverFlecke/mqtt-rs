use crate::packet::{MqttControlPacket, MqttFixedHeader, kind::PacketType};

/// Create a ping request packet.
pub fn create_ping_req() -> MqttControlPacket {
	MqttControlPacket {
		header: MqttFixedHeader {
			kind: PacketType::PingReq,
			remaining_length: 0,
		},
		variable_header: None,
		payload: None,
	}
}

pub fn create_ping_resp() -> MqttControlPacket {
	MqttControlPacket {
		header: MqttFixedHeader {
			kind: PacketType::PingResp,
			remaining_length: 0,
		},
		variable_header: None,
		payload: None,
	}
}
