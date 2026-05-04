use crate::state::pane_entry_row::PaneEntryRow;
use crate::views::entry_row_viewport::build_selection_relative_viewport_range;
use crate::views::project_explorer::pane_state::{ProjectExplorerFocusTarget, ProjectExplorerPaneState, ProjectSelectorInputMode};

pub fn build_visible_project_entry_rows(
    project_explorer_pane_state: &ProjectExplorerPaneState,
    viewport_capacity: usize,
) -> Vec<PaneEntryRow> {
    if viewport_capacity == 0 {
        return Vec::new();
    }

    let is_search_input_active = project_explorer_pane_state.input_mode == ProjectSelectorInputMode::Search;
    let search_marker_text = if is_search_input_active { "/".to_string() } else { String::new() };
    let mut entry_rows = vec![if is_search_input_active {
        PaneEntryRow::selected(
            search_marker_text.to_string(),
            format!("search: {}", project_explorer_pane_state.pending_search_name_input),
            None,
        )
    } else {
        PaneEntryRow::normal(
            search_marker_text.to_string(),
            format!("search: {}", project_explorer_pane_state.pending_search_name_input),
            None,
        )
    }];

    let project_row_capacity = viewport_capacity.saturating_sub(1);
    if project_row_capacity == 0 {
        return entry_rows;
    }

    let visible_project_range = build_selection_relative_viewport_range(
        project_explorer_pane_state.project_entries.len(),
        project_explorer_pane_state.selected_project_list_index,
        project_row_capacity,
    );
    entry_rows.reserve(visible_project_range.len());

    for visible_project_position in visible_project_range {
        if let Some(project_entry) = project_explorer_pane_state
            .project_entries
            .get(visible_project_position)
        {
            let is_selected_project = project_explorer_pane_state.selected_project_list_index == Some(visible_project_position);
            let is_active_project = project_explorer_pane_state
                .active_project_directory_path
                .as_ref()
                .zip(project_entry.get_project_directory())
                .is_some_and(|(active_project_directory, project_entry_directory)| *active_project_directory == project_entry_directory);
            let project_directory_display = project_entry
                .get_project_directory()
                .map(|project_directory| project_directory.display().to_string())
                .unwrap_or_else(|| "<unknown>".to_string());
            let marker_text = if is_active_project { "*".to_string() } else { String::new() };
            let primary_text = project_entry.get_name().to_string();
            let secondary_text = Some(project_directory_display);

            if project_explorer_pane_state.focus_target != ProjectExplorerFocusTarget::ProjectList {
                entry_rows.push(PaneEntryRow::disabled(marker_text, primary_text, secondary_text));
            } else if is_selected_project {
                entry_rows.push(PaneEntryRow::selected(marker_text, primary_text, secondary_text));
            } else {
                entry_rows.push(PaneEntryRow::normal(marker_text, primary_text, secondary_text));
            }
        }
    }

    entry_rows
}

pub fn build_visible_project_item_entry_rows(
    project_explorer_pane_state: &ProjectExplorerPaneState,
    viewport_capacity: usize,
) -> Vec<PaneEntryRow> {
    let visible_project_item_range = build_selection_relative_viewport_range(
        project_explorer_pane_state.project_item_visible_entries.len(),
        project_explorer_pane_state.selected_project_item_visible_index,
        viewport_capacity,
    );
    let mut entry_rows = Vec::with_capacity(visible_project_item_range.len());

    for visible_project_item_position in visible_project_item_range {
        if let Some(project_item_entry) = project_explorer_pane_state
            .project_item_visible_entries
            .get(visible_project_item_position)
        {
            let is_selected_project_item = project_explorer_pane_state.selected_project_item_visible_index == Some(visible_project_item_position);
            let activation_marker = if project_item_entry.is_activated { "[x]" } else { "[ ]" };
            let directory_marker = if project_item_entry.is_directory {
                if project_item_entry.is_expanded { "-" } else { "+" }
            } else {
                " "
            };
            let indentation = " ".repeat(project_item_entry.depth.saturating_mul(2));
            let marker_text = directory_marker.to_string();
            let primary_text = format!("{}{} {}", indentation, activation_marker, project_item_entry.display_name);
            let secondary_text = if project_item_entry.preview_value.is_empty() {
                None
            } else {
                Some(format!("value={}", project_item_entry.preview_value))
            };

            if project_explorer_pane_state.focus_target != ProjectExplorerFocusTarget::ProjectHierarchy {
                entry_rows.push(PaneEntryRow::disabled(marker_text, primary_text, secondary_text));
            } else if is_selected_project_item {
                entry_rows.push(PaneEntryRow::selected(marker_text, primary_text, secondary_text));
            } else {
                entry_rows.push(PaneEntryRow::normal(marker_text, primary_text, secondary_text));
            }
        }
    }

    entry_rows
}

pub fn build_visible_project_symbol_entry_rows(
    project_explorer_pane_state: &ProjectExplorerPaneState,
    viewport_capacity: usize,
) -> Vec<PaneEntryRow> {
    let visible_symbol_claim_range = build_selection_relative_viewport_range(
        project_explorer_pane_state.symbol_claims.len(),
        project_explorer_pane_state.selected_symbol_claim_index,
        viewport_capacity,
    );
    let mut entry_rows = Vec::with_capacity(visible_symbol_claim_range.len());

    for visible_symbol_claim_position in visible_symbol_claim_range {
        if let Some(symbol_claim) = project_explorer_pane_state
            .symbol_claims
            .get(visible_symbol_claim_position)
        {
            let is_selected_symbol_claim = project_explorer_pane_state.selected_symbol_claim_index == Some(visible_symbol_claim_position);
            let marker_text = "@".to_string();
            let primary_text = symbol_claim.get_display_name().to_string();
            let secondary_text = Some(format!(
                "{} | {} | {}",
                symbol_claim.get_struct_layout_id(),
                symbol_claim.get_locator(),
                symbol_claim.get_symbol_locator_key()
            ));

            if project_explorer_pane_state.focus_target != ProjectExplorerFocusTarget::ProjectSymbols {
                entry_rows.push(PaneEntryRow::disabled(marker_text, primary_text, secondary_text));
            } else if is_selected_symbol_claim {
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
    use super::{build_visible_project_item_entry_rows, build_visible_project_symbol_entry_rows};
    use crate::views::project_explorer::pane_state::{ProjectExplorerFocusTarget, ProjectExplorerPaneState, ProjectHierarchyEntry};
    use squalr_engine_api::structures::projects::project_symbol_claim::ProjectSymbolClaim;
    use std::path::PathBuf;

    #[test]
    fn project_item_rows_show_preview_value_instead_of_absolute_path() {
        let mut project_explorer_pane_state = ProjectExplorerPaneState::default();
        project_explorer_pane_state.focus_target = ProjectExplorerFocusTarget::ProjectHierarchy;
        project_explorer_pane_state.selected_project_item_visible_index = Some(0);
        project_explorer_pane_state.project_item_visible_entries = vec![ProjectHierarchyEntry {
            project_item_path: PathBuf::from("C:/projects/opened/TestProject/project_items/Addresses/Health.json"),
            display_name: "Health".to_string(),
            preview_value: "255".to_string(),
            depth: 0,
            is_directory: false,
            is_expanded: false,
            is_activated: false,
        }];

        let visible_entry_rows = build_visible_project_item_entry_rows(&project_explorer_pane_state, 4);

        assert_eq!(visible_entry_rows.len(), 1);
        assert_eq!(visible_entry_rows[0].secondary_text.as_deref(), Some("value=255"));
    }

    #[test]
    fn symbol_claim_rows_show_type_locator_and_locator_key() {
        let mut project_explorer_pane_state = ProjectExplorerPaneState::default();
        project_explorer_pane_state.focus_target = ProjectExplorerFocusTarget::ProjectSymbols;
        project_explorer_pane_state.selected_symbol_claim_index = Some(0);
        project_explorer_pane_state.symbol_claims = vec![ProjectSymbolClaim::new_absolute_address(
            String::from("Player"),
            0x1234,
            String::from("player"),
        )];

        let visible_entry_rows = build_visible_project_symbol_entry_rows(&project_explorer_pane_state, 4);

        assert_eq!(visible_entry_rows.len(), 1);
        assert_eq!(visible_entry_rows[0].marker_text, "@");
        assert_eq!(visible_entry_rows[0].primary_text, "Player");
        assert_eq!(visible_entry_rows[0].secondary_text.as_deref(), Some("player | 0x1234 | absolute:1234"));
    }
}
