use crossbeam_channel::Sender;
use log::Record;
use log4rs::append::Append;

#[derive(Debug)]
pub struct OutputLogCollector {
    log_sender: Sender<String>,
}

impl OutputLogCollector {
    pub fn new(log_sender: Sender<String>) -> Self {
        OutputLogCollector { log_sender }
    }
}

impl Append for OutputLogCollector {
    fn append(
        &self,
        record: &Record,
    ) -> anyhow::Result<()> {
        let log_message = format!("[{}] {}\n", record.level(), record.args());

        // Just silently fail -- logging errors inside a failing logging framework seems disasterous.
        let _ = self.log_sender.send(log_message);

        return Ok(());
    }

    fn flush(&self) {}
}
