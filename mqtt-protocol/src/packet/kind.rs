#[derive(Debug, Clone, Copy, strum::FromRepr)]
#[repr(u8)]
pub enum PacketType {
	// Reserved = 0,
	Connect = 1,
	ConnAck = 2,
	Publish = 3,
	PubAck = 4,
	PubRec = 5,
	PubRel = 6,
	PubComp = 7,
	Subscribe = 8,
	SubAck = 9,
	Unsubscribe = 10,
	UnsubAck = 11,
	PingReq = 12,
	PingResp = 13,
	Disconnect = 14,
	Auth = 15,
}
