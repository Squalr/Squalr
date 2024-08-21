use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::scanners::constraints::scan_filter_constraint::ScanFilterConstraint;
use crate::snapshots::snapshot_region::SnapshotRegion;

pub trait Scanner: Send + Sync {
    fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        constraint: &ScanConstraint,
        filter_constraint: &ScanFilterConstraint,
    ) -> Vec<SnapshotRegionFilter>;
}
