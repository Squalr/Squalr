use olorin_engine_api::registries::scan_rules::element_scan_rule_registry::ElementScanRuleRegistry;
use olorin_engine_api::structures::scanning::{
    filters::{snapshot_region_filter::SnapshotRegionFilter, snapshot_region_filter_collection::SnapshotRegionFilterCollection},
    parameters::{element_scan::element_scan_parameters::ElementScanParameters, mapped::mapped_scan_parameters::MappedScanParameters},
};

pub struct ElementScanExecutionPlanner {}

impl ElementScanExecutionPlanner {
    pub fn map(
        snapshot_region_filter: &SnapshotRegionFilter,
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        element_scan_parameters: &ElementScanParameters,
    ) -> MappedScanParameters {
        let mut mapped_scan_parameters = MappedScanParameters::new(snapshot_region_filter_collection, element_scan_parameters);

        match ElementScanRuleRegistry::get_instance().get_registry().read() {
            Ok(element_scan_rule_registry) => {
                for (_id, rule) in element_scan_rule_registry.iter() {
                    rule.map_parameters(
                        snapshot_region_filter_collection,
                        snapshot_region_filter,
                        element_scan_parameters,
                        &mut mapped_scan_parameters,
                    );
                }
            }
            Err(error) => log::error!("Error acquiring element scan registry: {}", error),
        }

        mapped_scan_parameters
    }
}
