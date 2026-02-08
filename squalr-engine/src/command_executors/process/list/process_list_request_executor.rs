use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::process::list::process_list_request::ProcessListRequest;
use squalr_engine_api::commands::process::list::process_list_response::ProcessListResponse;
use squalr_engine_processes::process_query::process_query_options::ProcessQueryOptions;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for ProcessListRequest {
    type ResponseType = ProcessListResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        log::info!(
            "Listing processes with options: require_windowed={}, search_name={:?}, match_case={}, limit={:?}",
            self.require_windowed,
            self.search_name,
            self.match_case,
            self.limit
        );

        let options = ProcessQueryOptions {
            search_name: self.search_name.as_ref().cloned(),
            required_process_id: None,
            require_windowed: self.require_windowed,
            match_case: self.match_case,
            fetch_icons: self.fetch_icons,
            limit: self.limit,
        };

        let processes = engine_privileged_state
            .get_os_providers()
            .process_query
            .get_processes(options);

        ProcessListResponse { processes }
    }
}
