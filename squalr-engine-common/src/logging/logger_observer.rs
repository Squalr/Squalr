use crate::logging::log_level::LogLevel;

pub trait ILoggerObserver {
    fn on_log_event(
        &self,
        log_level: LogLevel,
        message: &str,
        inner_message: Option<&str>,
    );
}

pub struct ConsoleLogger;

impl ILoggerObserver for ConsoleLogger {
    fn on_log_event(
        &self,
        log_level: LogLevel,
        message: &str,
        inner_message: Option<&str>,
    ) {
        println!("{:?}: {} {:?}", log_level, message, inner_message);
    }
}
