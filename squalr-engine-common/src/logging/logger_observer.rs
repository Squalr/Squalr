use crate::logging::log_level::LogLevel;

pub trait LoggerObserver {
    fn on_log_event(
        &self,
        log_level: LogLevel,
        message: &str,
        inner_message: Option<&str>,
    );
}

pub struct ConsoleLogger;

impl LoggerObserver for ConsoleLogger {
    fn on_log_event(
        &self,
        log_level: LogLevel,
        message: &str,
        inner_message: Option<&str>,
    ) {
        println!("{:?}: {} {:?}", log_level, message, inner_message);
    }
}
