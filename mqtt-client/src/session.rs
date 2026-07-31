/// A client side session with a MQTT broker.
#[derive(Debug, Clone)]
pub struct Session {
	/// Client ID used to identify the connection to the broker.
	#[allow(dead_code)]
	pub client_id: String,

	/// Last packet ID used to identify the packets sent by the client.
	#[allow(dead_code)]
	last_packet_id: u16,
	// pub maximum_packet_size: u32,
	// pub receive_maximum: u16,
}

impl Session {
	/// Create a new session with the given client id.
	pub fn new(client_id: String) -> Self {
		Self {
			client_id,
			last_packet_id: 0,
		}
	}
}
