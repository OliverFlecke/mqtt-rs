# MQTT-RS

[![Build](https://github.com/OliverFlecke/mqtt-rs/actions/workflows/build.yml/badge.svg)](https://github.com/OliverFlecke/mqtt-rs/actions/workflows/build.yml)
[![dependency status](https://deps.rs/repo/github/OliverFlecke/mqtt-rs/status.svg)](https://deps.rs/repo/github/OliverFlecke/mqtt-rs)

An implementation of the MQTT protocol in Rust.
This is mainly a learning project, stemming from wanting to learn more about the
MQTT protocol, which I have used for various application on a lower level, and
network programming.
Secondly, I was annoyed with the different CLI tools for MQTT.
The aim is support MQTT v5.0.

The project is laid out with three crates:

- `mqtt-protocol`: lower level packet types and utilities for encoding and decoding packets.
- `mqtt-client`: a client implementation that can connect to a broker and send and receive packets.
- `mqtt-cli`: a command line interface (and planned REPL) to the client.

## Installation

See [GitHub releases](https://github.com/OliverFlecke/mqtt-rs/releases) for pre-built binaries and
how to install.

## Usage

After installing, use `mqtt --help` to see the full list of available commands.

```sh
# Publish a message to a topic
mqtt publish my/topic "Hello World"

# Subscribe to a topic
mqtt subscribe my/topic

# Configure connection options
mqtt --host localhost --port 1883 --client-id alice publish my/topic "Hello World"

# Publish a retained message with QoS 2
mqtt publish --retain --qos exactly-once my/topic "Hello World"
```

## Features

Currently the following features are supported or are planned to be implemented:

- [ ] Protocol
	- [x] Connect
	- [x] Subscribe
	- [x] Publish
	- [x] Publish QoS 1
	- [x] Publish QoS 2
	- [x] Subscribe QoS 1
	- [x] Subscribe QoS 2
	- [ ] no_std support - currently there are dependencies on some types in `std`
	      which is used for encoding and decoding of packets, but it is quite
		  minimal. Should be possible to refactor this to use no_std. Main challenge
		  is to avoid using `Vec<u8>` for encoding.
	- [ ] Auth
- [ ] Client
	- [x] Connect
	- [x] Session management
	- [ ] Automatic reconnect
	- [x] TCP connection
	- [ ] TLS ?
	- [ ] Websocket
- [ ] CLI
	- [x] Commands for pub and sub
	- [x] REPL
	- [ ] Testing connection and features of broker
