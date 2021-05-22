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
use std::sync::mpsc::{Sender, Receiver};
use std::sync::mpsc;
use std::thread;
use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;
use chrono::Utc;

struct QOSDelayMessage(i32, i32);

fn create_subscriber_thread(logger: &Logger, config: Arc<Config>, tx: Sender<QOSDelayMessage>) -> JoinHandle<()> {
    thread::spawn({
        let t_logger: Logger = logger.clone();
        let t_tx: Sender<QOSDelayMessage> = tx.clone();
        move || {
            let mut subscriber: Subscriber = Subscriber::new(config.clone(), t_logger.new(get_current_thread_id!()));
            subscriber.initialize();
            let receiver: Receiver<Option<mqtt::Message>> = subscriber.consume();
            subscriber.connect();
            subscriber.subscribe_topics(&[2, 2]);

            subscriber.log_at(Level::Info, "Processing requests...");
            receiver.iter().for_each(|msg: Option<mqtt::Message>| {
                if let Some(msg) = msg {
                    subscriber.log_at(Level::Info, format!("Received [Message: {}] [Topic: {}] [QoS: {}]", msg.payload_str(), msg.topic(), msg.qos()).as_str());
                    if msg.topic() == config.subscriber_connection.topics.get(0).unwrap().as_str() {
                        try_except_with_log_action!(t_tx.send(QOSDelayMessage(msg.qos(), -1)), Level::Error, "Could not send message to publisher thread", t_tx, subscriber);
                    } else if msg.topic() == config.subscriber_connection.topics.get(1).unwrap().as_str() {
                        try_except_with_log_action!(t_tx.send(QOSDelayMessage(-1, msg.payload_str().parse::<i32>().unwrap())), Level::Error, "Could not send message to publisher thread", t_tx, subscriber);
                    }
                } else if !subscriber.client.is_connected() {
                    if subscriber.try_reconnect() {
                        subscriber.log_at(Level::Info, "Resubscribing to topics...");
                        subscriber.subscribe_topics(&[2, 2]);
                    } else {
                        drop(t_tx.clone());
                        return;
                    }
                }
            });
            subscriber.disconnect();
        }
    })
}

fn create_publisher_thread(logger: &Logger, config: Arc<Config>, rx: Receiver<QOSDelayMessage>) -> JoinHandle<()> {
    thread::spawn({
        let t_logger: Logger = logger.clone();
        let mut c_qos: i32 = 0;
        let mut c_delay: i32 = 0;
        move || {
            let mut publisher: Publisher = Publisher::new(config.clone(), t_logger.new(get_current_thread_id!()));
            publisher.initialize();
            publisher.connect();
            #[allow(irrefutable_let_patterns)]
            while let QOSDelayMessage(q, d) = match rx.recv() {
                Ok(v) => v,
                Err(e) => {
                    publisher.log_at(Level::Critical, format!("Could not receive qos/delay message").as_str());
                    panic!("{:?}", e);
                }
            } {
                message_range_check!((0..=2), q, "QoS", c_qos, publisher);
                message_range_check!((0..=500), d, "Delay", c_delay, publisher);
                let topic: String = match config.clone().publisher_connection.topics.get(0) {
                    Some(v) => v.replace("{qos}", format!("{}", c_qos).as_str()).replace("{delay}", format!("{}", c_delay).as_str()),
                    None => {
                        publisher.log_at(Level::Critical, "No topic was specified for the publisher");
                        panic!("Missing required topic to format");
                    }
                };
                for idx in 0..config.publisher_connection.message_quantity {
                    let msg: mqtt::Message = mqtt::Message::new(topic.clone(), format!("{} :: {:?}", idx, Utc::now()), c_qos);
                    publisher.log_at(Level::Info, format!("Published [Message: {}] [Topic: {}] [QoS: {}]", msg.payload_str(), topic, c_qos).as_str());
                    let tok: Result<(), mqtt::Error> = publisher.client.publish(msg);
                    if let Err(e) = tok {
                        publisher.log_at(Level::Error, format!("Error sending message: {:?}", e).as_str());
                        break;
                    }
                    thread::sleep(Duration::from_millis(c_delay as u64))
                }
            }
            publisher.disconnect();
        }
    })
}

fn main() {
    let logger: Logger = initialize_logging(String::from("pubcontroller_"));
    let config: Arc<Config> = Arc::new(Config::new("resource/pubcontroller.properties", &logger.new(get_current_thread_id!())));
    let (tx, rx): (Sender<QOSDelayMessage>, Receiver<QOSDelayMessage>) = mpsc::channel();
    let mut threads: Vec<JoinHandle<()>> = Vec::with_capacity(2);

    threads.push(create_subscriber_thread(&logger, config.clone(), tx));
    threads.push(create_publisher_thread(&logger, config.clone(), rx));
    let thread_logger: Logger = logger.new(get_current_thread_id!());
    join_threads!(threads, thread_logger);
}