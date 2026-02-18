use crate::views::struct_viewer::pane_state::StructViewerPaneState;

pub const STRUCT_VIEWER_FIXED_SUMMARY_LINE_COUNT: usize = 10;

pub fn build_struct_viewer_summary_lines(
    struct_viewer_pane_state: &StructViewerPaneState,
    focused_field_preview_capacity: usize,
) -> Vec<String> {
    let selected_field_display_format = struct_viewer_pane_state.selected_field_active_display_format();
    let selected_field_display_format_progress = struct_viewer_pane_state.selected_field_display_format_progress();
    let selected_field_edit_state = struct_viewer_pane_state.selected_field_edit_state_label();
    let mut summary_lines = vec![
        "[ACT] r refresh-source | Enter commit edit.".to_string(),
        "[NAV] Up/Down select field.".to_string(),
        "[FMT] [ prev | ] next display format (blocked on uncommitted edit).".to_string(),
        "[EDIT] type | Backspace | Ctrl+u clear (value fields only).".to_string(),
        format!(
            "[SRC] source={:?} | selected_struct={} | field_count={}.",
            struct_viewer_pane_state.source,
            option_to_compact_text(struct_viewer_pane_state.selected_struct_name.as_deref()),
            struct_viewer_pane_state.focused_field_count()
        ),
        format!(
            "[SEL] field={} | edit_state={}.",
            option_to_compact_text(struct_viewer_pane_state.selected_field_name.as_deref()),
            selected_field_edit_state
        ),
        format!(
            "[FMT] active={} | index={}.",
            selected_field_display_format
                .map(|active_display_format| active_display_format.to_string())
                .unwrap_or_else(|| "none".to_string()),
            selected_field_display_format_progress
                .map(|(active_display_value_index, display_value_count)| { format!("{}/{}", active_display_value_index + 1, display_value_count) })
                .unwrap_or_else(|| "0/0".to_string())
        ),
        format!(
            "[EDIT] pending={} | uncommitted={} | committing={}.",
            struct_viewer_pane_state.pending_edit_text, struct_viewer_pane_state.has_uncommitted_edit, struct_viewer_pane_state.is_committing_edit
        ),
        format!(
            "[LINK] selected_scan_results={} | selected_project_items={}.",
            struct_viewer_pane_state.selected_scan_result_refs.len(),
            struct_viewer_pane_state.selected_project_item_paths.len()
        ),
        format!("[STAT] {}.", struct_viewer_pane_state.status_message),
    ];

    let visible_field_count = struct_viewer_pane_state
        .focused_field_count()
        .min(focused_field_preview_capacity);
    for field_position in 0..visible_field_count {
        if let Some(focused_field) = struct_viewer_pane_state
            .focused_struct
            .as_ref()
            .and_then(|focused_struct| focused_struct.get_fields().get(field_position))
        {
            let selected_marker = if struct_viewer_pane_state.selected_field_position == Some(field_position) {
                ">"
            } else {
                " "
            };
            let field_kind_marker = StructViewerPaneState::field_kind_marker(focused_field);
            let editability_marker = StructViewerPaneState::field_editability_marker(focused_field);
            let field_name = focused_field.get_name();
            let format_suffix = struct_viewer_pane_state
                .active_display_value_for_field(field_name)
                .map(|active_display_value| format!(" ({})", active_display_value.get_anonymous_value_string_format()))
                .unwrap_or_else(String::new);
            let value_preview = struct_viewer_pane_state
                .active_display_value_for_field(field_name)
                .map(|active_display_value| active_display_value.get_anonymous_value_string().to_string())
                .unwrap_or_else(|| "<nested>".to_string());
            summary_lines.push(format!(
                "{} [FLD {}|{}] {}{} = {}.",
                selected_marker, field_kind_marker, editability_marker, field_name, format_suffix, value_preview
            ));
        }
    }

    summary_lines
}

fn option_to_compact_text(option_text: Option<&str>) -> String {
    option_text
        .map(|text| format!("\"{}\"", text))
        .unwrap_or_else(|| "none".to_string())
}
