use log::Record;
use log4rs::append::Append;
use squalr_engine_api::structures::logging::log_event::LogEvent;
use std::{
    collections::VecDeque,
    fmt,
    sync::{Arc, RwLock},
};

pub struct LogHistoryAppender {
    max_retain_size: usize,
    log_history: Arc<RwLock<VecDeque<LogEvent>>>,
}

impl LogHistoryAppender {
    pub fn new(log_history: Arc<RwLock<VecDeque<LogEvent>>>) -> Self {
        Self {
            max_retain_size: 4096,
            log_history,
        }
    }
}

impl fmt::Debug for LogHistoryAppender {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self.log_history.read() {
            Ok(history) => formatter
                .debug_struct("LogHistoryAppender")
                .field("log_history_len", &history.len())
                .finish(),
            Err(_) => formatter
                .debug_struct("LogHistoryAppender")
                .field("log_history", &"<poisoned>")
                .finish(),
        }
    }
}

impl Append for LogHistoryAppender {
    fn append(
        &self,
        record: &Record,
    ) -> anyhow::Result<()> {
        match self.log_history.write() {
            Ok(mut log_history) => {
                let level = record.level();
                let message = format!("{}", record.args());
                let event = LogEvent { message, level };

                while log_history.len() >= self.max_retain_size {
                    log_history.pop_front();
                }

                log_history.push_back(event);
            }
            Err(_error) => {
                // Just silently fail -- logging more errors inside a failing logging framework would risk infinite loops.
            }
        }

        Ok(())
    }

    fn flush(&self) {}
}
