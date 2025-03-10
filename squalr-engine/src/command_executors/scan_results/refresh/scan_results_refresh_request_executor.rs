use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan_results::refresh::scan_results_refresh_request::ScanResultsRefreshRequest;
use squalr_engine_api::commands::scan_results::refresh::scan_results_refresh_response::ScanResultsRefreshResponse;
use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
use squalr_engine_memory::memory_queryer::memory_queryer::MemoryQueryer;
use squalr_engine_memory::memory_queryer::memory_queryer_trait::IMemoryQueryer;
use squalr_engine_memory::memory_reader::MemoryReader;
use squalr_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ScanResultsRefreshRequest {
    type ResponseType = ScanResultsRefreshResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        let mut scan_results_list = vec![];

        // Collect modules if possible so that we can resolve whether individual addresses are static later.
        let modules = if let Some(opened_process_info) = engine_privileged_state.get_opened_process() {
            MemoryQueryer::get_instance().get_modules(&opened_process_info)
        } else {
            vec![]
        };

        // Wrap each ScanResultBase with a full ScanResult that includes current values and module information.
        for scan_result_base in self.scan_results.clone().into_iter() {
            let mut recently_read_value = None;
            let mut module_name = String::default();
            let mut module_offset = scan_result_base.address;

            // Best-effort attempt to read the values for this scan result.
            if let Some(opened_process_info) = engine_privileged_state.get_opened_process() {
                if let Some(mut data_value) = scan_result_base.data_type.get_default_value() {
                    if MemoryReader::get_instance().read(&opened_process_info, scan_result_base.address, &mut data_value) {
                        recently_read_value = Some(data_value);
                    }
                }
            }

            // Check whether this scan result belongs to a module (ie check if the address is static).
            if let Some((found_module_name, address)) = MemoryQueryer::get_instance().address_to_module(scan_result_base.address, &modules) {
                module_name = found_module_name;
                module_offset = address;
            }

            scan_results_list.push(ScanResult::new(scan_result_base, module_name, module_offset, recently_read_value));
        }

        ScanResultsRefreshResponse {
            scan_results: scan_results_list,
        }
    }
}
