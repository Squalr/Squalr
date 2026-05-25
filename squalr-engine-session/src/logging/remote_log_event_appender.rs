use crate::logging::log_record_filter::should_suppress_record;
use log::Record;
use log4rs::append::Append;
use squalr_engine_api::events::logging::log_recorded_event::LogRecordedEvent;
use std::{
    fmt,
    sync::{Arc, LazyLock, RwLock},
};

type RemoteLogEventSender = Arc<dyn Fn(LogRecordedEvent) + Send + Sync>;

static REMOTE_LOG_EVENT_SENDER: LazyLock<RwLock<Option<RemoteLogEventSender>>> = LazyLock::new(|| RwLock::new(None));

pub struct RemoteLogEventAppender;

impl RemoteLogEventAppender {
    pub fn new() -> Self {
        Self
    }

    pub fn set_sender(sender: Option<RemoteLogEventSender>) {
        if let Ok(mut remote_log_event_sender) = REMOTE_LOG_EVENT_SENDER.write() {
            *remote_log_event_sender = sender;
        }
    }
}

impl fmt::Debug for RemoteLogEventAppender {
    fn fmt(
        &self,
        formatter: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        formatter.debug_struct("RemoteLogEventAppender").finish()
    }
}

impl Append for RemoteLogEventAppender {
    fn append(
        &self,
        record: &Record,
    ) -> anyhow::Result<()> {
        if should_suppress_record(record) {
            return Ok(());
        }

        let sender = match REMOTE_LOG_EVENT_SENDER.read() {
            Ok(remote_log_event_sender) => remote_log_event_sender.clone(),
            Err(_) => None,
        };

        if let Some(sender) = sender {
            sender(LogRecordedEvent::new(record.level(), record.target().to_string(), record.args().to_string()));
        }

        Ok(())
    }

    fn flush(&self) {}
}
