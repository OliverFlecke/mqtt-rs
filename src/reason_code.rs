#[derive(Debug)]
#[repr(u8)]
pub enum ReasonCode {
	Success = 0x00, // CONNACK, PUBACK, PUBREC, PUBREL, PUBCOMP, UNSUBACK, AUTH
	// DISCONNECT treats this as a "Normal Disconnect"
	// GrantedQoS0 = 0x00, // SUBACK
	// GrantedQoS1 = 0x01, // SUBACK
	// GrantedQOS2 = 0x02, // SUBACK
	DisconnectWithWillMessage = 0x04, // DISCONNECT

	NoMatchingSubscribers = 0x10, // PUBACK, PUBREC
	NoSubscriptionExisted = 0x11, // UNSUBACK

	ContinueAuthentication = 0x18, // AUTH
	ReAuthenication = 0x19,        // AUTH

	UnspecifiedError = 0x80, // CONNACK, PUBACK, PUBREC, SUBACK, UNSUBACK, DISCONNECT
	MalformedPacket = 0x81,  // CONNACK, DISCONNECT
	ProtocolError = 0x82,    // CONNACK, DISCONNECT
	ImplementationSpecificError = 0x83, // CONNACK, PUBACK, PUBREC, SUBACK, UNSUBACK, DISCONNECT
	UnsupportedProtocolVersion = 0x84, // CONNACK
	ClientIdentifierNotValid = 0x85, // CONNACK
	BadUserNameOrPassword = 0x86, // CONNACK
	NotAuthorized = 0x87,    // CONNACK, PUBACK, PUBREC, SUBACK, UNSUBACK, DISCONNECT
	ServerUnavailable = 0x88, // CONNACK
	ServerBusy = 0x89,       // CONNACK, DISCONNECT
	Banned = 0x8A,           //CONNACK
	ServerShuttingDown = 0x8B, // DISCONNECT
	BadAuthenticationMethod = 0x8C, // CONNACK, DISCONNECT

	KeepAliveTimeout = 0x8D, // DISCONNECT
	SessionTakenOver = 0x8E, // DISCONNECT

	TopicFilterInvalid = 0x8F,       // SUBACK, UNSUBACK, DISCONNECT
	TopicNameInvalid = 0x90,         // CONNACK, PUBACK, PUBREC, DISCONNECT
	PacketIdentifierInUse = 0x91,    // PUBACK, PUBREC, SUBACK, UNSUBACK
	PacketIdentifierNotFound = 0x92, // PUBREL, PUBCOMP
	ReceiveMaximumExceeded = 0x93,   // DISCONNECT

	TopicAliasInvalid = 0x94,    // DISCONNECT
	PacketTooLarge = 0x95,       // CONNACK, DISCONNECT
	MessageRateTooHigh = 0x96,   // DISCONNECT
	QuotaExceeded = 0x97,        // CONNACK, PUBACK, PUBREC, SUBACK, DISCONNECT
	AdministrativeAction = 0x98, // DISCONNECT
	PayloadFormatInvalid = 0x99, // CONNACK, PUBACK, PUBREC, DISCONNECT

	RetainNotSupported = 0x9A, // CONNACK, DISCONNECT
	QoSNotSupported = 0x9B,    // CONNACK, DISCONNECT
	UseAnotherServer = 0x9C,   // CONNACK, DISCONNECT

	ServerMoved = 0x9D,                         // CONNACK, DISCONNECT
	SharedSubscriptionsNotSupported = 0x9E,     // SUBACK, DISCONNECT
	ConnectionRateExceeded = 0x9F,              // CONNACK, DISCONNECT
	MaximumConnectionTime = 0xA0,               // DISCONNECT
	SubscriptionIdentifiersNotSupported = 0xA1, // SUBACK, DISCONNECT
	WildcardSubscriptionsNotSupported = 0xA2,   // SUBACK, DISCONNECT
}
