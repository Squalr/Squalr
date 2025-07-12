use crate::structures::scanning::{
    filters::{snapshot_region_filter::SnapshotRegionFilter, snapshot_region_filter_collection::SnapshotRegionFilterCollection},
    parameters::{element_scan::element_scan_parameters::ElementScanParameters, mapped::mapped_scan_parameters::MappedScanParameters},
};

pub trait ElementScanMappingRule {
    fn get_id(&self) -> &str;
    fn map_parameters(
        &self,
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        snapshot_region_filter: &SnapshotRegionFilter,
        element_scan_parameters: &ElementScanParameters,
        mapped_scan_parameters: &mut MappedScanParameters,
    );
}
