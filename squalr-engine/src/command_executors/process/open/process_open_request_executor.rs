use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::process::open::process_open_request::ProcessOpenRequest;
use squalr_engine_api::commands::process::open::process_open_response::ProcessOpenResponse;
use squalr_engine_session::os::Pid;
use squalr_engine_session::os::ProcessQueryOptions;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for ProcessOpenRequest {
    type ResponseType = ProcessOpenResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        open_process(self, engine_privileged_state, None)
    }
}

pub(crate) fn open_process(
    request: &ProcessOpenRequest,
    engine_privileged_state: &Arc<EnginePrivilegedState>,
    expected_generation: Option<u64>,
) -> ProcessOpenResponse {
    if request.process_id.is_none() && request.search_name.is_none() {
        log::error!("Error: Neither PID nor search name provided. Cannot open process.");
        return ProcessOpenResponse { opened_process_info: None };
    }

    log::info!("Opening process...");

    let options = ProcessQueryOptions {
        search_name: request.search_name.clone(),
        required_process_id: request.process_id.map(Pid::from_u32),
        require_windowed: false,
        match_case: request.match_case,
        fetch_icons: false,
        limit: Some(1),
    };

    let os_providers = engine_privileged_state.get_os_providers();
    let processes = os_providers.process_query.get_processes(options);

    if let Some(process_info) = processes.first() {
        match os_providers.process_query.open_process(&process_info) {
            Ok(opened_process_info) => {
                if let Err(error) = engine_privileged_state
                    .get_process_manager()
                    .replace_process(Some(opened_process_info.clone()), expected_generation)
                {
                    log::error!("Failed to replace opened process: {error}");
                    if let Err(cleanup_error) = os_providers
                        .process_query
                        .close_process(opened_process_info.get_handle())
                    {
                        log::error!("Failed to release uninstalled process handle: {cleanup_error}");
                    }
                    return ProcessOpenResponse { opened_process_info: None };
                }

                return ProcessOpenResponse {
                    opened_process_info: Some(opened_process_info),
                };
            }
            Err(error) => {
                log::info!("Failed to open process {}: {}", process_info.get_process_id_raw(), error);
            }
        }
    } else {
        log::error!("No matching process found.");
    }

    ProcessOpenResponse { opened_process_info: None }
}
