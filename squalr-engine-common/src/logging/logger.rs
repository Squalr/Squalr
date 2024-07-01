use crate::logging::log_level::LogLevel;
use crate::logging::logger_observer::ILoggerObserver;
use crate::logging::observer_handle::ObserverHandle;

use std::collections::HashSet;
use std::sync::{Arc, Mutex, Once};

pub struct Logger {
    observers: Arc<Mutex<HashSet<ObserverHandle>>>,
}

impl Logger {
    // Create a new Logger
    pub fn new() -> Self {
        Logger {
            observers: Arc::new(Mutex::new(HashSet::new())),
        }
    }
    
    pub fn instance() -> &'static Logger {
        static mut SINGLETON: Option<Logger> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = Logger::new();
                SINGLETON = Some(instance);
            });

            SINGLETON.as_ref().unwrap()
        }
    }

    // Subscribe an observer
    pub fn subscribe(&self, observer: Arc<dyn ILoggerObserver + Send + Sync>) {
        self.observers.lock().unwrap().insert(ObserverHandle::new(observer));
    }

    // Unsubscribe an observer
    pub fn unsubscribe(&self, observer: &Arc<dyn ILoggerObserver + Send + Sync>) {
        self.observers.lock().unwrap().retain(|o| !Arc::ptr_eq(o.get(), observer));
    }

    // Log a message with an optional inner message
    pub fn log(&self, log_level: LogLevel, message: &str, inner_message: Option<&str>) {
        let observers = self.observers.lock().unwrap();
        for observer in observers.iter() {
            observer.get().on_log_event(log_level, message, inner_message);
        }
    }

    // Convenience method to log a message with an exception
    pub fn log_with_exception(&self, log_level: LogLevel, message: &str, exception: Option<&dyn std::error::Error>) {
        let inner_message = exception.map(|e| e.to_string());
        self.log(log_level, message, inner_message.as_deref());
    }
}
