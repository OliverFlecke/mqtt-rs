use clap::Parser;
use mqtt::{
	cli::Cli,
	client::MqttClient,
	packet::{MqttControlPacket, connect, create_disconnect},
};
use tokio::signal;
use tracing::level_filters::LevelFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	tracing::subscriber::set_global_default(
		tracing_subscriber::fmt()
			.with_max_level(LevelFilter::DEBUG)
			.finish(),
	)?;

	let cli = Cli::parse();

	tracing::debug!("Starting mqtt-cli");

	let client = MqttClient::connect(format!("{}:{}", cli.host, cli.port)).await?;
	let (tx, mut rx) = client.listen_and_wait()?;
	tokio::spawn(async move {
		while let Some(packet) = rx.recv().await {
			tracing::debug!("Packet received: {:x?}", packet.header.kind);
		}
	});

	tx.send(connect(Some(String::from("alice")))).await?;
	tx.send(MqttControlPacket::create_publish(
		"test".to_string(),
		b"hello world".to_vec(),
	))
	.await?;

	tokio::select! {
		_ = signal::ctrl_c() => {
			tx.send(create_disconnect()).await?;

			tracing::debug!("Shutting down");
		},
	}

	Ok(())
}
