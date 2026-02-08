use crate::ui_state::InstallerUiState;
use eframe::egui::Context;
use log::{Level, Log, Metadata, Record, SetLoggerError};
use std::sync::{Arc, Mutex};

pub(crate) const MAX_LOG_BUFFER_BYTES: usize = 256 * 1024;

struct InstallerLogger {
    ui_state: Arc<Mutex<InstallerUiState>>,
    repaint_context: Context,
}

impl InstallerLogger {
    fn new(
        ui_state: Arc<Mutex<InstallerUiState>>,
        repaint_context: Context,
    ) -> Self {
        Self { ui_state, repaint_context }
    }
}

impl Log for InstallerLogger {
    fn enabled(
        &self,
        metadata: &Metadata,
    ) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(
        &self,
        record: &Record,
    ) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let log_message = format!("[{}] {}\n", record.level(), record.args());

        if let Ok(mut ui_state) = self.ui_state.lock() {
            ui_state.append_log(&log_message);
            self.repaint_context.request_repaint();
        }
    }

    fn flush(&self) {}
}

pub(crate) fn trim_log_buffer(
    log_buffer: &mut String,
    max_buffer_bytes: usize,
) {
    if log_buffer.len() <= max_buffer_bytes {
        return;
    }

    let mut trim_start_index = log_buffer.len().saturating_sub(max_buffer_bytes);
    while trim_start_index < log_buffer.len() && !log_buffer.is_char_boundary(trim_start_index) {
        trim_start_index += 1;
    }

    log_buffer.drain(..trim_start_index);
}

pub(crate) fn initialize_logger(
    ui_state: Arc<Mutex<InstallerUiState>>,
    repaint_context: Context,
) -> Result<(), SetLoggerError> {
    let logger = InstallerLogger::new(ui_state, repaint_context);
    log::set_boxed_logger(Box::new(logger))?;
    log::set_max_level(log::LevelFilter::Info);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::trim_log_buffer;

    #[test]
    fn trim_log_buffer_keeps_recent_log_data() {
        let mut log_buffer = "line-1\nline-2\nline-3\n".to_string();
        trim_log_buffer(&mut log_buffer, 8);
        assert_eq!(log_buffer, "\nline-3\n");
    }

    #[test]
    fn trim_log_buffer_does_not_modify_when_under_limit() {
        let mut log_buffer = "line-1\n".to_string();
        trim_log_buffer(&mut log_buffer, 1024);
        assert_eq!(log_buffer, "line-1\n");
    }
}
