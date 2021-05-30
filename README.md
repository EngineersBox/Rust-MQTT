# Rust-MQTT
An MQTT pub/sub with built in analyser

## Installing Rust

This project is built with Rust, as such you will need the language installed. For UNIX based OS' this is
straightforward and can be down via the steps below. I would advise you look at the official documentation in order to
install on your required system <https://www.rust-lang.org/tools/install>

For UNIX you can do the following:

```shell
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update
```

verify the installation was successful with

```shell
rustc --version
```

if the command was not found the the `PATH` variable was most likely not set and will need to be done manually by
appending `~/.cargo/bin` to `PATH`

## Run

Running the project is very simple via the cargo utility (this is part of the rust standard installation).
It is important to note that the **pubcontroller** should be started before the **analyser** as the subscriptions should
be established before the analyser tries to send. Since the first set of messages are QoS 0, these will not be queued
on the broker and as such won't be available to the subscriber past the arrival at the broker

### Pubcontroller

You can run the **pubcontroller** with the following:

```shell
cargo run --bin pubcontroller
```

### Analyser

You can run the **analyser** with the following:

```shell
cargo run --bin analyser
```

## Configuration

There are default configurations for the **pubcontroller** and **analyser** in the `resource` directory. These config files
define how the publisher, subscriber and MQTT client behave.

The two configuration files are as follows:
* `resource/pubcontroller.properties`
* `resource/analyser.properties`

The configuration properties avaiable are:
* `broker`: Properties for the broker connection
  * `host`: Hostname to connect to
  * `port`: Port to connect to. MQTT defaults to `1883` for TCP and `8883` for WebSocket connections
* `creds`: Credentials to connect to the broker,
  * `username`: Username to connect with
  * `password`: Password to connect with
* `client`: Configurations for persistence and sessions
  *`keep_alive`: How long persistent connections should last with inactivity
  * `timeout`: Duration for terminating a connection with idle state
  * `clean_session`: Whether to persist a previous cached session (ID, queued messages, etc)
* `subscriber_connection`: Defines the topics and retry rates
  * `id`: Client ID to register with the broker (unique)
  * `topics`: Which topics to subscribe to
  * `retries`: How many times to retry a reconnect to the broker
  * `retry_duration`: How often to perform a reconnection in milliseconds
* `publisher_connection`: Defines the topics and message quantity
  * `id`: Client ID to register with the broker (unique)
  * `topics`: Which topics to subscribe to
  * `message_quantity`: Number of messages to send relative to time period