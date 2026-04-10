use crate::views::memory_viewer::pane_state::MemoryViewerPaneState;

pub fn build_memory_viewer_summary_lines(memory_viewer_pane_state: &MemoryViewerPaneState) -> Vec<String> {
    let mut summary_lines = vec![
        String::from("[ACT] r refresh pages | [/] page | Home/End first/last page | Up/Down row | PgUp/PgDn jump."),
        format!(
            "[PAGE] index={} / {} | rows={} | selected_row={} | mode={:?} | loading={}.",
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
            "[ADDR] page_base={} | row_addr={} | viewport_rows={}.",
            option_hex(memory_viewer_pane_state.current_page_base_address()),
            option_hex(memory_viewer_pane_state.selected_row_address()),
            memory_viewer_pane_state.last_visible_row_capacity
        ),
        format!("[STAT] {}.", memory_viewer_pane_state.stats_string),
        format!("[INFO] {}.", memory_viewer_pane_state.status_message),
    ];

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
