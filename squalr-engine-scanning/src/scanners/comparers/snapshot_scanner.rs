use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::snapshots::snapshot_region::SnapshotRegion;
use crate::snapshots::snapshot_sub_region::SnapshotSubRegion;

pub trait Scanner: Send + Sync {
    fn scan_region(&self,
        snapshot_region: &SnapshotRegion,
        snapshot_sub_region: &SnapshotSubRegion,
        constraint: &ScanConstraint
    ) -> Vec<SnapshotSubRegion>;
}
