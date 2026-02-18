use crate::state::pane_entry_row::PaneEntryRow;
use crate::views::entry_row_viewport::build_selection_relative_viewport_range;
use crate::views::process_selector::pane_state::ProcessSelectorPaneState;

pub fn build_visible_process_entry_rows(
    process_selector_pane_state: &ProcessSelectorPaneState,
    viewport_capacity: usize,
) -> Vec<PaneEntryRow> {
    if viewport_capacity == 0 {
        return Vec::new();
    }

    let is_search_input_active = process_selector_pane_state.input_mode == crate::views::process_selector::pane_state::ProcessSelectorInputMode::Search;
    let search_marker_text = if is_search_input_active { "/".to_string() } else { String::new() };
    let mut entry_rows = vec![if is_search_input_active {
        PaneEntryRow::selected(
            search_marker_text.to_string(),
            format!("search: {}", process_selector_pane_state.pending_search_name_input),
            None,
        )
    } else {
        PaneEntryRow::normal(
            search_marker_text.to_string(),
            format!("search: {}", process_selector_pane_state.pending_search_name_input),
            None,
        )
    }];

    let process_row_capacity = viewport_capacity.saturating_sub(1);
    if process_row_capacity == 0 {
        return entry_rows;
    }

    let visible_process_range = build_selection_relative_viewport_range(
        process_selector_pane_state.process_list_entries.len(),
        process_selector_pane_state.selected_process_list_index,
        process_row_capacity,
    );
    entry_rows.reserve(visible_process_range.len());

    for visible_process_position in visible_process_range {
        if let Some(process_entry) = process_selector_pane_state
            .process_list_entries
            .get(visible_process_position)
        {
            let is_selected_process = process_selector_pane_state.selected_process_list_index == Some(visible_process_position);
            let should_highlight_selected_process = !is_search_input_active;
            let is_opened_process = process_selector_pane_state.opened_process_identifier == Some(process_entry.get_process_id_raw());
            let marker_text = match (is_selected_process && should_highlight_selected_process, is_opened_process) {
                (true, true) => ">*".to_string(),
                (true, false) => ">".to_string(),
                (false, true) => "*".to_string(),
                (false, false) => String::new(),
            };
            let primary_text = process_entry.get_name().to_string();
            let secondary_text = Some(format!("pid={}", process_entry.get_process_id_raw()));

            if is_selected_process && should_highlight_selected_process {
                entry_rows.push(PaneEntryRow::selected(marker_text, primary_text, secondary_text));
            } else {
                entry_rows.push(PaneEntryRow::normal(marker_text, primary_text, secondary_text));
            }
        }
    }

    entry_rows
}

#[cfg(test)]
mod tests {
    use super::build_visible_process_entry_rows;
    use crate::state::pane_entry_row::PaneEntryRowTone;
    use crate::views::process_selector::pane_state::{ProcessSelectorInputMode, ProcessSelectorPaneState};
    use squalr_engine_api::structures::processes::process_info::ProcessInfo;

    fn create_process_entry(
        process_name: &str,
        process_identifier: u32,
    ) -> ProcessInfo {
        ProcessInfo::new(process_identifier, process_name.to_string(), true, None)
    }

    #[test]
    fn search_input_mode_highlights_only_search_row() {
        let mut process_selector_pane_state = ProcessSelectorPaneState::default();
        process_selector_pane_state.apply_process_list(vec![
            create_process_entry("Alpha", 100),
            create_process_entry("Beta", 200),
        ]);
        process_selector_pane_state.selected_process_list_index = Some(0);
        process_selector_pane_state.input_mode = ProcessSelectorInputMode::Search;
        process_selector_pane_state.pending_search_name_input = "Al".to_string();

        let visible_entry_rows = build_visible_process_entry_rows(&process_selector_pane_state, 5);

        assert_eq!(visible_entry_rows[0].tone, PaneEntryRowTone::Selected);
        assert_eq!(visible_entry_rows[0].marker_text, "/");
        assert_eq!(visible_entry_rows[1].tone, PaneEntryRowTone::Normal);
        assert_ne!(visible_entry_rows[1].marker_text, ">");
        assert_ne!(visible_entry_rows[1].marker_text, ">*");
    }
}
