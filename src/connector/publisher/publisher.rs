use std::{
    time::Duration,
    process,
};
use crate::config::config::Config;
use slog::Logger;
use crate::connector::connector::Connector;
use std::sync::Arc;

pub struct Publisher {
    config: Arc<Config>,
    pub logger: Logger,
    conn_opts: mqtt::ConnectOptions,
    pub client: mqtt::Client
}

impl Publisher {
    pub fn new(config: Arc<Config>, logger: Logger) -> Publisher {
        Publisher {
            config,
            logger,
            conn_opts: Default::default(),
            client: mqtt::Client::new(mqtt::CreateOptions::default()).unwrap(),
        }
    }
}

impl Connector for Publisher {
    fn initialize(&mut self) {
        let create_opts: mqtt::CreateOptions = mqtt::CreateOptionsBuilder::new()
            .server_uri(self.config.broker.clone())
            .client_id(self.config.publisher_connection.id.clone())
            .finalize();
        self.client = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
            panic!("Error creating the client: {:?}", err);
        });
        debug!(self.logger, "Initialised client with options");
        self.conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_millis(self.config.client.keep_alive))
            .clean_session(true)
            .user_name(self.config.creds.username.clone())
            .password(self.config.creds.password.clone())
            .connect_timeout(Duration::from_millis(self.config.client.timeout))
            .finalize();
        debug!(self.logger, "Created connection options");
        info!(self.logger, "Initialised client");
    }
    fn connect(&mut self) {
        if let Err(e) = self.client.connect(self.conn_opts.clone()) {
            panic!("Unable to connect:\n\t{:?}", e);
        }
        info!(self.logger, "Connected to broker");
    }
    fn disconnect(&mut self) {
        self.client.disconnect(None);
        info!(self.logger, "Disconnect from the broker");
    }
}

// fn main() {
    // let mut publisher: Publisher;
    // publisher.initialize();
    // publisher.connect();
    // Create a message and publish it.
    // Publish message to 'test' and 'hello' topics.
    // for num in 0..5 {
    //     let content =  "Hello world! ".to_string() + &num.to_string();
    //     let mut msg = mqtt::Message::new(DFLT_TOPICS[0], content.clone(), QOS);
    //     if num % 2 == 0 {
    //         println!("Publishing messages on the {:?} topic", DFLT_TOPICS[1]);
    //         msg = mqtt::Message::new(DFLT_TOPICS[1], content.clone(), QOS);
    //     } else {
    //         println!("Publishing messages on the {:?} topic", DFLT_TOPICS[0]);
    //     }
    //     let tok = publisher.client.publish(msg);
    //
    //     if let Err(e) = tok {
    //         println!("Error sending message: {:?}", e);
    //         break;
    //     }
    // }
    // publisher.disconnect();
// }