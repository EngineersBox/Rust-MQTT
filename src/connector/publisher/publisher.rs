use std::{
    time::Duration,
};
use crate::config::config::Config;
use slog::{Logger, Level};
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
            error!(self.logger, "Could not create client");
            panic!("{:?}", err);
        });
        debug!(self.logger, "Initialised client with options");
        self.conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_millis(self.config.client.keep_alive))
            .clean_session(self.config.client.clean_session)
            .user_name(self.config.creds.username.clone())
            .password(self.config.creds.password.clone())
            .connect_timeout(Duration::from_millis(self.config.client.timeout))
            .finalize();
        debug!(self.logger, "Created connection options");
        info!(self.logger, "Initialised client with id: {}", self.config.publisher_connection.id.clone());
    }
    fn connect(&mut self) {
        match self.client.connect(self.conn_opts.clone()) {
            Ok(rsp) => {
                if let Some(conn_rsp) = rsp.connect_response() {
                    info!(
                        self.logger,
                        "Connected to '{}' with MQTT version {}",
                        conn_rsp.server_uri, conn_rsp.mqtt_version
                    );
                }
            }
            Err(e) => {
                error!(self.logger, "Unable to connect to [{}]: {:?}", self.config.broker, e);
                panic!("{:?}", e);
            }
        }
    }
    fn disconnect(&mut self) {
        match self.client.disconnect(None) {
            Ok(()) => {}
            Err(e) => {
                error!(self.logger, "Could not disconnect from broker");
                panic!("{:?}", e);
            }
        };
        info!(self.logger, "Disconnect from the broker");
    }
    fn log_at(&self, level: Level, msg: &str) {
        match level {
            Level::Critical => crit!(self.logger, "{}", msg),
            Level::Error => error!(self.logger, "{}", msg),
            Level::Warning => warn!(self.logger, "{}", msg),
            Level::Info => info!(self.logger, "{}", msg),
            Level::Debug => debug!(self.logger, "{}", msg),
            Level::Trace => trace!(self.logger, "{}", msg),
        }
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