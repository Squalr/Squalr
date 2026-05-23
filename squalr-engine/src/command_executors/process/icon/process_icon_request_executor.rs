use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::process::icon::process_icon_request::ProcessIconRequest;
use squalr_engine_api::commands::process::icon::process_icon_response::{ProcessIconEntry, ProcessIconResponse};
use squalr_engine_session::os::Pid;
use squalr_engine_session::os::ProcessQueryOptions;
use std::collections::HashSet;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for ProcessIconRequest {
    type ResponseType = ProcessIconResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let unique_process_ids = self
            .process_ids
            .iter()
            .copied()
            .collect::<HashSet<_>>();
        let process_query = &engine_privileged_state.get_os_providers().process_query;
        let process_icons = unique_process_ids
            .into_iter()
            .map(|process_id| {
                let options = ProcessQueryOptions {
                    search_name: None,
                    required_process_id: Some(Pid::from_u32(process_id)),
                    require_windowed: false,
                    match_case: false,
                    fetch_icons: true,
                    limit: Some(1),
                };

                let process_icon = process_query
                    .get_processes(options)
                    .into_iter()
                    .next()
                    .and_then(|process_info| process_info.get_icon().clone());

                ProcessIconEntry {
                    process_id,
                    process_icon,
                }
            })
            .collect();

        ProcessIconResponse { process_icons }
    }
}
