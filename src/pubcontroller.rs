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

use slog::Logger;
use std::sync::Arc;
use std::sync::mpsc::{Iter, Receiver};
use std::{process, thread};
use mqtt::Message;
use std::borrow::{Borrow, BorrowMut};
use futures::executor::block_on;

fn main() {
    let logger: Logger = initialize_logging();
    let config: Arc<Config> = Arc::new(Config::new("resource/config.properties", &logger));
    // let publisher: Publisher = Publisher::new(config.clone(), &logger);
    let mut subscriber: Subscriber = Subscriber::new(config.clone(), &logger);

    let pc_logger: Logger = logger.new(o!("Subscriber" => process::id()));
;
    subscriber.initialize();
    let rx: Receiver<Option<Message>> = subscriber.consume();
    subscriber.connect();
    subscriber.subscribe_topics(&[1]);

    info!(pc_logger, "Processing requests...");
    for msg in rx.iter() {
        if let Some(msg) = msg {
            println!("{}", msg);
        } else if !subscriber.client.is_connected() {
            if subscriber.try_reconnect() {
                info!(pc_logger, "Resubscribe topics...");
                subscriber.subscribe_topics(&[1]);
            } else {
                break;
            }
        }
    }
    subscriber.disconnect();
}