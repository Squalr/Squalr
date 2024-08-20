use squalr_engine_common::dynamic_struct::data_type::DataType;

use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::snapshots::snapshot_region::SnapshotRegion;

pub trait Scanner: Send + Sync {
    fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        constraint: &ScanConstraint,
        data_type: &DataType,
    ) -> Vec<SnapshotRegionFilter>;
}
