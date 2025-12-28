use crate::command_executors::engine_request_executor::EngineCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan_results::freeze::scan_results_freeze_request::ScanResultsFreezeRequest;
use squalr_engine_api::commands::scan_results::freeze::scan_results_freeze_response::ScanResultsFreezeResponse;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::memory::pointer::Pointer;
use squalr_engine_memory::memory_queryer::memory_queryer::MemoryQueryer;
use squalr_engine_memory::memory_queryer::memory_queryer_trait::IMemoryQueryer;
use squalr_engine_memory::memory_reader::MemoryReader;
use squalr_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;
use std::sync::Arc;

impl EngineCommandRequestExecutor for ScanResultsFreezeRequest {
    type ResponseType = ScanResultsFreezeResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as EngineCommandRequestExecutor>::ResponseType {
        let symbol_registry = SymbolRegistry::get_instance();
        let snapshot = engine_privileged_state.get_snapshot();
        let snapshot_guard = match snapshot.read() {
            Ok(snapshot) => snapshot,
            Err(error) => {
                log::error!("Failed to acquire read lock on Snapshot: {}", error);

                return ScanResultsFreezeResponse::default();
            }
        };
        let freeze_list_registry = engine_privileged_state.get_freeze_list_registry();
        let mut freeze_list_registry_guard = match freeze_list_registry.write() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire write lock on FreezeListRegistry: {}", error);

                return ScanResultsFreezeResponse::default();
            }
        };

        // Collect modules if possible so that we can resolve whether individual addresses are static later.
        let modules = if let Some(opened_process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        {
            MemoryQueryer::get_instance().get_modules(&opened_process_info)
        } else {
            vec![]
        };

        for scan_result_ref in &self.scan_result_refs {
            if let Some(scan_result) = snapshot_guard.get_scan_result(scan_result_ref.get_scan_result_index()) {
                let address = scan_result.get_address();
                let mut module_name = String::default();
                let mut module_offset = scan_result.get_address();

                // Check whether this scan result belongs to a module (ie check if the address is static).
                if let Some((found_module_name, address)) = MemoryQueryer::get_instance().address_to_module(module_offset, &modules) {
                    module_name = found_module_name;
                    module_offset = address;
                }

                let pointer = Pointer::new(module_offset, vec![], module_name);

                if self.is_frozen {
                    if let Some(opened_process_info) = engine_privileged_state
                        .get_process_manager()
                        .get_opened_process()
                    {
                        let data_type_ref = scan_result.get_data_type_ref();

                        if let Some(mut data_value) = symbol_registry.get_default_value(data_type_ref) {
                            if MemoryReader::get_instance().read(&opened_process_info, address, &mut data_value) {
                                freeze_list_registry_guard.set_address_frozen(pointer, data_value.get_value_bytes().to_vec());
                            }
                        }
                    }
                } else {
                    freeze_list_registry_guard.set_address_unfrozen(&pointer);
                }
            }
        }

        ScanResultsFreezeResponse {}
    }
}
