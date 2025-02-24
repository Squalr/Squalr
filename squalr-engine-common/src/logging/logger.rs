use crate::logging::log_level::LogLevel;
use crate::logging::logger_observer::LoggerObserver;
use crate::logging::observer_handle::ObserverHandle;
use std::collections::HashSet;
use std::sync::{Arc, Mutex, Once};

pub struct Logger {
    observers: Arc<Mutex<HashSet<ObserverHandle>>>,
}

impl Logger {
    fn new() -> Self {
        Logger {
            observers: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    fn get_instance() -> &'static Logger {
        static mut INSTANCE: Option<Logger> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = Logger::new();
                INSTANCE = Some(instance);
            });

            #[allow(static_mut_refs)]
            INSTANCE.as_ref().unwrap_unchecked()
        }
    }

    pub fn subscribe(observer: Arc<dyn LoggerObserver>) {
        Logger::get_instance()
            .observers
            .lock()
            .unwrap()
            .insert(ObserverHandle::new(observer.clone()));
    }

    pub fn unsubscribe(observer: &Arc<dyn LoggerObserver>) {
        Logger::get_instance()
            .observers
            .lock()
            .unwrap()
            .retain(|o| !Arc::ptr_eq(o.get(), observer));
    }

    // Log a message with an optional inner message
    pub fn log(
        log_level: LogLevel,
        message: &str,
        inner_message: Option<&str>,
    ) {
        let observers = Logger::get_instance().observers.lock().unwrap();
        for observer in observers.iter() {
            Logger::log_to_observer(observer.get(), log_level, message, inner_message);
        }
    }

    // Convenience method to log a message with an exception
    pub fn log_with_exception(
        log_level: LogLevel,
        message: &str,
        exception: Option<&dyn std::error::Error>,
    ) {
        let inner_message = exception.map(|e| e.to_string());
        Logger::log(log_level, message, inner_message.as_deref());
    }

    fn log_to_observer(
        observer: &Arc<dyn LoggerObserver>,
        log_level: LogLevel,
        message: &str,
        inner_message: Option<&str>,
    ) {
        observer.on_log_event(log_level, message, inner_message);
    }
}
