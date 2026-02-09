use crate::command_executors::privileged_request_executor::PrivilegedCommandRequestExecutor;
use crate::command_executors::scan::scan_results_metadata_collector::collect_scan_results_metadata;
use crate::engine_privileged_state::EnginePrivilegedState;
use squalr_engine_api::commands::scan::element_scan::element_scan_request::ElementScanRequest;
use squalr_engine_api::commands::scan::element_scan::element_scan_response::ElementScanResponse;
use squalr_engine_api::events::scan_results::updated::scan_results_updated_event::ScanResultsUpdatedEvent;
use squalr_engine_api::registries::scan_rules::element_scan_rule_registry::ElementScanRuleRegistry;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_api::structures::scanning::constraints::scan_constraint_finalized::ScanConstraintFinalized;
use squalr_engine_api::structures::scanning::plans::element_scan::element_scan_plan::ElementScanPlan;
use squalr_engine_scanning::scan_settings_config::ScanSettingsConfig;
use squalr_engine_scanning::scanners::element_scan_executor_task::ElementScanExecutor;
use squalr_engine_scanning::scanners::scan_execution_context::ScanExecutionContext;
use std::sync::Arc;

impl PrivilegedCommandRequestExecutor for ElementScanRequest {
    type ResponseType = ElementScanResponse;

    fn execute(
        &self,
        engine_privileged_state: &Arc<EnginePrivilegedState>,
    ) -> <Self as PrivilegedCommandRequestExecutor>::ResponseType {
        if let Some(process_info) = engine_privileged_state
            .get_process_manager()
            .get_opened_process()
        {
            let snapshot = engine_privileged_state.get_snapshot();
            let alignment = ScanSettingsConfig::get_memory_alignment().unwrap_or(MemoryAlignment::Alignment1);
            let floating_point_tolerance = ScanSettingsConfig::get_floating_point_tolerance();
            let memory_read_mode = ScanSettingsConfig::get_memory_read_mode();
            let is_single_thread_scan = ScanSettingsConfig::get_is_single_threaded_scan();
            let debug_perform_validation_scan = ScanSettingsConfig::get_debug_perform_validation_scan();

            // Deanonymize all scan constraints against all data types.
            // For example, an immediate comparison of >= 23 could end up being a byte, float, etc.
            let scan_constraints_by_data_type = self
                .data_type_refs
                .iter()
                .map(|data_type_ref| {
                    // Deanonymize the initial anonymous scan constraints against the current data type.
                    let scan_constraints = self
                        .scan_constraints
                        .iter()
                        .filter_map(|anonymous_scan_constraint| anonymous_scan_constraint.deanonymize_constraint(data_type_ref, floating_point_tolerance))
                        .collect();

                    // Optimize the scan constraints by running them through each parameter rule sequentially.
                    let scan_constraints_finalized = ElementScanRuleRegistry::get_instance()
                        .get_scan_parameters_rule_registry()
                        .iter()
                        .fold(scan_constraints, |mut scan_constraint, (_id, scan_parameter_rule)| {
                            scan_parameter_rule.map_parameters(&mut scan_constraint);
                            scan_constraint
                        })
                        .into_iter()
                        .map(|scan_constraint| ScanConstraintFinalized::new(scan_constraint))
                        .collect();

                    (data_type_ref.clone(), scan_constraints_finalized)
                })
                .collect();

            let element_scan_plan = ElementScanPlan::new(
                scan_constraints_by_data_type,
                alignment,
                floating_point_tolerance,
                memory_read_mode,
                is_single_thread_scan,
                debug_perform_validation_scan,
            );
            let memory_read_provider = engine_privileged_state.get_os_providers().memory_read.clone();
            let scan_execution_context = ScanExecutionContext::new(
                None,
                None,
                Some(Arc::new(move |opened_process_info, address, values| {
                    memory_read_provider.read_bytes(opened_process_info, address, values)
                })),
            );
            ElementScanExecutor::execute_scan(process_info, snapshot, element_scan_plan, true, &scan_execution_context);
            engine_privileged_state.emit_event(ScanResultsUpdatedEvent { is_new_scan: false });

            ElementScanResponse {
                scan_results_metadata: collect_scan_results_metadata(engine_privileged_state),
            }
        } else {
            log::error!("No opened process");
            ElementScanResponse::default()
        }
    }
}
