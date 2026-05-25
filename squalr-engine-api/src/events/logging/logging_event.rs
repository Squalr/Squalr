use crate::events::logging::log_recorded_event::LogRecordedEvent;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LoggingEvent {
    LogRecorded { log_recorded_event: LogRecordedEvent },
}
