pub trait Connector {
    fn initialize(&self);
    fn connect(&self);
    fn disconnect(&self);
}