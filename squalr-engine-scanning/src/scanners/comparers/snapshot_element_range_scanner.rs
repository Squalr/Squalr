use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use std::sync::Arc;

pub trait SnapshotElementRangeScanner {
    fn scan_region(&mut self, element_range: Arc<SnapshotElementRange>, constraints: Arc<ScanConstraints>) -> Vec<Arc<SnapshotElementRange>>;
}
