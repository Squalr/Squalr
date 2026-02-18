use crate::views::output::pane_state::OutputPaneState;

pub const OUTPUT_FIXED_SUMMARY_LINE_COUNT: usize = 3;

pub fn build_output_summary_lines(
    output_pane_state: &OutputPaneState,
    log_preview_capacity: usize,
) -> Vec<String> {
    let mut summary_lines = vec![
        "[ACT] r refresh-log | x clear | +/- max-lines.".to_string(),
        format!(
            "[LOG] lines={} | max_lines={} | auto_scroll={}.",
            output_pane_state.log_lines.len(),
            output_pane_state.max_log_line_count,
            output_pane_state.did_auto_scroll_to_latest
        ),
        format!("[STAT] {}.", output_pane_state.status_message),
    ];

    if log_preview_capacity > 0 && !output_pane_state.log_lines.is_empty() {
        let preview_line_count = output_pane_state
            .log_lines
            .len()
            .min(log_preview_capacity.saturating_sub(1));
        summary_lines.push("[RECENT]".to_string());
        let start_line_index = output_pane_state
            .log_lines
            .len()
            .saturating_sub(preview_line_count);
        for preview_line in &output_pane_state.log_lines[start_line_index..] {
            summary_lines.push(format!("[LOG] {}", preview_line));
        }
    }

    summary_lines
}
