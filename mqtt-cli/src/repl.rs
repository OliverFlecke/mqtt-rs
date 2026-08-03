use anyhow::Context;
use clap::Parser;
use mqtt_client::MqttClient;
use mqtt_protocol::packet::{Payload, VariableHeader};
use rustyline::{DefaultEditor, error::ReadlineError};

use crate::{Publish, Subscribe};

/// Readline REPL for the MQTT client.
pub async fn handler(client: MqttClient) -> anyhow::Result<()> {
	let mut rl = DefaultEditor::new()?;
	let mut client = client;

	// TODO: it would be good to have a context to know which topics we are
	// subscribed to. That would also allow us to easily provide outputs that
	// informs the user when the sub/unsub from a topic.
	// Should that be tracked here or be a feature of the client?

	let ct = client.on_message(move |msg| match msg.into() {
		(Some(VariableHeader::Publish(header)), Some(Payload::Publish(payload))) => {
			let payload: String = payload
				.try_into()
				.context("Unable to decode received message")
				.unwrap();
			println!("{:}: {:}", header.topic(), payload);
		}
		(Some(VariableHeader::SubAck(_)), _) => {
			// Would be nice to print the topic that we have subscribed to.
			println!("Subscribed");
		}
		_ => {}
	});

	loop {
		let readline = rl.readline("> ");
		match readline {
			Ok(line) => {
				if line.is_empty() {
					continue;
				}

				rl.add_history_entry(line.as_str())?;
				tracing::debug!("Read line: {}", line);

				let mut args = vec!["repl".to_string()];
				match shell_words::split(line.as_str()) {
					Ok(words) => args.extend(words),
					Err(err) => {
						println!("Parsing error: {err}");
						continue;
					}
				}

				let args = match Args::try_parse_from(args) {
					Ok(args) => args,
					Err(err) => {
						eprintln!("{:?}", err);
						continue;
					}
				};

				tracing::debug!("Parsed arguments {:#?}", args);
				handle_command(&mut client, args.command).await?;
			}
			Err(ReadlineError::Interrupted) | Err(ReadlineError::Eof) => {
				break;
			}
			Err(err) => {
				eprintln!("Error: {:?}", err);
				break;
			}
		}
	}

	ct.cancel();

	Ok(())
}

async fn handle_command(client: &mut MqttClient, command: Command) -> anyhow::Result<()> {
	match command {
		Command::Publish(publish) => {
			client
				.publish(
					publish.topic.into(),
					publish.message.into_bytes(),
					publish.retain,
					publish.qos.into(),
				)
				.await?;
		}
		Command::Subscribe(sub) => {
			client.subscribe(sub.topic.as_str().into()).await?;
		}
		Command::Unsubscribe { topic } => {
			client.unsubscribe(topic.as_str().into()).await?;
			println!("Unsubscribed from {}", topic.as_str());
		}
	};

	Ok(())
}

#[derive(Debug, Parser)]
struct Args {
	#[command(subcommand)]
	pub command: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
	#[command(alias("pub"))]
	Publish(Publish),
	#[command(alias("sub"))]
	Subscribe(Subscribe),
	#[command(alias("unsub"))]
	Unsubscribe { topic: String },
}
