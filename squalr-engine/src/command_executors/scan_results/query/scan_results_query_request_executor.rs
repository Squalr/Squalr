use crate::command_executors::engine_request_executor::EngineRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan_results::query::scan_results_query_request::ScanResultsQueryRequest;
use squalr_engine_api::commands::scan_results::query::scan_results_query_response::ScanResultsQueryResponse;
use squalr_engine_scanning::scan_settings::ScanSettings;
use std::sync::Arc;

impl EngineRequestExecutor for ScanResultsQueryRequest {
    type ResponseType = ScanResultsQueryResponse;

    fn execute(
        &self,
        execution_context: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineRequestExecutor>::ResponseType {
        let results_page_size = ScanSettings::get_instance().get_results_page_size() as u64;
        let mut scan_results_list = vec![];
        let mut last_page_index = 0;
        let mut result_count = 0;
        let mut total_size_in_bytes = 0;

        if let Ok(snapshot) = execution_context.get_snapshot().read() {
            result_count = snapshot.get_number_of_results();
            last_page_index = result_count / results_page_size;
            total_size_in_bytes = snapshot.get_byte_count();

            // Get the range of indicies for the elements of this page.
            let index_of_first_page_entry = self.page_index.clamp(0, last_page_index) * results_page_size;
            let index_of_last_page_entry = index_of_first_page_entry + results_page_size;

            for result_index in index_of_first_page_entry..index_of_last_page_entry {
                let scan_result_base = match snapshot.get_scan_result(result_index) {
                    None => break,
                    Some(scan_result_base) => scan_result_base,
                };

                scan_results_list.push(scan_result_base);
            }
        }

        ScanResultsQueryResponse {
            scan_results: scan_results_list,
            page_index: self.page_index,
            page_size: results_page_size,
            last_page_index,
            result_count,
            total_size_in_bytes,
        }
    }
}
