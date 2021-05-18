mod config;
mod macros;
mod logging;
mod connector;

use logging::logging::initialize_logging;
use config::config::Config;
use connector::subscriber::subscriber::Subscriber;

#[macro_use]
extern crate slog;
extern crate paho_mqtt as mqtt;
extern crate slog_term;
extern crate slog_async;
extern crate slog_json;
extern crate regex;

use slog::Logger;
use std::sync::Arc;

fn main() {
    let logger: Logger = initialize_logging();
    let config: Arc<Config> = Arc::new(Config::new("resource/analyser.properties", &logger.new(get_current_thread_id!())));
    // (0..5).collect::<Vec<i32>>().iter().for_each(|num: &i32| {
    //     let content: String =  "Hello world! ".to_string() + &num.to_string();
    //     publisher.log_at(Level::Debug, format!("Message: {:?}", content.clone()).as_str());
    //     let msg: mqtt::Message = mqtt::Message::new("request/qos", content.clone(), 1);
    //     publisher.log_at(Level::Debug, "Publishing messages on the request/qos topic");
    //     let tok: Result<(), mqtt::Error> = publisher.client.publish(msg);
    //
    //     if let Err(e) = tok {
    //         publisher.log_at(Level::Error, format!("Error sending message: {:?}", e).as_str());
    //         return;
    //     }
    // });
}