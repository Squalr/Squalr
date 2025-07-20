use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use olorin_engine_api::commands::scan_results::freeze::scan_results_freeze_request::ScanResultsFreezeRequest;
use olorin_engine_api::commands::scan_results::freeze::scan_results_freeze_response::ScanResultsFreezeResponse;
use olorin_engine_memory::memory_reader::MemoryReader;
use olorin_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ScanResultsFreezeRequest {
    type ResponseType = ScanResultsFreezeResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        let data_type_registry = engine_privileged_state.get_data_type_registry();
        let data_type_registry_guard = match data_type_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on DataTypeRegistry: {}", error);

                return ScanResultsFreezeResponse {};
            }
        };
        let freeze_list_registry = engine_privileged_state.get_freeze_list_registry();
        let mut freeze_list_registry_guard = match freeze_list_registry.write() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire write lock on FreezeListRegistry: {}", error);

                return ScanResultsFreezeResponse {};
            }
        };

        for scan_result in &self.scan_results {
            let address = scan_result.get_address();
            if self.is_frozen {
                if let Some(opened_process_info) = engine_privileged_state
                    .get_process_manager()
                    .get_opened_process()
                {
                    let data_type_ref = scan_result.get_data_type_ref();

                    if let Some(mut data_value) = data_type_registry_guard.get_default_value(data_type_ref) {
                        if MemoryReader::get_instance().read(&opened_process_info, address, &mut data_value) {
                            freeze_list_registry_guard.set_address_frozen(address, data_value);
                        }
                    }
                }
            } else {
                freeze_list_registry_guard.set_address_unfrozen(address);
            }
        }

        ScanResultsFreezeResponse {}
    }
}
