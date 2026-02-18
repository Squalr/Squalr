use crate::views::scan_results::pane_state::ScanResultsPaneState;

pub fn build_scan_results_summary_lines(scan_results_pane_state: &ScanResultsPaneState) -> Vec<String> {
    let selected_type_filters = if scan_results_pane_state.filtered_data_type_ids.is_empty() {
        "all".to_string()
    } else {
        scan_results_pane_state
            .filtered_data_type_ids
            .iter()
            .cloned()
            .collect::<Vec<_>>()
            .join(",")
    };
    let available_types = if scan_results_pane_state.available_data_type_ids.is_empty() {
        "none".to_string()
    } else {
        scan_results_pane_state.available_data_type_ids.join(",")
    };
    let mut summary_lines = vec![
        "[ACT] r query | R refresh-page | [/] page | f freeze | a add | x delete.".to_string(),
        "[NAV] Up/Down move | Shift+Up/Down range | Home/End.".to_string(),
        "[EDIT] y pull | type value | Enter commit.".to_string(),
        format!("[TYPE] active={} | available={}.", selected_type_filters, available_types),
        format!(
            "[PAGE] {}/{} | size={} | results={}.",
            display_page_number(scan_results_pane_state.current_page_index),
            display_page_number(scan_results_pane_state.cached_last_page_index),
            scan_results_pane_state.results_per_page,
            scan_results_pane_state.total_result_count
        ),
        format!(
            "[SEL] index={} | selected={} | bytes={}.",
            option_to_compact_text(
                scan_results_pane_state
                    .selected_result_index
                    .map(|selected_result_index| selected_result_index + 1)
            ),
            scan_results_pane_state.selected_result_count(),
            scan_results_pane_state.total_size_in_bytes
        ),
        format!(
            "[LAST] result_count={} | total_bytes={}.",
            scan_results_pane_state.total_result_count, scan_results_pane_state.total_size_in_bytes
        ),
        format!("[STAT] {}.", scan_results_pane_state.status_message),
    ];

    if !scan_results_pane_state.pending_value_edit_text.is_empty() {
        let edit_value_line = format!("[EDIT VAL] {}.", scan_results_pane_state.pending_value_edit_text);
        let status_line_index = summary_lines
            .iter()
            .position(|summary_line| summary_line.starts_with("[STAT]"))
            .unwrap_or(summary_lines.len());
        summary_lines.insert(status_line_index, edit_value_line);
    }

    summary_lines
}

fn option_to_compact_text<T: std::fmt::Display>(option_value: Option<T>) -> String {
    option_value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn display_page_number(page_index: u64) -> u64 {
    page_index.saturating_add(1)
}
