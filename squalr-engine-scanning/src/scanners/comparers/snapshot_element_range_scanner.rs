use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use std::sync::{Arc, RwLock};

pub trait SnapshotElementRangeScanner: Send + Sync {
    fn scan_region(&mut self, element_range: &Arc<RwLock<SnapshotElementRange>>, constraints: Arc<ScanConstraint>) -> Vec<Arc<RwLock<SnapshotElementRange>>>;
}
