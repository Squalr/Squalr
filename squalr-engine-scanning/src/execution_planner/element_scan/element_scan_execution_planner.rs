use squalr_engine_api::registries::scan_rules::element_scan_rule_registry::ElementScanRuleRegistry;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::data_values::data_value::DataValue;
use squalr_engine_api::structures::scanning::scan_constraint::ScanConstraint;
use squalr_engine_api::structures::scanning::{
    filters::{snapshot_region_filter::SnapshotRegionFilter, snapshot_region_filter_collection::SnapshotRegionFilterCollection},
    parameters::{element_scan::element_scan_parameters::ElementScanParameters, mapped::mapped_scan_parameters::MappedScanParameters},
};
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use std::sync::{Arc, RwLock};

pub struct ElementScanExecutionPlanner {}

impl ElementScanExecutionPlanner {
    pub fn map_scan_constraint(
        element_scan_rule_registry: &Arc<RwLock<ElementScanRuleRegistry>>,
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        element_scan_parameters: &ElementScanParameters,
        scan_constraint: &ScanConstraint,
    ) -> Option<MappedScanParameters> {
        let symbol_registry_guard = match symbol_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                return None;
            }
        };
        let element_scan_rule_registry_guard = match element_scan_rule_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on ElementScanRuleRegistry: {}", error);
                return None;
            }
        };
        let data_type_ref = snapshot_region_filter_collection.get_data_type_ref();

        let data_value = match &scan_constraint.value {
            Some(anonymous_value) => symbol_registry_guard
                .deanonymize_value(data_type_ref, anonymous_value.get_value())
                .ok()?,
            None => DataValue::new(data_type_ref.clone(), Vec::new()),
        };

        drop(symbol_registry_guard);

        let mut mapped_scan_parameters = MappedScanParameters::new(
            data_value,
            element_scan_parameters.get_memory_alignment(),
            scan_constraint.compare_type,
            element_scan_parameters.get_floating_point_tolerance(),
        );

        // Apply all scan rules to the mapped parameters.
        for (_id, rule) in element_scan_rule_registry_guard.get_registry().iter() {
            rule.map_parameters(
                symbol_registry,
                snapshot_region,
                snapshot_region_filter_collection,
                snapshot_region_filter,
                element_scan_parameters,
                &mut mapped_scan_parameters,
            );
        }

        Some(mapped_scan_parameters)
    }
}
