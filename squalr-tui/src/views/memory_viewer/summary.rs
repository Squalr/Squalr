use crate::views::memory_viewer::pane_state::{MemoryViewerInputMode, MemoryViewerPaneState};

pub fn build_memory_viewer_summary_lines(memory_viewer_pane_state: &MemoryViewerPaneState) -> Vec<String> {
    let action_line = match memory_viewer_pane_state.input_mode {
        MemoryViewerInputMode::Normal => {
            String::from("[ACT] r refresh | g// seek | [/] page | Arrows move | Shift extend | PgUp/PgDn jump | Ctrl+A page | Hex edit.")
        }
        MemoryViewerInputMode::SeekInput => String::from("[SEEK] Enter commit | Esc cancel | Backspace delete | Ctrl+U clear."),
    };
    let selection_text = memory_viewer_pane_state
        .selection_summary()
        .map(|selection_summary| selection_summary.selection_display_text)
        .unwrap_or_else(|| String::from("none"));
    let selected_byte_count = memory_viewer_pane_state
        .get_selected_address_bounds()
        .map(|(selection_start_address, selection_end_address)| {
            selection_end_address
                .saturating_sub(selection_start_address)
                .saturating_add(1)
        })
        .unwrap_or(0);

    let mut summary_lines = vec![
        action_line,
        format!(
            "[PAGE] index={} / {} | rows={} | row={} | mode={:?} | loading={}.",
            memory_viewer_pane_state.current_page_index.saturating_add(1),
            memory_viewer_pane_state
                .cached_last_page_index
                .saturating_add(1),
            memory_viewer_pane_state.current_page_row_count(),
            memory_viewer_pane_state.selected_row_index.saturating_add(1),
            memory_viewer_pane_state.page_retrieval_mode,
            memory_viewer_pane_state.is_querying_memory_pages
        ),
        format!(
            "[ADDR] page_base={} | row_start={} | cursor={} | selection={} | bytes={}.",
            option_hex(memory_viewer_pane_state.current_page_base_address()),
            option_hex(memory_viewer_pane_state.selected_row_address()),
            option_hex(memory_viewer_pane_state.selected_cursor_address()),
            selection_text,
            selected_byte_count
        ),
    ];

    if memory_viewer_pane_state.input_mode == MemoryViewerInputMode::SeekInput {
        summary_lines.push(format!("[INP] goto={}.", memory_viewer_pane_state.pending_seek_input));
    }

    summary_lines.push(format!("[STAT] {}.", memory_viewer_pane_state.stats_string));
    summary_lines.push(format!("[INFO] {}.", memory_viewer_pane_state.status_message));

    if memory_viewer_pane_state.virtual_pages.is_empty() {
        summary_lines.insert(1, String::from("[PAGE] No memory pages loaded."));
    }

    summary_lines
}

fn option_hex(address: Option<u64>) -> String {
    address
        .map(|address| format!("0x{:X}", address))
        .unwrap_or_else(|| String::from("none"))
}
