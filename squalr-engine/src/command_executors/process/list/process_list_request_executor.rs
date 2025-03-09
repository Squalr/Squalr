use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::process::list::process_list_request::ProcessListRequest;
use squalr_engine_api::commands::process::list::process_list_response::ProcessListResponse;
use squalr_engine_processes::process_query::process_query_options::ProcessQueryOptions;
use squalr_engine_processes::process_query::process_queryer::ProcessQuery;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ProcessListRequest {
    type ResponseType = ProcessListResponse;

    fn execute(
        &self,
        _execution_context: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
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

        let processes = ProcessQuery::get_processes(options);

        ProcessListResponse { processes }
    }
}
