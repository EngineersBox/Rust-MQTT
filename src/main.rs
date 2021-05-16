mod config;
mod macros;
mod logging;
mod connector;

#[macro_use]
extern crate slog;
extern crate paho_mqtt as mqtt;
extern crate slog_term;
extern crate slog_async;
extern crate slog_json;
extern crate regex;

fn main() {
    println!("Hello, world!");
}
