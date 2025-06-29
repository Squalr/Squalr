use squalr_engine_api::structures::scanning::{
    filters::{snapshot_region_filter::SnapshotRegionFilter, snapshot_region_filter_collection::SnapshotRegionFilterCollection},
    parameters::{element_scan::element_scan_parameters::ElementScanParameters, mapped::mapped_scan_parameters::MappedScanParameters},
};

pub trait ScanParameterMappingRule {
    fn map_parameters(
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        snapshot_region_filter: &SnapshotRegionFilter,
        element_scan_parameters: &ElementScanParameters,
        mapped_scan_parameters: &mut MappedScanParameters,
    );
}
