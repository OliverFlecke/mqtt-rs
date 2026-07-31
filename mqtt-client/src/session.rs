/// A client side session with a MQTT broker.
#[derive(Debug, Clone)]
pub struct Session {
	#[allow(dead_code)]
	pub client_id: String,
	// pub maximum_packet_size: u32,
	// pub receive_maximum: u16,
}
