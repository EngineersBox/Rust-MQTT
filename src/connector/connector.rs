use slog::Level;

pub trait Connector {
    ///
    /// Creates a connection based on the options given in a configuration. This will configure
    /// aspects such as:
    /// * username
    /// * password
    /// * host
    /// * port
    /// * Keep alive
    /// * will send
    /// * etc
    ///
    fn initialize(&mut self);
    ///
    /// Invoke the connector to do an initial CONNECT handshake between the client and broker
    ///
    fn connect(&mut self);
    ///
    /// Invoke the connector to do a final DISCONNECT handshake between the client and broker.
    /// This will send the WILL_SEND predefined message to the broker prior to termination
    fn disconnect(&mut self);
    ///
    /// Create a log entry for a given level
    ///
    /// # Arguments:
    /// * level: An logging level of INFO, DEBUG, ERROR, CRITICAL or WARN
    /// * msg: Message to log
    ///
    fn log_at(&self, level: Level, msg: &str);
}