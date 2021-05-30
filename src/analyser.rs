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
use std::thread;
use std::sync::{Arc, mpsc};
use std::thread::JoinHandle;
use std::time::Duration;
use regex::Regex;
use lazy_static::lazy_static;

lazy_static! {
    static ref MODULE_SEPARATOR_REGEX: Regex = Regex::new(r"\s::\s").expect("Could not compile module separator regex");
}

///
/// Message to be send in channel between publisher and subscriber
///
/// # Structure
/// 1. QoS level
/// 2. Delay in milliseconds
///
#[derive(Debug)]
struct QOSDelayMessage(i32, i32);

///
/// Verifies whether the message is final based on whether the index matches the message count defined
/// in the supplied config instance
///
/// # Arguments
/// * msg: String message to verify as last or not
/// * config: Config instance with message count defined
///
/// # Returns
/// * `true` if is last message (`index == message_quantity - 1`), `false` otherwise
fn is_final_message(msg: &str, config: Arc<Config>) -> bool {
    if !MODULE_SEPARATOR_REGEX.is_match(msg) {
        return false;
    }
    return MODULE_SEPARATOR_REGEX.split(msg)
        .map(String::from)
        .collect::<Vec<String>>()[0]
        .parse::<i32>()
        .expect("Could not parse message index") == (config.publisher_connection.message_quantity - 1);
}

///
/// Create a thread with a subscriber initialized within. This will send `n` messages to the
// QoS and Delay levels provided by the received messages from the publisher via the channel instance.
///
/// # Arguments
/// * logger: Logger instance to log to
/// * config: Configuration to use to initialize the subscriber
/// * rx: Receiver channel instance to receive changes to QoS and Delay
///
/// # Returns
/// * `JoinHandle<()>` for joining thread as blocking
///
fn create_subscriber_thread(logger: &Logger, config: Arc<Config>, rx: Receiver<QOSDelayMessage>) -> JoinHandle<()> {
    thread::spawn({
        // Clone this instances since they will be moving scope and will need to persist for the lifetime of the thread
        let t_logger: Logger = logger.clone();
        let mut c_qos: i32 = 0;
        let mut c_delay: i32 = 0;
        move || {
            let mut subscriber: Subscriber = Subscriber::new(config.clone(), t_logger.new(get_current_thread_id!()));
            #[allow(irrefutable_let_patterns)]
            while let QOSDelayMessage(q, d) = match rx.recv() {
                Ok(v) => {
                    info!(t_logger, "Received thead message: {:?}", v);
                    v
                },
                Err(e) => {
                    crit!(t_logger, "Could not receive qos/delay message");
                    panic!("{:?}", e);
                }
            } {
                // Verify the QoS is in range and update if so, panic otherwise
                message_range_check!((0..=2), q, "QoS", c_qos, subscriber);
                // Verify the delay is in range and update if so, panic otherwise
                message_range_check!((0..=500), d, "Delay", c_delay, subscriber);
                subscriber = Subscriber::new(config.clone(), t_logger.new(get_current_thread_id!()));
                subscriber.subscribed_topics = match config.clone().subscriber_connection.topics.get(0) {
                    /// Here we format the topic from `counter/{qos}/{delay}` by replacing `{qos}` and `{delay}` with their respective values
                    Some(v) => vec![v.replace("{qos}", format!("{}", c_qos).as_str()).replace("{delay}", format!("{}", c_delay).as_str())],
                    None => {
                        subscriber.log_at(Level::Critical, "No topic was specified for the publisher");
                        panic!("Missing required topic to format");
                    }
                };
                subscriber.initialize();
                let receiver: Receiver<Option<mqtt::Message>> = subscriber.consume();
                subscriber.connect();
                subscriber.subscribe_topics(&[2]);
                subscriber.log_at(Level::Info, "Processing responses...");
                for msg in receiver.iter() {
                    if let Some(msg_value) = msg {
                        subscriber.log_at(Level::Info, format!("Received [Message: {}] [Topic: {}] [QoS: {}]", msg_value.payload_str(), msg_value.topic(), msg_value.qos()).as_str());
                        if is_final_message(msg_value.payload_str().as_ref(), config.clone()) {
                            // Due to the blocking nature of the `receiver.iter()` method, we need to break in order
                            // re-subscribe at the next qos/delay topic
                            subscriber.log_at(Level::Info, "Received final message, breaking from receiver");
                            break;
                        }
                    } else if !subscriber.client.is_connected() {
                        if subscriber.try_reconnect() {
                            subscriber.log_at(Level::Info, "Resubscribing to topics...");
                            subscriber.subscribe_topics(&[2]);
                        } else {
                            break;
                        }
                    }
                }
                subscriber.disconnect();
            }
            subscriber.disconnect();
        }
    })
}

///
/// Create a thread with a subscriber initialized within. This will publish messages to the broker
/// to indicate QoS and Delay changes.
///
/// # Arguments
/// * logger: Logger instance to log to
/// * config: Configuration to use to initialize the publisher
/// * tx: Sender channel instance to convey delay and qos level changes to publisher
///
/// # Returns
/// * `JoinHandle<()>` for joining thread as blocking
///
fn create_publisher_thread(logger: &Logger, config: Arc<Config>, tx: Sender<QOSDelayMessage>) -> JoinHandle<()> {
    thread::spawn({
        let t_logger: Logger = logger.clone();
        let t_tx: Sender<QOSDelayMessage> = tx.clone();
        move || {
            let mut publisher: Publisher = Publisher::new(config.clone(), t_logger.new(get_current_thread_id!()));
            publisher.initialize();
            publisher.connect();
            macro_rules! send_msg {
                ($msg:expr) => {
                    publisher.log_at(Level::Info, format!("Published [Message: {}] [Topic: {}] [QoS: {}]", $msg.payload_str(), $msg.topic(), $msg.qos()).as_str());
                    let tok: Result<(), mqtt::Error> = publisher.client.publish($msg);
                    if let Err(e) = tok {
                        publisher.log_at(Level::Error, format!("Error sending message: {:?}", e).as_str());
                        break;
                    }
                }
            }
            for qos in 0..=2 {
                for &delay in &[0,10,20,50,100,500] {
                    if delay == 0 {
                        // If this is the first delay level, we need to change QoS level, send a message to the broker indicating as such
                        try_except_with_log_action!(t_tx.send(QOSDelayMessage(qos, -1)), Level::Error, "Could not send message to subscriber thread", t_tx, publisher);
                        let msg: mqtt::Message = mqtt::Message::new(
                            "request/qos",
                            format!("{}", delay),
                            qos
                        );
                        send_msg!(msg);
                    }
                    // Send a message to the broker indicating a delay change
                    try_except_with_log_action!(t_tx.send(QOSDelayMessage(-1, delay)), Level::Error, "Could not send message to subscriber thread", t_tx, publisher);
                    thread::sleep(Duration::from_secs(2));
                    let msg: mqtt::Message = mqtt::Message::new(
                        "request/delay",
                        format!("{}", delay),
                        qos
                    );
                    send_msg!(msg);
                    thread::sleep(Duration::from_secs((40 * (qos + 1)) as u64));
                }
            }
            publisher.disconnect();
        }
    })
}

fn main() {
    let logger: Logger = initialize_logging(String::from("analyser_"));
    let config: Arc<Config> = Arc::new(Config::new("resource/analyser.properties", &logger.new(get_current_thread_id!())));
    let (tx, rx): (Sender<QOSDelayMessage>, Receiver<QOSDelayMessage>) = mpsc::channel();
    let mut threads: Vec<JoinHandle<()>> = Vec::with_capacity(2);

    threads.push(create_publisher_thread(&logger, config.clone(), tx));
    threads.push(create_subscriber_thread(&logger, config.clone(), rx));
    let thread_logger: Logger = logger.new(get_current_thread_id!());

    join_threads!(threads, thread_logger);
}