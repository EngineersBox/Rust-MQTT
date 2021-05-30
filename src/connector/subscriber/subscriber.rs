use std::{
    thread,
    time::Duration
};
use crate::config::config::Config;
use slog::{Logger, Level};
use crate::connector::connector::Connector;
use std::sync::mpsc::Receiver;

use std::sync::Arc;

///
/// An MQTT subscriber for a given set of topics. This will provide a means to
/// instantiate and invoke the necessary flow for handling subscriptions.
///
/// In order to configure the subscriber a config and logger should be provided, where the config
/// will utilise the broker registration and connection configurations. See [Config](rust-mqtt::config::config::Config)
///
/// <br/><br/>
///
/// # Usage Flow
/// In order to initialize and run the subscriber the following flow is expected:
/// 1. Create a new instance
/// 2. Store receiver instance
/// 3. Connect to broker
/// 4. Subscribe to topics at specified QoS levels
/// 5. Process message(s) in receiver iterator
/// 6. Disconnect from broker
///
/// # Example
/// ```rust
/// let mut subscriber: Subscriber = Subscriber::new(...);
/// subscriber.initialize();
/// let receiver: Receiver<Option<mqtt::Message>> = subscriber.consume();
/// subscriber.connect();
/// subscriber.subscribe_topics(&[qos...]);
/// receiver.iter().for_each(|msg: Option<mqtt::Message>| {...});
/// subscriber.disconnect();
/// ```
///
pub struct Subscriber {
    config: Arc<Config>,
    pub logger: Logger,
    conn_opts: mqtt::ConnectOptions,
    pub subscribed_topics: Vec<String>,
    pub client: mqtt::Client,
}

impl Subscriber {
    ///
    /// Create a new subscriber with a config and logger.
    /// The config will utilise the broker registration and connection configurations.
    /// See [Config](rust-mqtt::config::Config)
    ///
    pub fn new(config: Arc<Config>, logger: Logger) -> Subscriber {
        Subscriber {
            config: config.clone(),
            logger,
            conn_opts: Default::default(),
            subscribed_topics: config.subscriber_connection.topics.clone(),
            client: mqtt::Client::new(mqtt::CreateOptions::default()).unwrap(),
        }
    }
    ///
    /// Attempt a reconnection to the broker, retrying `n` times defined in the config used
    /// to initialize the subscriber instance
    ///
    /// # Returns
    /// * Reconnection state: `true` if reconnect was successful, `false` otherwise
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
    ///
    /// Subscribe to the topics provided by the configuration at given QoS levels
    ///
    /// # Arguments
    /// * Array of QoS: An array of QoS levels to subscribe to the configured topics at, these will be
    /// on a per index basis where the order of indexes matches the order of defined topics in the config
    ///
    /// # Returns
    /// * Subscription state: `true` if subscription was successful, `false` otherwise
    pub fn subscribe_topics(&self, qos: &[i32]) -> bool {
        if let Err(e) = self.client.subscribe_many(self.subscribed_topics.as_slice(), qos) {
            error!(self.logger, "Could not subscribe to topics {:?}: {:?}", self.subscribed_topics, e);
            return false;
        }
        info!(self.logger, "Subscribed to topics {:?} for QoS {:?}", self.subscribed_topics, qos);
        true
    }
    ///
    /// Invoke the consumer for accepting messages for the subscribed topics. These will be provided
    /// via a blocking iterator that can be called in a loop.
    ///
    /// # Example
    /// ``` rust
    /// let rec = sub.consume();
    /// for msg in rec.iter() {
    ///     // ...
    /// }
    /// ```
    ///
    /// # Returns
    /// * Receiver<Option<Message>>: A receiver that provides [Message](paho_mqtt::Message) via a blocking iterator
    pub fn consume(&mut self) -> Receiver<Option<mqtt::Message>> {
        return self.client.start_consuming();
    }
}

impl Connector for Subscriber {
    ///
    /// See the initialize definition in [Connector](rust-mqtt::connector::connector::Connector)
    ///
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
    ///
    /// See the initialize definition in [Connector](rust-mqtt::connector::connector::Connector)
    ///
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
    ///
    /// See the initialize definition in [Connector](rust-mqtt::connector::connector::Connector)
    ///
    fn disconnect(&mut self) {
        if self.client.is_connected() {
            self.client.unsubscribe_many(self.subscribed_topics.as_slice()).unwrap();
            self.client.disconnect(None).unwrap();
            info!(self.logger, "Disconnected from the broker");
        } else {
            info!(self.logger, "Already disconnected from broker, ignoring disconnect call")
        }
    }
    ///
    /// See the initialize definition in [Connector](rust-mqtt::connector::connector::Connector)
    ///
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
