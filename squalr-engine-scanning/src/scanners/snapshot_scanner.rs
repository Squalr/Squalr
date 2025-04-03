use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::structures::scanning::parameters::scan_parameters::ScanParameters;

pub trait Scanner: Send + Sync {
    fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters: &ScanParameters,
    ) -> Vec<SnapshotRegionFilter>;
}
