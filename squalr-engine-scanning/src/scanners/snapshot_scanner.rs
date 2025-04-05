use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;

pub trait Scanner<ParameterType>: Send + Sync {
    fn scan_region(
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters: &ParameterType,
    ) -> Vec<SnapshotRegionFilter>;
}
