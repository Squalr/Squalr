use crate::filters::snapshot_filter::SnapshotFilter;
use crate::snapshots::snapshot::Snapshot;
use squalr_engine_processes::process_info::ProcessInfo;
use std::sync::{Arc, RwLock};

pub struct ScanResults {
    /// The snapshot of memory in another process, which contains all captured regions and their memory values.
    snapshot: Option<Arc<RwLock<Snapshot>>>,
    /// The filters over this snapshot produced by a scan. Generally, there will only be one filter containing the scan results.
    /// However, for advanced scans like "All data types", this will result in many indepdent filters over the snapshot.
    snapshot_filters: Vec<SnapshotFilter>
}

impl ScanResults {
    pub fn new() ->
    Self {
        Self {
            snapshot: None,
            snapshot_filters: vec![],
        }
    }

    pub fn get_snapshot_create_if_none(&mut self, process_info: &ProcessInfo) -> Arc<RwLock<Snapshot>> {
        if let Some(snapshot) = &self.snapshot {
            return snapshot.clone();
        }

        let snapshot = Arc::new(RwLock::new(SnapshotQueryer::get_snapshot(process_info, PageRetrievalMode::FROM_SETTINGS)));
        self.snapshot = Some(snapshot.clone());
        self.create_default_filter();

        return snapshot;
    }

    pub fn create_default_filter(&mut self) {
        let default_filter = SnapshotFilter::new(self.snapshot.as_mut().unwrap().clone());
        self.snapshot_filters = vec![default_filter];
    }
}
