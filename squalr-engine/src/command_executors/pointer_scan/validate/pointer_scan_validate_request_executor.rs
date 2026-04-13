use crate::command_executors::pointer_scan::pointer_scan_target_resolver::PointerScanTargetResolver;
use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::command_executors::snapshot_region_builder::merge_memory_regions_into_snapshot_regions;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::pointer_scan::validate::pointer_scan_validate_request::PointerScanValidateRequest;
use squalr_engine_api::commands::pointer_scan::validate::pointer_scan_validate_response::PointerScanValidateResponse;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_api::structures::pointer_scans::pointer_scan_address_space::PointerScanAddressSpace;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use squalr_engine_scanning::pointer_scans::pointer_scan_materializer::PointerScanMaterializer;
use squalr_engine_scanning::pointer_scans::pointer_scan_validator::PointerScanValidator;
use squalr_engine_scanning::scan_settings_config::ScanSettingsConfig;
use squalr_engine_scanning::scanners::scan_execution_context::ScanExecutionContext;
use squalr_engine_scanning::scanners::value_collector_task::ValueCollector;
use squalr_engine_session::os::PageRetrievalMode;
use std::sync::{Arc, RwLock};

impl PrivilegedCommandRequestExecutor for PointerScanValidateRequest {
    type ResponseType = PointerScanValidateResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        let Some(process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        else {
            return PointerScanValidateResponse {
                validation_performed: false,
                validation_target_address: None,
                pruned_node_count: 0,
                status_message: "No opened process is available for pointer validation.".to_string(),
                pointer_scan_summary: None,
            };
        };

        let pointer_scan_results = match engine_privileged_state.get_pointer_scan_results().read() {
            Ok(pointer_scan_results_guard) => match pointer_scan_results_guard.as_ref() {
                Some(pointer_scan_results) => pointer_scan_results.clone(),
                None => {
                    return PointerScanValidateResponse {
                        validation_performed: false,
                        validation_target_address: None,
                        pruned_node_count: 0,
                        status_message: "No active pointer scan results are available.".to_string(),
                        pointer_scan_summary: None,
                    };
                }
            },
            Err(error) => {
                log::error!("Failed to acquire read lock on pointer scan results store: {}", error);

                return PointerScanValidateResponse {
                    validation_performed: false,
                    validation_target_address: None,
                    pruned_node_count: 0,
                    status_message: "Failed to access the active pointer scan results.".to_string(),
                    pointer_scan_summary: None,
                };
            }
        };

        if pointer_scan_results.get_session_id() != self.session_id {
            return PointerScanValidateResponse {
                validation_performed: false,
                validation_target_address: None,
                pruned_node_count: 0,
                status_message: format!("Pointer scan results {} were not found.", self.session_id),
                pointer_scan_summary: None,
            };
        }

        let modules = match pointer_scan_results.get_address_space() {
            PointerScanAddressSpace::Auto => engine_privileged_state
                .get_os_providers()
                .memory_query_raw
                .get_modules(&process_info),
            PointerScanAddressSpace::GameMemory => engine_privileged_state
                .get_os_providers()
                .memory_query
                .get_modules(&process_info),
            PointerScanAddressSpace::EmulatorMemory => engine_privileged_state
                .get_os_providers()
                .memory_query_raw
                .get_modules(&process_info),
        };
        let parsed_target_address = self.target.target_address.as_ref().and_then(|target_address| {
            let trimmed_target_address = target_address.get_anonymous_value_string().trim();

            if trimmed_target_address.is_empty() {
                return None;
            }

            match target_address.get_anonymous_value_string_format() {
                squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat::Address
                | squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat::Hexadecimal => {
                    let hexadecimal_input = trimmed_target_address
                        .strip_prefix("0x")
                        .or_else(|| trimmed_target_address.strip_prefix("0X"))
                        .unwrap_or(trimmed_target_address);

                    u64::from_str_radix(hexadecimal_input, 16).ok()
                }
                squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat::Decimal => {
                    trimmed_target_address.parse::<u64>().ok()
                }
                squalr_engine_api::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat::Binary => {
                    let binary_input = trimmed_target_address
                        .strip_prefix("0b")
                        .or_else(|| trimmed_target_address.strip_prefix("0B"))
                        .unwrap_or(trimmed_target_address);

                    u64::from_str_radix(binary_input, 2).ok()
                }
                _ => trimmed_target_address.parse::<u64>().ok(),
            }
        });
        let memory_regions = match pointer_scan_results.get_address_space() {
            PointerScanAddressSpace::Auto => engine_privileged_state
                .get_os_providers()
                .memory_query_raw
                .get_memory_page_bounds(&process_info, PageRetrievalMode::FromUserMode),
            PointerScanAddressSpace::GameMemory => engine_privileged_state
                .get_os_providers()
                .memory_query
                .get_pointer_scan_memory_page_bounds(&process_info, PageRetrievalMode::FromVirtualModules, parsed_target_address),
            PointerScanAddressSpace::EmulatorMemory => engine_privileged_state
                .get_os_providers()
                .memory_query_raw
                .get_memory_page_bounds(&process_info, PageRetrievalMode::FromUserMode),
        };
        if memory_regions.is_empty() {
            return PointerScanValidateResponse {
                validation_performed: false,
                validation_target_address: None,
                pruned_node_count: 0,
                status_message: "No readable memory regions were available for pointer validation.".to_string(),
                pointer_scan_summary: Some(pointer_scan_results.summarize()),
            };
        }
        let memory_read_provider = engine_privileged_state.get_os_providers().memory_read.clone();
        let scan_execution_context = ScanExecutionContext::new(
            None,
            None,
            Some(Arc::new(move |opened_process_info, address, values| {
                memory_read_provider.read_bytes(opened_process_info, address, values)
            })),
        );
        let mut validation_snapshot = Snapshot::new();
        validation_snapshot.set_snapshot_regions(merge_memory_regions_into_snapshot_regions(memory_regions));
        let validation_snapshot = Arc::new(RwLock::new(validation_snapshot));

        ValueCollector::collect_values(process_info.clone(), validation_snapshot.clone(), true, &scan_execution_context);
        let resolved_targets = match engine_privileged_state.read_symbol_registry(|symbol_registry| {
            PointerScanTargetResolver::resolve_targets(
                &self.target,
                symbol_registry,
                pointer_scan_results.get_pointer_size(),
                validation_snapshot.clone(),
                process_info.clone(),
                ScanSettingsConfig::get_memory_alignment().unwrap_or(MemoryAlignment::Alignment1),
                ScanSettingsConfig::get_floating_point_tolerance(),
                ScanSettingsConfig::get_is_single_threaded_scan(),
                ScanSettingsConfig::get_debug_perform_validation_scan(),
                &scan_execution_context,
            )
        }) {
            Ok(resolved_targets) => resolved_targets,
            Err(error) => {
                log::error!("{}", error);
                return PointerScanValidateResponse {
                    validation_performed: false,
                    validation_target_address: None,
                    pruned_node_count: 0,
                    status_message: error,
                    pointer_scan_summary: Some(pointer_scan_results.summarize()),
                };
            }
        };
        let validation_target_address = resolved_targets.target_descriptor.get_target_address();

        let validation_snapshot_guard = match validation_snapshot.read() {
            Ok(validation_snapshot_guard) => validation_snapshot_guard,
            Err(error) => {
                log::error!("Failed to acquire read lock on validation snapshot: {}", error);

                return PointerScanValidateResponse {
                    validation_performed: false,
                    validation_target_address: None,
                    pruned_node_count: 0,
                    status_message: "Failed to access the validation snapshot.".to_string(),
                    pointer_scan_summary: Some(pointer_scan_results.summarize()),
                };
            }
        };
        let validated_pointer_scan_results = PointerScanValidator::validate_scan(
            process_info,
            &pointer_scan_results,
            resolved_targets.target_descriptor,
            resolved_targets.target_addresses,
            &validation_snapshot_guard,
            &modules,
            &scan_execution_context,
            true,
        );
        let validated_pointer_scan_summary = validated_pointer_scan_results.summarize();
        let pruned_node_count = pointer_scan_results
            .get_total_node_count()
            .saturating_sub(validated_pointer_scan_results.get_total_node_count());

        match engine_privileged_state.get_pointer_scan_results().write() {
            Ok(mut pointer_scan_results_guard) => {
                *pointer_scan_results_guard = Some(validated_pointer_scan_results);
            }
            Err(error) => {
                log::error!("Failed to acquire write lock on pointer scan results store: {}", error);
            }
        }
        match engine_privileged_state.get_pointer_scan_materializer().write() {
            Ok(mut pointer_scan_materializer_guard) => {
                *pointer_scan_materializer_guard = Some(PointerScanMaterializer::new());
            }
            Err(error) => {
                log::error!("Failed to acquire write lock on pointer scan materializer store: {}", error);
            }
        }

        PointerScanValidateResponse {
            validation_performed: true,
            validation_target_address,
            pruned_node_count,
            status_message: format!(
                "Validated pointer scan results {} against {}. Pruned {} nodes.",
                self.session_id,
                validated_pointer_scan_summary.get_target_descriptor(),
                pruned_node_count
            ),
            pointer_scan_summary: Some(validated_pointer_scan_summary),
        }
    }
}
