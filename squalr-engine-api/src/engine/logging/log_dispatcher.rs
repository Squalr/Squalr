use crate::structures::logging::log_event::LogEvent;
use crossbeam_channel::Sender;
use log::Record;
use log4rs::append::Append;

#[derive(Debug)]
pub struct LogDispatcher {
    log_sender: Sender<LogEvent>,
}

impl LogDispatcher {
    pub fn new(log_sender: Sender<LogEvent>) -> Self {
        LogDispatcher { log_sender }
    }
}

impl Append for LogDispatcher {
    fn append(
        &self,
        record: &Record,
    ) -> anyhow::Result<()> {
        let level = record.level();
        let message = format!("{}", record.args());

        // Just silently fail -- logging more errors inside a failing logging framework risks infinite loops.
        let _ = self.log_sender.send(LogEvent { message, level });

        return Ok(());
    }

    fn flush(&self) {}
}
