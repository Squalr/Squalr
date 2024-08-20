use squalr_engine_common::logging::log_level::LogLevel;
use squalr_engine_common::logging::logger_observer::ILoggerObserver;
use std::sync::Arc;

pub struct LogListener;

impl ILoggerObserver for LogListener {
    fn on_log_event(
        &self,
        log_level: LogLevel,
        message: &str,
        inner_message: Option<&str>
    ) {
        match inner_message {
            Some(inner) => println!("[{:?}] {} - {}", log_level, message, inner),
            None => println!("[{:?}] {}", log_level, message),
        }
    }
}

impl LogListener {
    pub fn new(
    ) -> Arc<Self> {
        Arc::new(Self)
    }
}
