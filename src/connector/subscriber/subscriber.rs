use std::{
    thread,
    time::Duration
};
use crate::config::config::Config;
use slog::{Logger, Level};
use crate::connector::connector::Connector;
use std::sync::mpsc::Receiver;

use std::sync::Arc;

pub struct Subscriber {
    config: Arc<Config>,
    pub logger: Logger,
    conn_opts: mqtt::ConnectOptions,
    pub subscribed_topics: Vec<String>,
    pub client: mqtt::Client,
}

impl Subscriber {
    pub fn new(config: Arc<Config>, logger: Logger) -> Subscriber {
        Subscriber {
            config: config.clone(),
            logger,
            conn_opts: Default::default(),
            subscribed_topics: config.subscriber_connection.topics.clone(),
            client: mqtt::Client::new(mqtt::CreateOptions::default()).unwrap(),
        }
    }
    pub fn try_reconnect(&self) -> bool {
        info!(self.logger, "Connection lost. Attempting to reconnect");
        for i in 0..self.config.subscriber_connection.retries {
            thread::sleep(Duration::from_millis(self.config.subscriber_connection.retry_duration));
            info!(self.logger, "Reconnect attempt {} of {}", i + 1, self.config.subscriber_connection.retries);
            if self.client.reconnect().is_ok() {
                info!(self.logger, "Successfully reconnected");
                return true;
            }
        }
        error!(self.logger, "Unable to reconnect after {} attempts.", self.config.subscriber_connection.retries);
        false
    }
    pub fn subscribe_topics(&self, qos: &[i32]) -> bool {
        if let Err(e) = self.client.subscribe_many(self.subscribed_topics.as_slice(), qos) {
            error!(self.logger, "Could not subscribe to topics {:?}: {:?}", self.subscribed_topics, e);
            return false;
        }
        info!(self.logger, "Subscribed to topics {:?} for QoS {:?}", self.subscribed_topics, qos);
        true
    }
    pub fn consume(&mut self) -> Receiver<Option<mqtt::Message>> {
        return self.client.start_consuming();
    }
}

impl Connector for Subscriber {
    fn initialize(&mut self) {
        let create_opts: mqtt::CreateOptions = mqtt::CreateOptionsBuilder::new()
            .server_uri(self.config.broker.clone())
            .client_id(self.config.subscriber_connection.id.clone())
            .finalize();
        self.client = mqtt::Client::new(create_opts).unwrap_or_else(|err| {
            panic!("Error creating the client: {:?}", err);
        });
        debug!(self.logger, "Initialised client with options");
        let lwt = mqtt::MessageBuilder::new()
            .topic("test")
            .payload("Consumer lost connection")
            .finalize();
        self.conn_opts = mqtt::ConnectOptionsBuilder::new()
            .keep_alive_interval(Duration::from_millis(self.config.client.keep_alive))
            .clean_session(self.config.client.clean_session)
            .user_name(self.config.creds.username.clone())
            .password(self.config.creds.password.clone())
            .connect_timeout(Duration::from_millis(self.config.client.timeout))
            .will_message(lwt)
            .finalize();
        debug!(self.logger, "Created connection options");
        info!(self.logger, "Initialised client with id: {}", self.config.subscriber_connection.id.clone());
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
        if self.client.is_connected() {
            self.client.unsubscribe_many(self.subscribed_topics.as_slice()).unwrap();
            self.client.disconnect(None).unwrap();
            info!(self.logger, "Disconnected from the broker");
        } else {
            info!(self.logger, "Already disconnected from broker, ignoring disconnect call")
        }
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
    // let mut subscriber: Subscriber;
    // subscriber.initialize();
    // subscriber.connect();
    // subscriber.subscribe_topics();
    //
    // println!("Processing requests...");
    // for msg in subscriber.consumer.unwrap().iter() {
    //     if let Some(msg) = msg {
    //         println!("{}", msg);
    //     } else if !subscriber.client.is_connected() {
    //         if subscriber.try_reconnect() {
    //             println!("Resubscribe topics...");
    //             // subscriber.subscribe_topics();
    //         } else {
    //             break;
    //         }
    //     }
    // }
    //
    // subscriber.disconnect();
// }