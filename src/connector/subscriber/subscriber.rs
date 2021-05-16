use std::{
    thread,
    process,
    time::Duration
};
use crate::config::config::Config;
use slog::Logger;
use crate::connector::connector::Connector;
use std::sync::mpsc::Receiver;

pub struct Subscriber {
    config: Box<Config>,
    logger: Logger,
    conn_opts: mqtt::ConnectOptions,
    subscribed_topics: Vec<String>,
    pub client: mqtt::Client,
    pub consumer: Option<Receiver<Option<mqtt::Message>>>
}

impl Subscriber {
    pub fn new(config: Box<Config>, logger: &Logger) -> Subscriber {
        Subscriber {
            config,
            logger: logger.new(o!("Subscriber" => process::id())),
            conn_opts: Default::default(),
            subscribed_topics: vec!(),
            client: mqtt::Client::new(mqtt::CreateOptions::default()).unwrap(),
            consumer: Option::None,
        }
    }
    fn try_reconnect(&self) -> bool {
        info!(self.logger, "Connection lost. Waiting to retry connection");
        for _ in 0..self.config.subscriber_connection.retries {
            thread::sleep(Duration::from_millis(self.config.subscriber_connection.retry_duration));
            if self.client.reconnect().is_ok() {
                info!(self.logger, "Successfully reconnected");
                return true;
            }
        }
        error!(self.logger, "Unable to reconnect after several attempts.");
        false
    }
    fn subscribe_topics(&mut self, topics: &Vec<String>, qos: &[i32]) -> bool {
        if let Err(e) = self.client.subscribe_many(topics.as_slice(), qos) {
            error!(self.logger, "Could not subscribe to topics {:?}: {:?}", topics, e);
            return false;
        }
        info!(self.logger, "Subscribed to topics {:?} for QoS {:?}", topics, qos);
        self.subscribed_topics = topics.clone();
        true
    }
}

impl Connector for Subscriber {
    fn initialize(&mut self) {
        let create_opts: mqtt::CreateOptions = mqtt::CreateOptionsBuilder::new()
            .server_uri(self.config.broker.clone())
            .client_id(self.config.client.id.clone())
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
            .clean_session(true)
            .user_name(self.config.creds.username.clone())
            .password(self.config.creds.password.clone())
            .connect_timeout(Duration::from_millis(self.config.client.timeout))
            .will_message(lwt)
            .finalize();
        debug!(self.logger, "Created connection options");
        info!(self.logger, "Initialised client");
    }

    fn connect(&mut self) {
        self.consumer = Option::Some(self.client.start_consuming());
        if let Err(e) = self.client.connect(self.conn_opts.clone()) {
            panic!("Unable to connect:\n\t{:?}", e);
        }
        info!(self.logger, "Connected to broker");
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
}

// fn main() {
    // let mut subscriber: Subscriber;
    // subscriber.initialize();
    // subscriber.connect();
    // // subscriber.subscribe_topics();
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