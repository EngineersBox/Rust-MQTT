use slog::Level;

pub trait Connector {
    fn initialize(&mut self);
    fn connect(&mut self);
    fn disconnect(&mut self);
    fn log_at(&self, level: Level, msg: &str);
}