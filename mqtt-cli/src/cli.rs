use clap::Parser;

#[derive(Parser)]
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

	#[command(subcommand)]
	pub command: Command,
}

#[derive(Parser)]
pub enum Command {
	Connect,
	Publish {
		#[arg()]
		topic: String,
		#[arg()]
		message: String,

		/// Indicate how frequent the message should be sent
		#[arg(short, long)]
		repeat_frequency_ms: Option<u64>,
	},
	Subscribe {
		#[arg()]
		topic: String,
	},
}
