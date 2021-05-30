use std::collections::HashMap;
use java_properties::read;
use std::fs::File;
use std::io::BufReader;
use crate::try_except_return_default;

use crate::config::exceptions;
use std::path::Path;
use regex::Regex;
use slog::Logger;
use std::str::FromStr;

///
/// Credentials to use to connection to the broker
///
pub struct Credentials {
    pub username: String,
    pub password: String,
}

///
/// A set of properties for a paho MQTT client
/// * `keep_alive`: How long persistent connections should last with inactivity
/// * `timeout`: Duration for terminating a connection with idle state
/// * `clean_session`: Whether to persist a previous cached session (ID, queued messages, etc)
///
pub struct Client {
    pub keep_alive: u64,
    pub timeout: u64,
    pub clean_session: bool,
}

///
/// A set of properties for a subscriber
/// * `id`: Client ID to register with the broker (unique)
/// * `retries`: How many times to retry a reconnect to the broker
/// * `retry_duration`: How often to perform a reconnection in milliseconds
/// * `topics`: Which topics to subscribe to
///
pub struct SubscriberConnection {
    pub id: String,
    pub retries: u64,
    pub retry_duration: u64,
    pub topics: Vec<String>,
}

///
/// A set of properties for a publisher:
/// * `id`: Client ID to register with the broker (unique)
/// * `topics`: Which topics to send to
/// * `message_quantity`: Number of messages to send relative to time period
///
pub struct PublisherConnection {
    pub id: String,
    pub topics: Vec<String>,
    pub message_quantity: i32,
}
///
/// Defines a set of configuration properties used by subscribers and publishers.
///
pub struct Config {
    pub broker: String,
    pub creds: Credentials,
    pub client: Client,
    pub subscriber_connection: SubscriberConnection,
    pub publisher_connection: PublisherConnection,
}

///
/// Reads a `.properties` file and creates a HashMap of key-value pairs
///
/// # Arguments
/// * filename: Location of the file relative to the crate
/// * logger: Logger instance to write to
///
/// # Returns
/// * HashMap<String, String>: Key-value pairs relative to the properties file
///
fn read_config_file(filename: &str, logger: &Logger) -> HashMap<String, String> {
    let path: &Path = Path::new(filename);
    let file: File = match File::open(&path) {
        Err(_) => panic!("{}", exceptions::FileError{ filename: String::from(filename) }),
        Ok(file) => file,
    };

    try_except_return_default! {
        read(BufReader::new(file)),
        "Could not read properties",
        HashMap::new(),
        logger
    }
}

///
/// Retrieve a value for a given key in the provided properties HashMap
///
/// # Type Arguments:
/// * `T`: Type with the `FromStr` trait
///
/// # Arguments
/// * properties: HashMap<String, String> of key-value pairs
/// * key: Key to retrieve the value of from the properties map instance
/// * logger: Logger instance to log to
///
/// # Returns
/// * `T` parsed version of the value, this will panic if the parsing fails
///
fn get_property<T: FromStr>(properties: &HashMap<String, String>, key: &str, logger: &Logger) -> T {
    if key.is_empty() {
        panic!(exceptions::ConfigPropertiesError::InvalidConfigPropertyKeyError{
            0:exceptions::InvalidConfigPropertyKeyError{key: String::from(key)},
        });
    }
    let value: Option<&String> = properties.get(key);
    if value.is_none() {
        error!(logger, "Could not find property: {}", key);
        panic!(exceptions::ConfigPropertiesError::MissingConfigPropertyError{
            0:exceptions::MissingConfigPropertyError{property: String::from(key)},
        });
    }
    let parsed_value = value.unwrap().parse::<T>();
    match parsed_value {
        Ok(v) => return v,
        Err(_) => panic!("Could not parse value for config: {}", key)
    }
}

impl Config {
    ///
    /// Creates a new config instance based on a given file path and a logger
    ///
    /// # Arguments
    /// * filename: Location of the `.properties` file to retrieve config properties from
    /// * logger: Logger instance to log to
    ///
    /// # Returns
    /// * Instance of Config with saturated fields based on config properties. Will panic if a property does not exist
    pub fn new(filename: &str, logger: &Logger) -> Config {
        let properties: HashMap<String, String> = read_config_file(filename ,logger);
        let list_split_regex: Regex = Regex::new(r",(\s)?").expect("Could not compile regex");
        Config {
            broker: format!(
                "tcp://{}:{}",
                get_property::<String>(&properties, "broker.host", logger),
                get_property::<String>(&properties, "broker.port", logger),
            ),
            creds: Credentials {
                username: get_property::<String>(&properties, "creds.username", logger),
                password: get_property::<String>(&properties, "creds.password", logger),
            },
            client: Client {
                keep_alive:  get_property::<u64>(&properties, "client.keep_alive", logger),
                timeout: get_property::<u64>(&properties, "client.timeout", logger),
                clean_session: get_property::<bool>(&properties, "client.clean_session", logger),
            },
            subscriber_connection: SubscriberConnection {
                id: get_property::<String>(&properties, "subscriber_connection.id", logger),
                retries: get_property::<u64>(&properties, "subscriber_connection.retries", logger),
                retry_duration: get_property::<u64>(&properties, "subscriber_connection.retry_duration", logger),
                topics: list_split_regex.split(get_property::<String>(&properties, "subscriber_connection.topics", logger).as_str()).map(|p| String::from(p)).collect::<Vec<String>>(),
            },
            publisher_connection: PublisherConnection {
                id: get_property::<String>(&properties, "publisher_connection.id", logger),
                topics: list_split_regex.split(get_property::<String>(&properties, "publisher_connection.topics", logger).as_str()).map(|p| String::from(p)).collect::<Vec<String>>(),
                message_quantity: get_property::<i32>(&properties, "publisher_connection.message_quantity", logger),
            }
        }
    }
}