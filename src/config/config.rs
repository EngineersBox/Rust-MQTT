use std::collections::HashMap;
use java_properties::read;
use std::fs::File;
use std::io::BufReader;
use crate::try_except_return_default;

use crate::configuration::exceptions;
use std::path::Path;
use regex::Regex;
use std::ops::Add;

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

fn read_config_file(filename: &str) -> HashMap<String, String> {
    let path: &Path = Path::new(filename);
    let file: File = match File::open(&path) {
        Err(_) => panic!("{}", exceptions::FileError{filename}),
        Ok(file) => file,
    };

    try_except_return_default! {
            read(BufReader::new(file)),
            "Could not read properties",
            HashMap::new()
        }
}

fn get_property<T>(properties: &HashMap<String, String>, key: &str) -> T {
    if key.is_empty() {
        return panic!(exceptions::ConfigPropertiesError::InvalidConfigPropertyKeyError{
            0:exceptions::InvalidConfigPropertyKeyError{key},
        });
    }
    let value: Option<&String> = properties.get(key);
    if value.is_none() {
        panic!(exceptions::ConfigPropertiesError::MissingConfigPropertyError{
            0:exceptions::MissingConfigPropertyError{property: key},
        });
    }
    return (*value.unwrap()).clone().into();
}

impl Config {
    pub fn new(filename: &str) -> Config {
        let properties: HashMap<String, String> = read_config_file(filename);
        let list_split_regex: Regex = Regex::new(r",(\s)?").expect("Could not compile regex");
        Config {
            broker: String::from("tcp://")
                .add(get_property::<&str>(&properties, "broker.host"))
                .add(get_property::<&str>(&properties, "broker.port")),
            creds: Credentials {
                username: get_property::<String>(&properties, "creds.username"),
                password: get_property::<String>(&properties, "creds.password"),
            },
            client: Client {
                id: get_property::<String>(&properties, "client.id"),
                keep_alive:  get_property::<u64>(&properties, "client.keep_alive"),
                timeout: get_property::<u64>(&properties, "client.timeout"),
            },
            subscriber_connection: SubscriberConnection {
                retries: get_property::<u64>(&properties, "subscriber_connection.retries"),
                retry_duration: get_property::<u64>(&properties, "subscriber_connection.retry_duration"),
                topics: list_split_regex.split(get_property::<&str>(&properties, "subscriber_connection.topics")).collect::<Vec<String>>(),
            },
            publisher_connection: PublisherConnection {
                topics: list_split_regex.split(get_property::<&str>(&properties, "publisher_connection.topics")).collect::<Vec<String>>(),
            }
        }
    }
}