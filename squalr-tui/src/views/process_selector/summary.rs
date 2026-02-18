use crate::views::process_selector::pane_state::ProcessSelectorPaneState;

pub fn build_process_selector_summary_lines(process_selector_pane_state: &ProcessSelectorPaneState) -> Vec<String> {
    vec![
        "[ACT] Enter/o open | / search | Up/Down move | Home/End jump | r refresh | w windowed/full | F1 Project | F4 Process.".to_string(),
        format!(
            "[LIST] shown={} | total={} | windowed_only={} | loading={}.",
            process_selector_pane_state.process_list_entries.len(),
            process_selector_pane_state.all_process_entries.len(),
            process_selector_pane_state.show_windowed_processes_only,
            process_selector_pane_state.is_awaiting_process_list_response
        ),
        format!(
            "[SEL] id={} | name={}.",
            option_to_compact_text(process_selector_pane_state.selected_process_identifier),
            option_string_to_compact_text(process_selector_pane_state.selected_process_name.as_deref())
        ),
        format!(
            "[OPEN] id={} | opening={}.",
            option_to_compact_text(process_selector_pane_state.opened_process_identifier),
            process_selector_pane_state.is_opening_process
        ),
        format!("[STAT] {}.", process_selector_pane_state.status_message),
        "[ROWS] top=5.".to_string(),
    ]
}

fn option_to_compact_text<T: std::fmt::Display>(option_value: Option<T>) -> String {
    option_value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "none".to_string())
}

fn option_string_to_compact_text(option_text: Option<&str>) -> String {
    option_text
        .map(|text| format!("\"{}\"", text))
        .unwrap_or_else(|| "none".to_string())
}
