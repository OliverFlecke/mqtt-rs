use clap::Parser;
use mqtt_cli::{Cli, Command};
use mqtt_client::{ConnectOptionsBuilder, MqttClient};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let args = Cli::parse();

	tracing::subscriber::set_global_default(
		tracing_subscriber::fmt()
			.with_max_level(args.log_level)
			.finish(),
	)?;
	tracing::trace!("Starting mqtt-cli");

	let options = ConnectOptionsBuilder::default()
		.client_id(args.client_id)
		.build()?;
	let client = MqttClient::connect(format!("{}:{}", args.host, args.port), options).await?;

	match args.command {
		Command::Connect => {
			// Should we do anything here? The client would already be connected above
		}
		Command::Publish(publish) => mqtt_cli::publish_handler(client, publish).await?,
		Command::Subscribe(sub) => mqtt_cli::subscribe_handler(client, sub).await?,
	}

	Ok(())
}
