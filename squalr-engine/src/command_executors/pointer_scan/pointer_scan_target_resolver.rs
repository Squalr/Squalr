use squalr_engine_api::registries::scan_rules::element_scan_rule_registry::ElementScanRuleRegistry;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::anonymous_value_string::AnonymousValueString;
use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::pointer_scans::pointer_scan_target_descriptor::PointerScanTargetDescriptor;
use squalr_engine_api::structures::pointer_scans::pointer_scan_target_request::PointerScanTargetRequest;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use squalr_engine_api::structures::scanning::constraints::anonymous_scan_constraint::AnonymousScanConstraint;
use squalr_engine_api::structures::scanning::constraints::scan_constraint_finalized::ScanConstraintFinalized;
use squalr_engine_api::structures::scanning::memory_read_mode::MemoryReadMode;
use squalr_engine_api::structures::scanning::plans::element_scan::element_scan_plan::ElementScanPlan;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_scanning::scanners::element_scan_executor_task::ElementScanExecutor;
use squalr_engine_scanning::scanners::scan_execution_context::ScanExecutionContext;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub struct ResolvedPointerScanTargets {
    pub target_descriptor: PointerScanTargetDescriptor,
    pub target_addresses: Vec<u64>,
}

pub struct PointerScanTargetResolver;

impl PointerScanTargetResolver {
    pub fn resolve_targets(
        target_request: &PointerScanTargetRequest,
        address_pointer_size: PointerScanPointerSize,
        snapshot: Arc<RwLock<Snapshot>>,
        process_info: OpenedProcessInfo,
        memory_alignment: MemoryAlignment,
        floating_point_tolerance: squalr_engine_api::structures::data_types::floating_point_tolerance::FloatingPointTolerance,
        is_single_thread_scan: bool,
        debug_perform_validation_scan: bool,
        scan_execution_context: &ScanExecutionContext,
    ) -> Result<ResolvedPointerScanTargets, String> {
        match (
            target_request.target_address.as_ref(),
            target_request.target_value.as_ref(),
            target_request.target_data_type_ref.as_ref(),
        ) {
            (Some(target_address), None, None) => Self::resolve_address_target(target_address, address_pointer_size),
            (None, Some(target_value), Some(target_data_type_ref)) => Self::resolve_value_target(
                target_value,
                target_data_type_ref,
                snapshot,
                process_info,
                memory_alignment,
                floating_point_tolerance,
                is_single_thread_scan,
                debug_perform_validation_scan,
                scan_execution_context,
            ),
            (None, None, None) => Err("Pointer scan target is missing.".to_string()),
            (Some(_target_address), Some(_target_value), _target_data_type_ref) => {
                Err("Pointer scan target cannot specify both an address and a value.".to_string())
            }
            (None, None, Some(_target_data_type_ref)) => Err("Pointer scan target data type requires a value.".to_string()),
            (None, Some(_target_value), None) => Err("Pointer scan value target requires a data type.".to_string()),
            (Some(_target_address), None, Some(_target_data_type_ref)) => Err("Pointer scan address targets cannot also specify a data type.".to_string()),
        }
    }

    fn resolve_address_target(
        target_address: &AnonymousValueString,
        pointer_size: PointerScanPointerSize,
    ) -> Result<ResolvedPointerScanTargets, String> {
        let symbol_registry = SymbolRegistry::get_instance();
        let target_address_data_type_ref = pointer_size.to_data_type_ref();
        let target_address_data_value = symbol_registry
            .deanonymize_value_string(&target_address_data_type_ref, target_address)
            .map_err(|error| format!("Failed to parse pointer scan target address: {}", error))?;
        let resolved_target_address = pointer_size
            .read_address_value(&target_address_data_value)
            .ok_or_else(|| format!("Failed to decode pointer scan target address using {}.", pointer_size))?;

        Ok(ResolvedPointerScanTargets {
            target_descriptor: PointerScanTargetDescriptor::address(resolved_target_address),
            target_addresses: vec![resolved_target_address],
        })
    }

    fn resolve_value_target(
        target_value: &AnonymousValueString,
        target_data_type_ref: &DataTypeRef,
        snapshot: Arc<RwLock<Snapshot>>,
        process_info: OpenedProcessInfo,
        memory_alignment: MemoryAlignment,
        floating_point_tolerance: squalr_engine_api::structures::data_types::floating_point_tolerance::FloatingPointTolerance,
        is_single_thread_scan: bool,
        debug_perform_validation_scan: bool,
        scan_execution_context: &ScanExecutionContext,
    ) -> Result<ResolvedPointerScanTargets, String> {
        let exact_scan_constraint = AnonymousScanConstraint::new(ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal), Some(target_value.clone()));
        let scan_constraint = exact_scan_constraint
            .deanonymize_constraint(target_data_type_ref, floating_point_tolerance)
            .ok_or_else(|| "Failed to parse pointer scan value target.".to_string())?;
        let finalized_scan_constraints = ElementScanRuleRegistry::get_instance()
            .get_scan_parameters_rule_registry()
            .iter()
            .fold(vec![scan_constraint], |mut scan_constraints, (_rule_id, scan_parameter_rule)| {
                scan_parameter_rule.map_parameters(&mut scan_constraints);
                scan_constraints
            })
            .into_iter()
            .map(ScanConstraintFinalized::new)
            .collect::<Vec<_>>();
        let element_scan_plan = ElementScanPlan::new(
            HashMap::from([(target_data_type_ref.clone(), finalized_scan_constraints)]),
            memory_alignment,
            floating_point_tolerance,
            MemoryReadMode::Skip,
            is_single_thread_scan,
            debug_perform_validation_scan,
        );
        let temporary_value_scan_snapshot = Arc::new(RwLock::new(Self::clone_snapshot_for_value_target_scan(snapshot.as_ref())?));

        ElementScanExecutor::execute_scan(
            process_info,
            temporary_value_scan_snapshot.clone(),
            element_scan_plan,
            true,
            scan_execution_context,
        );
        let snapshot_guard = temporary_value_scan_snapshot
            .read()
            .map_err(|error| format!("Failed to access pointer scan value target snapshot: {}", error))?;
        let result_count = snapshot_guard.get_number_of_results();
        let (_page_index, scan_results_page) = snapshot_guard.get_scan_results_page(None, 0, result_count.max(1));
        let mut target_addresses = scan_results_page
            .into_iter()
            .map(|scan_result| scan_result.get_address())
            .collect::<Vec<_>>();
        target_addresses.sort_unstable();
        target_addresses.dedup();

        Ok(ResolvedPointerScanTargets {
            target_descriptor: PointerScanTargetDescriptor::value(target_value.clone(), target_data_type_ref.clone(), target_addresses.len() as u64),
            target_addresses,
        })
    }

    fn clone_snapshot_for_value_target_scan(snapshot: &RwLock<Snapshot>) -> Result<Snapshot, String> {
        let snapshot_guard = snapshot
            .read()
            .map_err(|error| format!("Failed to access pointer scan snapshot for value target resolution: {}", error))?;
        let mut cloned_snapshot = Snapshot::new();
        let mut cloned_snapshot_regions = Vec::with_capacity(snapshot_guard.get_snapshot_regions().len());

        for snapshot_region in snapshot_guard.get_snapshot_regions() {
            let mut cloned_snapshot_region = SnapshotRegion::new(
                NormalizedRegion::new(snapshot_region.get_base_address(), snapshot_region.get_region_size()),
                snapshot_region.page_boundaries.clone(),
            );

            cloned_snapshot_region.current_values = snapshot_region.get_current_values().clone();
            cloned_snapshot_region.previous_values = snapshot_region.get_previous_values().clone();
            cloned_snapshot_region.page_boundary_tombstones = snapshot_region.page_boundary_tombstones.clone();
            cloned_snapshot_regions.push(cloned_snapshot_region);
        }

        cloned_snapshot.set_snapshot_regions(cloned_snapshot_regions);

        Ok(cloned_snapshot)
    }
}
