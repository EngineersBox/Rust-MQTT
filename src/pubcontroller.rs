mod config;
mod macros;
mod logging;
mod connector;

use logging::logging::initialize_logging;
use config::config::Config;
use connector::publisher::publisher::Publisher;
use connector::subscriber::subscriber::Subscriber;
use crate::connector::connector::Connector;

#[macro_use]
extern crate slog;
extern crate paho_mqtt as mqtt;
extern crate slog_term;
extern crate slog_async;
extern crate slog_json;
extern crate regex;
extern crate thread_id;

use slog::{Logger, Level};
use std::sync::mpsc::Receiver;
use std::thread;
use mqtt::Message;
use std::sync::Arc;

fn main() {
    let logger: Logger = initialize_logging();
    let config: Arc<Config> = Arc::new(Config::new("resource/config.properties", &logger.new(get_current_thread_id!())));
    // let publisher: Publisher = Publisher::new(config.clone(), &logger);
    thread::spawn({
        let t_logger: Logger = logger.clone();
        let t_config: Arc<Config> = config.clone();
        move || {
            let mut subscriber: Subscriber = Subscriber::new(t_config, t_logger.new(get_current_thread_id!()));
            subscriber.initialize();
            let receiver: Receiver<Option<Message>> = subscriber.consume();
            subscriber.connect();
            subscriber.subscribe_topics(&[1, 1]);

            subscriber.log_at(Level::Info, "Processing requests...");
            for msg in receiver.iter() {
                if let Some(msg) = msg {
                    subscriber.log_at(Level::Info, format!("INCOMING MESSAGE: {}", msg).as_str());
                } else if !subscriber.client.is_connected() {
                    if subscriber.try_reconnect() {
                        subscriber.log_at(Level::Info, "Resubscribing to topics...");
                        subscriber.subscribe_topics(&[1, 1]);
                    } else {
                        break;
                    }
                }
            }
            subscriber.disconnect();
        }
    });
    thread::spawn({
        let t_logger: Logger = logger.clone();
        let t_config: Arc<Config> = config.clone();
        move || {
            let mut publisher: Publisher = Publisher::new(t_config, t_logger.new(get_current_thread_id!()));
            publisher.initialize();
            publisher.connect();
            for num in 0..5 {
                let content =  "Hello world! ".to_string() + &num.to_string();
                publisher.log_at(Level::Debug, format!("Message: {:?}", content.clone()).as_str());
                let msg = mqtt::Message::new("request/qos", content.clone(), 1);
                publisher.log_at(Level::Debug, "Publishing messages on the request/qos topic");
                let tok = publisher.client.publish(msg);

                if let Err(e) = tok {
                    publisher.log_at(Level::Error, format!("Error sending message: {:?}", e).as_str());
                    break;
                }
            }
            publisher.disconnect();
        }
    });
    loop {}
}