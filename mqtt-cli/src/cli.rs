use clap::Parser;
use mqtt_protocol::packet::{self};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
	/// Host to connect to.
	///
	/// Defaults to 127.0.0.1.
	#[arg(long, default_value = "localhost")]
	pub host: String,

	/// Port on the broker to connect to.
	///
	/// Defaults to 1883.
	#[arg(short, long, default_value = "1883")]
	pub port: u16,

	/// Client ID used to identify the connection to the broker.
	#[arg(short, long)]
	pub client_id: Option<String>,

	#[arg(short, long)]
	pub stay_connected: bool,

	#[arg(long, default_value_t = true)]
	pub heartbeat: bool,

	/// Log level to output information with.
	#[arg(short, long, default_value_t = tracing::Level::WARN)]
	pub log_level: tracing::Level,

	#[command(subcommand)]
	pub command: Command,
}

/// Quality of service to publish the message with.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::ValueEnum)]
pub enum QoS {
	#[default]
	AtMostOnce = 0,
	AtLeastOnce = 1,
	ExactlyOnce = 2,
}

impl From<QoS> for packet::QoS {
	fn from(val: QoS) -> Self {
		match val {
			QoS::AtMostOnce => packet::QoS::AtMostOnce,
			QoS::AtLeastOnce => packet::QoS::AtLeastOnce,
			QoS::ExactlyOnce => packet::QoS::ExactlyOnce,
		}
	}
}

#[derive(Debug, clap::Subcommand)]
pub enum Command {
	/// Connect to a broker, to validate the connection.
	Connect,

	Repl,

	/// Publish a message to a topic.
	#[command(alias("pub"))]
	Publish(Publish),

	/// Subscribe to a topic.
	#[command(alias("sub"))]
	Subscribe(Subscribe),
}

/// Arguments for the `publish` command.
#[derive(Debug, clap::Args)]
pub struct Publish {
	/// Topic to publish the message to.
	#[arg()]
	pub topic: String,

	/// Message to publish.
	#[arg()]
	pub message: String,

	/// Quality of service to publish the message with.
	#[arg(short, long, default_value = "at-most-once")]
	pub qos: QoS,

	/// Indicate how frequent the message should be sent
	#[arg(long)]
	pub repeat_frequency_ms: Option<u64>,

	/// Flag to indicate whether the message should be retained.
	#[arg(short, long, default_value_t = false)]
	pub retain: bool,
}

/// Arguments for the `subscribe` command.
#[derive(Debug, clap::Args)]
pub struct Subscribe {
	/// Topic to subscribe to.
	#[arg()]
	pub topic: String,

	#[arg(short, long, default_value = "at-most-once")]
	pub qos: QoS,

	#[arg(long, default_value = "send")]
	pub retain_handling: RetainHandling,

	#[arg(long)]
	pub retain_as_published: bool,

	#[arg(long)]
	pub no_local: bool,
}

#[derive(Debug, Clone, Copy, Default, clap::ValueEnum)]
#[repr(u8)]
pub enum RetainHandling {
	/// Send retained messages at the time of subscribe
	#[default]
	Send = 0,

	/// Send retained messages at subscribe only if the subscription does not currently exist
	SendOnlyIfSubscriptionDoesNotExist = 1,

	/// Do not send retained messages at the time of subscribe
	DoNotSend = 2,
}

impl From<RetainHandling> for packet::RetainHandling {
	fn from(val: RetainHandling) -> Self {
		match val {
			RetainHandling::Send => packet::RetainHandling::Send,
			RetainHandling::SendOnlyIfSubscriptionDoesNotExist => {
				packet::RetainHandling::SendOnlyIfSubscriptionDoesNotExist
			}
			RetainHandling::DoNotSend => packet::RetainHandling::DoNotSend,
		}
	}
}
