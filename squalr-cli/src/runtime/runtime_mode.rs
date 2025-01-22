pub trait RuntimeMode {
    fn run_loop(&mut self);
    fn shutdown(&mut self);
}
