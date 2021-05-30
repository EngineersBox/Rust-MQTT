use std::{
    time::Duration,
};
use crate::config::config::Config;
use slog::{Logger, Level};
use crate::connector::connector::Connector;
use std::sync::Arc;

///
/// An MQTT publisher for a given set of topics. This will provide a means to
/// instantiate and invoke the necessary flow for sending messages.
///
/// In order to configure the publisher a config and logger should be provided, where the config
/// will utilise the broker registration and connection configurations. See [Config](rust-mqtt::config::config::Config)
///
/// <br/><br/>
///
/// # Usage Flow
/// In order to initialize and run the subscriber the following flow is expected:
/// 1. Create a new instance
/// 2. Initialize instance
/// 3. Connect to broker
/// 4. Send message(s) to broker
/// 5. Disconnect from broker
///
/// # Example
/// ```rust
/// let mut publisher: Publisher = Publisher::new(...);
/// publisher.initialize();
/// publisher.connect();
/// let msg: mqtt::Message = mqtt::Message::new(...);
/// let tok: Result<(), mqtt::Error> = publisher.client.publish(msg);
/// if let Err(e) = tok {...}
/// publisher.disconnect();
/// ```
///
pub struct Publisher {
    config: Arc<Config>,
    pub logger: Logger,
    conn_opts: mqtt::ConnectOptions,
    pub client: mqtt::Client
}

impl Publisher {
    ///
    /// Create a new subscriber with a config and logger.
    /// The config will utilise the broker registration and connection configurations.
    /// See [Config](rust-mqtt::config::Config)
    ///
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
    ///
    /// See the initialize definition in [Connector](rust-mqtt::connector::connector::Connector)
    ///
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
        match self.client.disconnect(None) {
            Ok(()) => {}
            Err(e) => {
                error!(self.logger, "Could not disconnect from broker");
                panic!("{:?}", e);
            }
        };
        info!(self.logger, "Disconnect from the broker");
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
