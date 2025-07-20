use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use olorin_engine_api::commands::scan_results::refresh::scan_results_refresh_request::ScanResultsRefreshRequest;
use olorin_engine_api::commands::scan_results::refresh::scan_results_refresh_response::ScanResultsRefreshResponse;
use olorin_engine_api::structures::scan_results::scan_result::ScanResult;
use olorin_engine_memory::memory_queryer::memory_queryer::MemoryQueryer;
use olorin_engine_memory::memory_queryer::memory_queryer_trait::IMemoryQueryer;
use olorin_engine_memory::memory_reader::MemoryReader;
use olorin_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ScanResultsRefreshRequest {
    type ResponseType = ScanResultsRefreshResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        let mut scan_results_list = vec![];

        // Collect modules if possible so that we can resolve whether individual addresses are static later.
        let modules = if let Some(opened_process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        {
            MemoryQueryer::get_instance().get_modules(&opened_process_info)
        } else {
            vec![]
        };

        // Wrap each ScanResultBase with a full ScanResult that includes current values and module information.
        for scan_result_base in self.scan_results.clone().into_iter() {
            let mut recently_read_value = None;
            let mut module_name = String::default();
            let address = scan_result_base.get_address();
            let mut module_offset = scan_result_base.get_address();

            // Best-effort attempt to read the values for this scan result.
            if let Some(opened_process_info) = engine_privileged_state
                .get_process_manager()
                .get_opened_process()
            {
                if let Some(mut data_value) = scan_result_base.get_current_value().clone() {
                    if MemoryReader::get_instance().read(&opened_process_info, address, &mut data_value) {
                        recently_read_value = Some(data_value);
                    }
                }
            }

            // Check whether this scan result belongs to a module (ie check if the address is static).
            if let Some((found_module_name, address)) = MemoryQueryer::get_instance().address_to_module(address, &modules) {
                module_name = found_module_name;
                module_offset = address;
            }

            let is_frozen = if let Ok(freeze_list_registry) = engine_privileged_state.get_freeze_list_registry().read() {
                freeze_list_registry.is_address_frozen(address)
            } else {
                false
            };

            scan_results_list.push(ScanResult::new(scan_result_base, module_name, module_offset, recently_read_value, is_frozen));
        }

        ScanResultsRefreshResponse {
            scan_results: scan_results_list,
        }
    }
}
