use squalr_engine_api::commands::pointer_scan::pointer_scan_response::PointerScanResponse;
use squalr_engine_api::structures::pointer_scans::pointer_scan_node::PointerScanNode;
use squalr_engine_api::structures::pointer_scans::pointer_scan_summary::PointerScanSummary;

pub fn handle_pointer_scan_response(pointer_scan_response: PointerScanResponse) {
    match pointer_scan_response {
        PointerScanResponse::Reset { pointer_scan_reset_response } => {
            if pointer_scan_reset_response.success {
                log::info!("Cleared the active pointer scan session.");
            } else {
                log::error!("Failed to clear the active pointer scan session.");
            }
        }
        PointerScanResponse::Start { pointer_scan_start_response } => {
            if let Some(pointer_scan_summary) = pointer_scan_start_response.pointer_scan_summary.as_ref() {
                log_pointer_scan_summary("Started pointer scan session", pointer_scan_summary);
            } else {
                log::info!("Pointer scan did not return a session summary.");
            }
        }
        PointerScanResponse::Summary { pointer_scan_summary_response } => {
            if let Some(pointer_scan_summary) = pointer_scan_summary_response.pointer_scan_summary.as_ref() {
                log_pointer_scan_summary("Pointer scan session summary", pointer_scan_summary);
            } else {
                log::info!("No active pointer scan session.");
            }
        }
        PointerScanResponse::Expand { pointer_scan_expand_response } => {
            if pointer_scan_expand_response.pointer_scan_nodes.is_empty() {
                log::info!(
                    "Pointer scan expansion for session {} returned no child nodes.",
                    pointer_scan_expand_response.session_id
                );
            } else {
                for pointer_scan_node in &pointer_scan_expand_response.pointer_scan_nodes {
                    log_pointer_scan_node(pointer_scan_node);
                }
            }
        }
        PointerScanResponse::Validate {
            pointer_scan_validate_response,
        } => {
            log::info!("{}", pointer_scan_validate_response.status_message);

            if let Some(pointer_scan_summary) = pointer_scan_validate_response.pointer_scan_summary.as_ref() {
                log_pointer_scan_summary("Pointer scan validation summary", pointer_scan_summary);
            }
        }
    }
}

fn log_pointer_scan_summary(
    label: &str,
    pointer_scan_summary: &PointerScanSummary,
) {
    log::info!(
        "{} {}: target=0x{:X}, pointer_size={}, max_depth={}, offset_radius=0x{:X}, roots={}, total_nodes={}, static_nodes={}, heap_nodes={}",
        label,
        pointer_scan_summary.get_session_id(),
        pointer_scan_summary.get_target_address(),
        pointer_scan_summary.get_pointer_size(),
        pointer_scan_summary.get_max_depth(),
        pointer_scan_summary.get_offset_radius(),
        pointer_scan_summary.get_root_node_count(),
        pointer_scan_summary.get_total_node_count(),
        pointer_scan_summary.get_total_static_node_count(),
        pointer_scan_summary.get_total_heap_node_count(),
    );

    for pointer_scan_level_summary in pointer_scan_summary.get_pointer_scan_level_summaries() {
        log::info!(
            "  depth {}: nodes={}, static_nodes={}, heap_nodes={}",
            pointer_scan_level_summary.get_depth(),
            pointer_scan_level_summary.get_node_count(),
            pointer_scan_level_summary.get_static_node_count(),
            pointer_scan_level_summary.get_heap_node_count(),
        );
    }
}

fn log_pointer_scan_node(pointer_scan_node: &PointerScanNode) {
    let module_label = if pointer_scan_node.get_module_name().is_empty() {
        "<heap>".to_string()
    } else {
        format!("{}+0x{:X}", pointer_scan_node.get_module_name(), pointer_scan_node.get_module_offset())
    };

    log::info!(
        "Pointer node {}: depth={}, class={:?}, base={}, pointer_address=0x{:X}, pointer_value=0x{:X}, offset={:+#X}, resolved_target=0x{:X}, child_count={}",
        pointer_scan_node.get_node_id(),
        pointer_scan_node.get_depth(),
        pointer_scan_node.get_pointer_scan_node_type(),
        module_label,
        pointer_scan_node.get_pointer_address(),
        pointer_scan_node.get_pointer_value(),
        pointer_scan_node.get_pointer_offset(),
        pointer_scan_node.get_resolved_target_address(),
        pointer_scan_node.get_child_node_ids().len(),
    );
}
