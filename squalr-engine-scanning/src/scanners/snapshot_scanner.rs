use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_common::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use squalr_engine_common::structures::scanning::scan_parameters_local::ScanParametersLocal;

pub trait Scanner: Send + Sync {
    fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Vec<SnapshotRegionFilter>;
}
