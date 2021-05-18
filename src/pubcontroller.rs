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

type QOSDelayMessage = [i32; 2];

fn create_subscriber_thread(logger: &Logger, config: Arc<Config>, tx: Sender<QOSDelayMessage>) -> JoinHandle<()> {
    thread::spawn({
        let t_logger: Logger = logger.clone();
        let t_config: Arc<Config> = config.clone();
        let t_tx: Sender<QOSDelayMessage> = tx.clone();
        move || {
            let mut subscriber: Subscriber = Subscriber::new(t_config.clone(), t_logger.new(get_current_thread_id!()));
            subscriber.initialize();
            let receiver: Receiver<Option<mqtt::Message>> = subscriber.consume();
            subscriber.connect();
            subscriber.subscribe_topics(&[1, 1]);

            #[macro_export]
            macro_rules! try_except_with_log_action {
                ($matcher:expr, $level:expr, $msg:literal) => {
                    match $matcher {
                        Ok(value) => value,
                        Err(e) => {
                            subscriber.log_at($level,  format!("{}: {}", $msg, e).as_str());
                            return;
                        },
                    }
                }
            }
            subscriber.log_at(Level::Info, "Processing requests...");
            receiver.iter().for_each(|msg: Option<mqtt::Message>| {
                if let Some(msg) = msg {
                    subscriber.log_at(Level::Debug, format!("Received message: [Topic: {}] [QoS: {}]", msg.topic(), msg.qos()).as_str());
                    if msg.topic() == t_config.subscriber_connection.topics.get(0).unwrap().as_str() {
                        try_except_with_log_action!(t_tx.send([msg.qos(), -1]), Level::Error, "Could not send message to publisher thread");
                    } else if msg.topic() == t_config.subscriber_connection.topics.get(1).unwrap().as_str() {
                        try_except_with_log_action!(t_tx.send([-1, msg.payload_str().parse::<i32>().unwrap()]), Level::Error, "Could not send message to publisher thread");
                    }
                } else if !subscriber.client.is_connected() {
                    if subscriber.try_reconnect() {
                        subscriber.log_at(Level::Info, "Resubscribing to topics...");
                        subscriber.subscribe_topics(&[1, 1]);
                    } else {
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
        let t_config: Arc<Config> = config.clone();
        let mut c_qos: i32 = 0;
        let mut c_delay: i32 = 0;
        move || {
            let mut publisher: Publisher = Publisher::new(t_config, t_logger.new(get_current_thread_id!()));
            macro_rules! message_range_check {
                ($range:expr, $target_value:expr, $target_name:expr, $current:expr) => {
                    if $range.contains(&$target_value) {
                        $current = $target_value;
                    } else if $target_value != -1 {
                        publisher.log_at(Level::Error, format!("{} was not within range {:?}: {}", $target_name, $range, $target_value).as_str());
                        continue;
                    }
                }
            }
            publisher.initialize();
            publisher.connect();
            #[allow(irrefutable_let_patterns)]
            while let rec_msg = match rx.recv() {
                Ok(v) => v,
                Err(e) => {
                    publisher.log_at(Level::Error, format!("Could not receive qos/delay message").as_str());
                    panic!("{:?}", e);
                }
            } {
                message_range_check!((0..=2), rec_msg[0], "QoS", c_qos);
                message_range_check!((0..=500), rec_msg[1], "Delay", c_delay);
                let topic: String = format!("counter/{}/{}", c_qos, c_delay);
                let msg: mqtt::Message = mqtt::Message::new(topic.clone(), topic.clone(), c_qos);
                publisher.log_at(Level::Info, format!("Published message [Topic: {}] [QoS: {}]", topic, c_qos).as_str());
                let tok: Result<(), mqtt::Error> = publisher.client.publish(msg);
                if let Err(e) = tok {
                    publisher.log_at(Level::Error, format!("Error sending message: {:?}", e).as_str());
                    break;
                }
            }
            publisher.disconnect();
        }
    })
}

fn main() {
    let logger: Logger = initialize_logging();
    let config: Arc<Config> = Arc::new(Config::new("resource/pubcontroller.properties", &logger.new(get_current_thread_id!())));
    let (tx, rx): (Sender<QOSDelayMessage>, Receiver<QOSDelayMessage>) = mpsc::channel();
    let mut threads: Vec<JoinHandle<()>> = Vec::with_capacity(2);

    threads.push(create_subscriber_thread(&logger, config.clone(), tx));
    threads.push(create_publisher_thread(&logger, config.clone(), rx));
    let thread_logger: Logger = logger.new(get_current_thread_id!());
    for t in threads {
        match t.join() {
            Ok(_) => {}
            Err(e) => {
                error!(thread_logger, "Handler thread panicked while joining");
                panic!("Join panic reason: {:?}", e);
            }
        }
    }
}