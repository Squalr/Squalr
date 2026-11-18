use crate::{command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor, engine_privileged_state::EnginePrivilegedState};
use squalr_engine_api::commands::process::close::{process_close_request::ProcessCloseRequest, process_close_response::ProcessCloseResponse};
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for ProcessCloseRequest {
    type ResponseType = ProcessCloseResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        close_process(engine_privileged_state, None)
    }
}

pub(crate) fn close_process(
    engine_privileged_state: &Arc<EnginePrivilegedState>,
    expected_generation: Option<u64>,
) -> ProcessCloseResponse {
    let process_info = engine_privileged_state
        .get_process_manager()
        .replace_process(None, expected_generation)
        .unwrap_or_else(|error| {
            log::error!("Failed to close process: {error}");
            None
        });
    ProcessCloseResponse { process_info }
}
