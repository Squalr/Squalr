use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::command_executors::scan::snapshot_value_collector::SnapshotValueCollector;
use crate::command_executors::snapshot_region_builder::merge_memory_regions_into_snapshot_regions;
use crate::engine_privileged_state::EnginePrivilegedState;
use crate::services::pointer_scans::pointer_scan_target_resolution::PointerScanTargetResolver;
use squalr_engine_api::commands::pointer_scan::validate::pointer_scan_compare_request::PointerScanCompareRequest;
use squalr_engine_api::commands::pointer_scan::validate::pointer_scan_validate_request::PointerScanValidateRequest;
use squalr_engine_api::commands::pointer_scan::validate::pointer_scan_validate_response::PointerScanValidateResponse;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_api::structures::pointer_scans::pointer_scan_address_space::PointerScanAddressSpace;
use squalr_engine_api::structures::pointer_scans::pointer_scan_results::PointerScanResults;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use squalr_engine_scanning::ScanControl;
use squalr_engine_scanning::pointer_scans::pointer_scan_validation_input::PointerScanValidationInput;
use squalr_engine_scanning::pointer_scans::pointer_scan_validator::PointerScanValidator;
use squalr_engine_session::os::PageRetrievalMode;
use squalr_engine_session::settings::scan_settings_store::ScanSettingsStore;
use std::sync::{Arc, RwLock};

impl PrivilegedCommandRequestExecutor for PointerScanValidateRequest {
    type ResponseType = PointerScanValidateResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        execute_target_check(self, engine_privileged_state, false)
    }
}

impl PrivilegedCommandRequestExecutor for PointerScanCompareRequest {
    type ResponseType = PointerScanValidateResponse;
    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> PointerScanValidateResponse {
        execute_target_check(&self.0, engine_privileged_state, true)
    }
}

fn execute_target_check(
    request: &PointerScanValidateRequest,
    engine_privileged_state: &Arc<EnginePrivilegedState>,
    compare: bool,
) -> PointerScanValidateResponse {
    let Some(process_info) = engine_privileged_state
        .get_process_manager()
        .get_opened_process()
    else {
        return PointerScanValidateResponse {
            status_message: "No opened process is available for pointer validation.".to_string(),
            ..Default::default()
        };
    };

    let (input, graph_identity) = match engine_privileged_state.get_pointer_scan_results().read() {
        Ok(pointer_scan_results_guard) => match pointer_scan_results_guard.as_ref() {
            Some(results) if results.get_session_id() == request.session_id => (PointerScanValidationInput::from(results), results.graph_identity()),
            Some(_) => {
                return PointerScanValidateResponse {
                    status_message: format!("Pointer scan results {} were not found.", request.session_id),
                    ..Default::default()
                };
            }
            None => {
                return PointerScanValidateResponse {
                    status_message: "No active pointer scan results are available.".to_string(),
                    ..Default::default()
                };
            }
        },
        Err(error) => {
            log::error!("Failed to acquire read lock on pointer scan results store: {}", error);

            return PointerScanValidateResponse {
                status_message: "Failed to access the active pointer scan results.".to_string(),
                ..Default::default()
            };
        }
    };

    let modules = match input.summary.get_address_space() {
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
    let parsed_target_address = request
        .target
        .target_address
        .as_ref()
        .and_then(|target_address| {
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
    let memory_regions = match input.summary.get_address_space() {
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
            status_message: "No readable memory regions were available for pointer validation.".to_string(),
            pointer_scan_summary: Some(input.summary.clone()),
            ..Default::default()
        };
    }
    let mut validation_snapshot = Snapshot::new();
    validation_snapshot.set_snapshot_regions(merge_memory_regions_into_snapshot_regions(memory_regions));
    let validation_snapshot = Arc::new(RwLock::new(validation_snapshot));

    SnapshotValueCollector::collect_values(
        process_info.clone(),
        validation_snapshot.clone(),
        engine_privileged_state.get_os_providers().memory_read.clone(),
        true,
    );
    let resolved_targets = match engine_privileged_state.read_symbol_registry(|symbol_registry| {
        PointerScanTargetResolver::resolve_targets(
            &request.target,
            symbol_registry,
            input.summary.get_pointer_size(),
            validation_snapshot.clone(),
            ScanSettingsStore::get_memory_alignment().unwrap_or(MemoryAlignment::Alignment1),
            ScanSettingsStore::get_floating_point_tolerance(),
            ScanSettingsStore::get_is_single_threaded_scan(),
            ScanSettingsStore::get_debug_perform_validation_scan(),
        )
    }) {
        Ok(resolved_targets) => resolved_targets,
        Err(error) => {
            log::error!("{}", error);
            return PointerScanValidateResponse {
                status_message: error,
                pointer_scan_summary: Some(input.summary.clone()),
                ..Default::default()
            };
        }
    };
    let validation_target_address = resolved_targets.target_descriptor.get_target_address();

    let validation_snapshot_guard = match validation_snapshot.read() {
        Ok(validation_snapshot_guard) => validation_snapshot_guard,
        Err(error) => {
            log::error!("Failed to acquire read lock on validation snapshot: {}", error);

            return PointerScanValidateResponse {
                status_message: "Failed to access the validation snapshot.".to_string(),
                pointer_scan_summary: Some(input.summary.clone()),
                ..Default::default()
            };
        }
    };
    if compare {
        let shared = PointerScanValidator::shared_roots(
            &input,
            &resolved_targets.target_addresses,
            &validation_snapshot_guard,
            &modules,
            &ScanControl::default(),
        );
        return update_current_results(engine_privileged_state, &graph_identity, |primary| {
            let exclusive = primary.prioritize_exclusive_roots(&shared);
            let shared_count = primary.get_total_static_node_count().saturating_sub(exclusive);
            PointerScanValidateResponse {
                validation_performed: true,
                validation_target_address,
                exclusive_root_count: Some(exclusive),
                status_message: format!(
                    "Compared with {}. Showing {exclusive} exclusive static roots first, followed by {shared_count} shared roots.",
                    resolved_targets.target_descriptor
                ),
                pointer_scan_summary: Some(input.summary),
                ..Default::default()
            }
        });
    }
    let validated = PointerScanValidator::validate_from_roots(
        &input,
        resolved_targets.target_descriptor,
        resolved_targets.target_addresses,
        &validation_snapshot_guard,
        &modules,
        &ScanControl::default(),
        true,
    );
    let summary = validated.summarize();
    let pruned_static_root_count = input
        .summary
        .get_total_static_node_count()
        .saturating_sub(summary.get_total_static_node_count());
    let pruned_node_count = input
        .summary
        .get_total_node_count()
        .saturating_sub(summary.get_total_node_count());
    update_current_results(engine_privileged_state, &graph_identity, |primary| {
        *primary = validated;
        PointerScanValidateResponse {
            validation_performed: true,
            validation_target_address,
            pruned_node_count,
            pruned_static_root_count,
            status_message: format!(
                "Validated pointer scan results {} against {}. Removed {} static roots. Heap nodes: {} -> {}.",
                request.session_id,
                summary.get_target_descriptor(),
                pruned_static_root_count,
                input.summary.get_total_heap_node_count(),
                summary.get_total_heap_node_count()
            ),
            pointer_scan_summary: Some(summary),
            ..Default::default()
        }
    })
}

/// Publish only if the primary graph that supplied the roots is still active.
fn update_current_results(
    state: &EnginePrivilegedState,
    identity: &Arc<()>,
    update: impl FnOnce(&mut PointerScanResults) -> PointerScanValidateResponse,
) -> PointerScanValidateResponse {
    let store = state.get_pointer_scan_results();
    let Ok(mut guard) = store.write() else {
        return PointerScanValidateResponse {
            status_message: String::from("Failed to access the active pointer scan results."),
            ..Default::default()
        };
    };
    match guard.as_mut() {
        Some(results) if Arc::ptr_eq(identity, &results.graph_identity()) => update(results),
        _ => PointerScanValidateResponse {
            status_message: String::from("The active pointer scan changed during validation."),
            ..Default::default()
        },
    }
}
