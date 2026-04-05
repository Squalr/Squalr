use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan_results::list::scan_results_list_request::ScanResultsListRequest;
use squalr_engine_api::commands::scan_results::list::scan_results_list_response::ScanResultsListResponse;
use squalr_engine_api::structures::memory::pointer::Pointer;
use squalr_engine_api::structures::scan_results::scan_result::ScanResult;
use squalr_engine_scanning::scan_settings_config::ScanSettingsConfig;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for ScanResultsListRequest {
    type ResponseType = ScanResultsListResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let results_page_size = (ScanSettingsConfig::get_results_page_size() as u64).max(1);
        let mut scan_results_list = vec![];
        let mut data_type_result_counts = vec![];
        let mut last_page_index = 0;
        let mut result_count = 0;
        let mut total_size_in_bytes = 0;
        let mut effective_page_index = 0;
        let os_providers = engine_privileged_state.get_os_providers();

        // Collect modules if possible so that we can resolve whether individual addresses are static later.
        let modules = if let Some(opened_process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        {
            os_providers.memory_query.get_modules(&opened_process_info)
        } else {
            vec![]
        };

        if let Ok(snapshot) = engine_privileged_state.get_snapshot().read() {
            engine_privileged_state.read_symbol_registry(|symbol_registry| {
                data_type_result_counts = snapshot.get_result_counts_by_data_type(symbol_registry);
                result_count = snapshot.get_number_of_results_for_data_types(symbol_registry, self.data_type_filters.as_deref());
                last_page_index = result_count.saturating_sub(1) / results_page_size;
                total_size_in_bytes = snapshot.get_byte_count();
                let (resolved_page_index, scan_results_page) =
                    snapshot.get_scan_results_page(symbol_registry, self.data_type_filters.as_deref(), self.page_index, results_page_size);
                effective_page_index = resolved_page_index;

                for scan_result_base in scan_results_page {
                    let mut recently_read_value = None;
                    let mut module_name = String::default();
                    let address = scan_result_base.get_address();
                    let mut module_offset = address;

                    // Best-effort attempt to read the values for this scan result.
                    if let Some(opened_process_info) = engine_privileged_state
                        .get_process_manager()
                        .get_opened_process()
                    {
                        let data_type_ref = scan_result_base.get_data_type_ref();

                        if let Some(mut data_value) = symbol_registry.get_default_value(data_type_ref) {
                            if os_providers
                                .memory_read
                                .read(&opened_process_info, address, &mut data_value)
                            {
                                recently_read_value = Some(data_value);
                            }
                        }
                    }

                    // Check whether this scan result belongs to a module (ie check if the address is static).
                    if let Some((found_module_name, address)) = os_providers.memory_query.address_to_module(address, &modules) {
                        module_name = found_module_name;
                        module_offset = address;
                    }

                    let pointer = Pointer::new(module_offset, vec![], module_name.clone());
                    let is_frozen = if let Ok(freeze_list_registry) = engine_privileged_state.get_freeze_list_registry().read() {
                        freeze_list_registry.is_address_frozen(&pointer)
                    } else {
                        false
                    };

                    let recently_read_display_values = recently_read_value
                        .as_ref()
                        .and_then(|data_value| {
                            symbol_registry
                                .anonymize_value_to_supported_formats(data_value)
                                .ok()
                        })
                        .unwrap_or_default();

                    scan_results_list.push(ScanResult::new(
                        scan_result_base,
                        module_name,
                        module_offset,
                        recently_read_value,
                        recently_read_display_values,
                        is_frozen,
                    ));
                }
            });
        }

        ScanResultsListResponse {
            scan_results: scan_results_list,
            data_type_result_counts,
            page_index: effective_page_index,
            page_size: results_page_size,
            last_page_index,
            result_count,
            total_size_in_bytes,
        }
    }
}
