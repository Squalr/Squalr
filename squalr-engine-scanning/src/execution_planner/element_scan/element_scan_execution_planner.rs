use squalr_engine_api::registries::scan_rules::element_scan_rule_registry::ElementScanRuleRegistry;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::data_values::data_value::DataValue;
use squalr_engine_api::structures::scanning::{
    filters::{snapshot_region_filter::SnapshotRegionFilter, snapshot_region_filter_collection::SnapshotRegionFilterCollection},
    parameters::{element_scan::element_scan_parameters::ElementScanParameters, mapped::mapped_scan_parameters::MappedScanParameters},
};
use std::sync::{Arc, RwLock};

pub struct ElementScanExecutionPlanner {}

impl ElementScanExecutionPlanner {
    pub fn map(
        element_scan_rule_registry: &Arc<RwLock<ElementScanRuleRegistry>>,
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
        snapshot_region_filter: &SnapshotRegionFilter,
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        element_scan_parameters: &ElementScanParameters,
    ) -> Vec<MappedScanParameters> {
        let symbol_registry_guard = match symbol_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on SymbolRegistry: {}", error);

                return Vec::new();
            }
        };
        /*
        pub fn is_valid_for_snapshot_region(
            &self,
            snapshot_region: &SnapshotRegion,
        ) -> bool {
            if snapshot_region.has_current_values() {
                match self.get_compare_type() {
                    ScanCompareType::Immediate(_) => return true,
                    ScanCompareType::Relative(_) | ScanCompareType::Delta(_) => snapshot_region.has_previous_values(),
                }
            } else {
                false
            }
        }

        pub fn is_valid_for_data_type(
            &self,
            data_type_ref: &DataTypeRef,
        ) -> bool {
            for data_value_and_alignment in &self.element_scan_values {
                if data_value_and_alignment.get_data_value().get_data_type_ref() == data_type_ref {
                    return true;
                }
            }

            false
        }
        */

        let mut mapped_scan_parameters_vec = Vec::new();
        let data_type_ref = snapshot_region_filter_collection.get_data_type_ref();

        // Given the data type for this specific filter collection, gather all constraints map them to new parameters to execute the scan.
        for scan_constraint in element_scan_parameters.get_scan_constraints() {
            let data_value = match &scan_constraint.value {
                Some(anonymous_value) => {
                    let data_value = match symbol_registry_guard.deanonymize_value(data_type_ref, anonymous_value.get_value()) {
                        Ok(data_value) => data_value,
                        Err(error) => {
                            log::error!("Error mapping data value: {}", error);
                            continue;
                        }
                    };

                    data_value
                }
                None => DataValue::new(data_type_ref.clone(), Vec::new()),
            };

            let mut mapped_scan_parameters = MappedScanParameters::new(
                data_value,
                element_scan_parameters.get_memory_alignment(),
                scan_constraint.compare_type,
                element_scan_parameters.get_floating_point_tolerance(),
            );
            let element_scan_rule_registry_guard = match element_scan_rule_registry.read() {
                Ok(registry) => registry,
                Err(error) => {
                    log::error!("Failed to acquire read lock on ElementScanRuleRegistry: {}", error);

                    mapped_scan_parameters_vec.push(mapped_scan_parameters);
                    continue;
                }
            };

            // Apply all scan rules to the mapped parameters.
            for (_id, rule) in element_scan_rule_registry_guard.get_registry().iter() {
                rule.map_parameters(
                    symbol_registry,
                    snapshot_region_filter_collection,
                    snapshot_region_filter,
                    element_scan_parameters,
                    &mut mapped_scan_parameters,
                );
            }

            mapped_scan_parameters_vec.push(mapped_scan_parameters);
        }

        mapped_scan_parameters_vec
    }
}
