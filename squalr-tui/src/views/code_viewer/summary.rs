use crate::views::code_viewer::pane_state::CodeViewerPaneState;
use squalr_engine_api::structures::memory::bitness::Bitness;

pub fn build_code_viewer_summary_lines(
    code_viewer_pane_state: &CodeViewerPaneState,
    process_bitness: Option<Bitness>,
) -> Vec<String> {
    let mut summary_lines = vec![
        String::from("[ACT] r refresh pages | [/] page | Home/End first/last page | Up/Down instruction | PgUp/PgDn jump."),
        format!(
            "[PAGE] index={} / {} | instructions={} | selected={} | bitness={} | loading={}.",
            code_viewer_pane_state.current_page_index.saturating_add(1),
            code_viewer_pane_state.cached_last_page_index.saturating_add(1),
            code_viewer_pane_state.current_instruction_count(process_bitness),
            option_hex(code_viewer_pane_state.selected_instruction_address()),
            option_bitness(process_bitness),
            code_viewer_pane_state.is_querying_memory_pages
        ),
        format!(
            "[ADDR] page_base={} | viewport={} | viewport_rows={}.",
            option_hex(code_viewer_pane_state.current_page_base_address()),
            code_viewer_pane_state.viewport_range_label(),
            code_viewer_pane_state.last_visible_row_capacity
        ),
        format!("[STAT] {}.", code_viewer_pane_state.stats_string),
        format!("[INFO] {}.", code_viewer_pane_state.status_message),
    ];

    if code_viewer_pane_state.virtual_pages.is_empty() {
        summary_lines.insert(1, String::from("[PAGE] No code pages loaded."));
    }

    summary_lines
}

fn option_hex(address: Option<u64>) -> String {
    address
        .map(|address| format!("0x{:X}", address))
        .unwrap_or_else(|| String::from("none"))
}

fn option_bitness(process_bitness: Option<Bitness>) -> &'static str {
    match process_bitness {
        Some(Bitness::Bit32) => "32-bit",
        Some(Bitness::Bit64) => "64-bit",
        None => "none",
    }
}
