pub trait Connector {
    fn initialize(&mut self);
    fn connect(&mut self);
    fn disconnect(&mut self);
}