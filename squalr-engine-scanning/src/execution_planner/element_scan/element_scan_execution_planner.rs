use crate::execution_planner::element_scan::element_scan_execution_rule::ScanParameterMappingRule;
use squalr_engine_api::structures::scanning::{
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
        // let rules: Vec<dyn ScanParameterMappingRule> = vec![];

        // JIRA: Fixme
        /*
        for rule in rules {
            rule.map_parameters(
                snapshot_region_filter_collection,
                snapshot_region_filter,
                element_scan_parameters,
                mapped_scan_parameters,
            );
        }*/

        mapped_scan_parameters
    }
}
