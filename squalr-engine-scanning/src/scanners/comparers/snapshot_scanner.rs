use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::snapshots::snapshot_sub_region::SnapshotSubRegion;
use std::sync::{Arc, RwLock};

pub trait Scanner: Send + Sync {
    fn scan_region(&self, snapshot_sub_region: &Arc<RwLock<SnapshotSubRegion>>, constraints: Arc<ScanConstraint>) -> Vec<Arc<RwLock<SnapshotSubRegion>>>;
}
