use std::io;

pub trait RuntimeMode {
    fn run(&mut self) -> io::Result<()>;
    fn shutdown(&mut self);
}
