use crate::views::output::summary::build_output_summary_lines;
use squalr_engine_api::structures::logging::log_event::LogEvent;

/// Stores recent log lines and output-pane configuration.
#[derive(Clone, Debug)]
pub struct OutputPaneState {
    pub log_lines: Vec<String>,
    pub max_log_line_count: usize,
    pub did_auto_scroll_to_latest: bool,
    pub status_message: String,
}

impl OutputPaneState {
    pub fn apply_log_history_with_feedback(
        &mut self,
        log_history: Vec<LogEvent>,
        should_update_status_message: bool,
    ) {
        self.log_lines = log_history
            .into_iter()
            .map(|log_event| format!("[{}] {}", log_event.level, log_event.message))
            .collect();
        self.trim_to_max_line_count();
        self.did_auto_scroll_to_latest = true;
        if should_update_status_message {
            self.status_message = format!("Loaded {} log lines.", self.log_lines.len());
        }
    }

    pub fn clear_log_lines(&mut self) {
        self.log_lines.clear();
        self.did_auto_scroll_to_latest = true;
        self.status_message = "Cleared output pane log lines.".to_string();
    }

    pub fn increase_max_line_count(&mut self) {
        self.max_log_line_count = (self.max_log_line_count + 25).min(2_000);
        self.trim_to_max_line_count();
    }

    pub fn decrease_max_line_count(&mut self) {
        self.max_log_line_count = self.max_log_line_count.saturating_sub(25).max(25);
        self.trim_to_max_line_count();
    }

    pub fn summary_lines(
        &self,
        log_preview_capacity: usize,
    ) -> Vec<String> {
        build_output_summary_lines(self, log_preview_capacity)
    }

    fn trim_to_max_line_count(&mut self) {
        if self.log_lines.len() <= self.max_log_line_count {
            return;
        }

        let remove_line_count = self.log_lines.len() - self.max_log_line_count;
        self.log_lines.drain(0..remove_line_count);
    }
}

impl Default for OutputPaneState {
    fn default() -> Self {
        Self {
            log_lines: Vec::new(),
            max_log_line_count: 200,
            did_auto_scroll_to_latest: false,
            status_message: "Ready.".to_string(),
        }
    }
}
