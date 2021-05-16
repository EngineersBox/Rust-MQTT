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
    let config: Arc<Config> = Arc::new(Config::new("resource/config.properties", &logger));
    let subscriber: Subscriber = Subscriber::new(config.clone(), &logger);
}