use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::pointer_scan::expand::pointer_scan_expand_request::PointerScanExpandRequest;
use squalr_engine_api::commands::pointer_scan::expand::pointer_scan_expand_response::PointerScanExpandResponse;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for PointerScanExpandRequest {
    type ResponseType = PointerScanExpandResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let pointer_scan_session_store = engine_privileged_state.get_pointer_scan_session();
        let pointer_scan_browser_store = engine_privileged_state.get_pointer_scan_browser();

        let mut pointer_scan_session_guard = match pointer_scan_session_store.write() {
            Ok(pointer_scan_session_guard) => pointer_scan_session_guard,
            Err(error) => {
                log::error!("Failed to acquire write lock on pointer scan session store: {}", error);
                return PointerScanExpandResponse {
                    session_id: self.session_id,
                    parent_node_id: self.parent_node_id,
                    page_index: 0,
                    last_page_index: 0,
                    total_node_count: 0,
                    pointer_scan_nodes: Vec::new(),
                };
            }
        };
        let mut pointer_scan_browser_guard = match pointer_scan_browser_store.write() {
            Ok(pointer_scan_browser_guard) => pointer_scan_browser_guard,
            Err(error) => {
                log::error!("Failed to acquire write lock on pointer scan browser store: {}", error);
                return PointerScanExpandResponse {
                    session_id: self.session_id,
                    parent_node_id: self.parent_node_id,
                    page_index: 0,
                    last_page_index: 0,
                    total_node_count: 0,
                    pointer_scan_nodes: Vec::new(),
                };
            }
        };

        if let (Some(pointer_scan_session), Some(pointer_scan_browser)) = (pointer_scan_session_guard.as_mut(), pointer_scan_browser_guard.as_mut()) {
            if pointer_scan_session.get_session_id() == self.session_id {
                let (pointer_scan_nodes, page_index, last_page_index, total_node_count) =
                    pointer_scan_browser.get_expanded_node_page(pointer_scan_session, self.parent_node_id, self.page_index, self.page_size);

                return PointerScanExpandResponse {
                    session_id: self.session_id,
                    parent_node_id: self.parent_node_id,
                    page_index,
                    last_page_index,
                    total_node_count,
                    pointer_scan_nodes,
                };
            }
        }

        PointerScanExpandResponse {
            session_id: self.session_id,
            parent_node_id: self.parent_node_id,
            page_index: 0,
            last_page_index: 0,
            total_node_count: 0,
            pointer_scan_nodes: Vec::new(),
        }
    }
}
