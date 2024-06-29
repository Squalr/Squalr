#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}
