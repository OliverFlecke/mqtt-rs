/// A client side session with a MQTT broker.
#[derive(Debug, Clone)]
pub struct Session {
	/// Client ID used to identify the connection to the broker.
	#[allow(dead_code)]
	pub client_id: String,

	/// Last packet ID used to identify the packets sent by the client.
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

	/// Get the next packet id to use.
	pub fn get_next_packet_id(&mut self) -> u16 {
		if self.last_packet_id == u16::MAX {
			self.last_packet_id = 0;
		}

		self.last_packet_id += 1;
		self.last_packet_id
	}
}
