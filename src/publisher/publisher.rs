use std::{
    env,
    process,
    time::Duration,
    thread,
};
use crate::config::config::Config;
use self::mqtt::CreateOptions;
use slog::Logger;

#[macro_use]
extern crate slog;
extern crate paho_mqtt as mqtt;

struct Publisher {
    config: *Config,
    logger: Logger,
    conn_opts: mqtt::ConnectOptions,
    pub client: mqtt::Client
}

impl Publisher {
    pub fn new(config: &Config, logger: &Logger) -> Publisher {
        Publisher {
            config,
            logger: logger.new(o!("Publisher" => thread::current().id().into())),
            conn_opts: Default::default(),
            client: Default::default(),
        }
    }
    pub fn initialize(&mut self) {
        let create_opts: CreateOptions = mqtt::CreateOptionsBuilder::new()
            .server_uri(self.config.broker.clone())
            .client_id(self.config.creds.client_id.clone())
            .finalize();
        self.client = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
            panic!("Error creating the client: {:?}", err);
        });
        debug!(self.logger, "Initialised client with options: {:?}", create_opts);
        self.conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_millis(self.config.client.keep_alive))
            .clean_session(true)
            .user_name(self.config.creds.username.clone())
            .password(self.config.creds.password.clone())
            .connect_timeout(Duration::from_millis(self.config.client.timeout))
            .finalize();
        debug!(self.logger, "Created connection options: {:?}", self.conn_opts);
        info!(self.logger, "Initialised client");
    }
    pub fn connect(&self) {
        if let Err(e) = self.client.connect(conn_opts) {
            panic!("Unable to connect:\n\t{:?}", e);
        }
        info!(self.logger "Connected to broker");
    }
    pub fn disconnect(&self) {
        self.client.disconnect(None);
        info!(self.logger, "Disconnect from the broker");
    }
}

fn main() {
    let mut publisher: Publisher;
    publisher.initialize();
    publisher.connect();

    // Create a message and publish it.
    // Publish message to 'test' and 'hello' topics.
    for num in 0..5 {
        let content =  "Hello world! ".to_string() + &num.to_string();
        let mut msg = mqtt::Message::new(DFLT_TOPICS[0], content.clone(), QOS);
        if num % 2 == 0 {
            println!("Publishing messages on the {:?} topic", DFLT_TOPICS[1]);
            msg = mqtt::Message::new(DFLT_TOPICS[1], content.clone(), QOS);
        } else {
            println!("Publishing messages on the {:?} topic", DFLT_TOPICS[0]);
        }
        let tok = publisher.client.publish(msg);

        if let Err(e) = tok {
            println!("Error sending message: {:?}", e);
            break;
        }
    }


    publisher.disconnect();
}