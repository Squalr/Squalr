use std::sync::{Arc, RwLock};

use olorin_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use olorin_engine_api::registries::scan_rules::element_scan_rule_registry::ElementScanRuleRegistry;
use olorin_engine_api::structures::scanning::{
    filters::{snapshot_region_filter::SnapshotRegionFilter, snapshot_region_filter_collection::SnapshotRegionFilterCollection},
    parameters::{element_scan::element_scan_parameters::ElementScanParameters, mapped::mapped_scan_parameters::MappedScanParameters},
};

pub struct ElementScanExecutionPlanner {}

impl ElementScanExecutionPlanner {
    pub fn map(
        element_scan_rule_registry: &Arc<RwLock<ElementScanRuleRegistry>>,
        data_type_registry: &Arc<RwLock<SymbolRegistry>>,
        snapshot_region_filter: &SnapshotRegionFilter,
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        element_scan_parameters: &ElementScanParameters,
    ) -> MappedScanParameters {
        let mut mapped_scan_parameters = MappedScanParameters::new(snapshot_region_filter_collection, element_scan_parameters);
        let element_scan_rule_registry_guard = match element_scan_rule_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on ElementScanRuleRegistry: {}", error);

                return mapped_scan_parameters;
            }
        };

        for (_id, rule) in element_scan_rule_registry_guard.get_registry().iter() {
            rule.map_parameters(
                data_type_registry,
                snapshot_region_filter_collection,
                snapshot_region_filter,
                element_scan_parameters,
                &mut mapped_scan_parameters,
            );
        }

        mapped_scan_parameters
    }
}
