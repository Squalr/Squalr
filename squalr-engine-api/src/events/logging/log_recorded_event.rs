use crate::events::{
    engine_event::{EngineEvent, EngineEventRequest},
    logging::logging_event::LoggingEvent,
};
use log::Level;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogRecordedEvent {
    pub level: Level,
    pub target: String,
    pub message: String,
}

impl LogRecordedEvent {
    pub fn new(
        level: Level,
        target: String,
        message: String,
    ) -> Self {
        Self { level, target, message }
    }
}

impl EngineEventRequest for LogRecordedEvent {
    fn to_engine_event(&self) -> EngineEvent {
        EngineEvent::Logging(LoggingEvent::LogRecorded {
            log_recorded_event: self.clone(),
        })
    }
}
