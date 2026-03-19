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
        match engine_privileged_state.get_pointer_scan_session().write() {
            Ok(mut pointer_scan_session_guard) => {
                if let Some(pointer_scan_session) = pointer_scan_session_guard.as_mut() {
                    if pointer_scan_session.get_session_id() == self.session_id {
                        return PointerScanExpandResponse {
                            session_id: self.session_id,
                            parent_node_id: self.parent_node_id,
                            pointer_scan_nodes: pointer_scan_session.get_expanded_nodes(self.parent_node_id),
                        };
                    }
                }

                PointerScanExpandResponse {
                    session_id: self.session_id,
                    parent_node_id: self.parent_node_id,
                    pointer_scan_nodes: Vec::new(),
                }
            }
            Err(error) => {
                log::error!("Failed to acquire write lock on pointer scan session store: {}", error);

                PointerScanExpandResponse {
                    session_id: self.session_id,
                    parent_node_id: self.parent_node_id,
                    pointer_scan_nodes: Vec::new(),
                }
            }
        }
    }
}
