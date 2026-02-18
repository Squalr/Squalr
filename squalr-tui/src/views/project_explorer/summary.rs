use crate::views::project_explorer::pane_state::{ProjectExplorerFocusTarget, ProjectExplorerPaneState};

pub fn build_project_explorer_summary_lines(project_explorer_pane_state: &ProjectExplorerPaneState) -> Vec<String> {
    let mode_label = if project_explorer_pane_state
        .active_project_directory_path
        .is_some()
    {
        "Hierarchy"
    } else {
        "Projects"
    };

    let mut summary_lines = vec![format!("[MODE] {}.", mode_label)];

    if project_explorer_pane_state.focus_target == ProjectExplorerFocusTarget::ProjectList {
        summary_lines.push("[ACT] / search | Up/Down move | Home/End jump | n create | Enter/o open | e rename | c close | x delete | r refresh.".to_string());
    } else {
        summary_lines.push("[TREE] Up/Down move | Home/End jump | l/Right expand | h/Left collapse | Space activate.".to_string());
        summary_lines.push("[MOVE] m stage | b move | [/] reorder | u clear-stage.".to_string());
        summary_lines.push(format!(
            "[PROJ] selected={} | active={} | dir={}.",
            option_to_compact_text(project_explorer_pane_state.selected_project_name.as_deref()),
            option_to_compact_text(project_explorer_pane_state.active_project_name.as_deref()),
            option_path_to_compact_text(
                project_explorer_pane_state
                    .active_project_directory_path
                    .as_deref()
            )
        ));
    }

    summary_lines.extend([
        format!(
            "[ITEM] selected={} | pending_name={}.",
            option_to_compact_text(project_explorer_pane_state.selected_item_path.as_deref()),
            project_explorer_pane_state.pending_project_name_input
        ),
        format!(
            "[PEND] move_count={} | delete_count={} | loading_items={}.",
            project_explorer_pane_state.pending_move_source_paths.len(),
            project_explorer_pane_state
                .pending_delete_confirmation_paths
                .len(),
            project_explorer_pane_state.is_awaiting_project_item_list_response
        ),
        format!(
            "[COUNT] project_count={} | visible_item_count={}.",
            project_explorer_pane_state.project_entries.len(),
            project_explorer_pane_state.project_item_visible_entries.len()
        ),
        format!("[STAT] {}.", project_explorer_pane_state.status_message),
    ]);

    summary_lines
}

fn option_to_compact_text(option_text: Option<&str>) -> String {
    option_text
        .map(|text| format!("\"{}\"", text))
        .unwrap_or_else(|| "none".to_string())
}

fn option_path_to_compact_text(option_path: Option<&std::path::Path>) -> String {
    option_path
        .map(|path| format!("\"{}\"", path.display()))
        .unwrap_or_else(|| "none".to_string())
}
