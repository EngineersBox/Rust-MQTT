use std::collections::HashMap;
use java_properties::read;
use std::fs::File;
use std::io::BufReader;
use crate::try_except_return_default;

use crate::config::exceptions;
use std::path::Path;
use regex::Regex;
use std::ops::Add;
use slog::Logger;
use std::str::FromStr;

pub struct Credentials {
    pub username: String,
    pub password: String,
}

pub struct Client {
    pub id: String,
    pub keep_alive: u64,
    pub timeout: u64,
}

pub struct SubscriberConnection {
    pub retries: u64,
    pub retry_duration: u64,
    pub topics: Vec<String>,
}

pub struct PublisherConnection {
    pub topics: Vec<String>,
}

pub struct Config {
    pub broker: String,
    pub creds: Credentials,
    pub client: Client,
    pub subscriber_connection: SubscriberConnection,
    pub publisher_connection: PublisherConnection,
}

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
                id: get_property::<String>(&properties, "client.id", logger),
                keep_alive:  get_property::<u64>(&properties, "client.keep_alive", logger),
                timeout: get_property::<u64>(&properties, "client.timeout", logger),
            },
            subscriber_connection: SubscriberConnection {
                retries: get_property::<u64>(&properties, "subscriber_connection.retries", logger),
                retry_duration: get_property::<u64>(&properties, "subscriber_connection.retry_duration", logger),
                topics: list_split_regex.split(get_property::<String>(&properties, "subscriber_connection.topics", logger).as_str()).map(|p| String::from(p)).collect::<Vec<String>>(),
            },
            publisher_connection: PublisherConnection {
                topics: list_split_regex.split(get_property::<String>(&properties, "publisher_connection.topics", logger).as_str()).map(|p| String::from(p)).collect::<Vec<String>>(),
            }
        }
    }
}