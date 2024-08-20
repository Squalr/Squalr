use crate::filters::snapshot_filter::SnapshotFilter;
use crate::snapshots::snapshot::Snapshot;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_memory::memory_queryer::memory_queryer::{MemoryQueryer, PageRetrievalMode};
use squalr_engine_processes::process_info::ProcessInfo;
use std::sync::{Arc, RwLock};

pub struct ScanResults {
    /// The snapshot of memory in another process, which contains all captured regions and their memory values.
    snapshot: Option<Arc<RwLock<Snapshot>>>,
}

impl ScanResults {
    pub fn new() ->
    Self {
        Self {
            snapshot: None,
        }
    }

    pub fn clear_snapshot(&mut self) {
        self.snapshot = None;
    }

    pub fn get_snapshot(&self) -> Option<Arc<RwLock<Snapshot>>> {
        if let Some(snapshot) = &self.snapshot {
            return Some(snapshot.clone());
        }

        return None;
    }

    pub fn get_or_create_snapshot(&mut self, name: String, process_info: &ProcessInfo) -> Arc<RwLock<Snapshot>> {
        if let Some(snapshot) = &self.snapshot {
            return snapshot.clone();
        }

        let snapshot = Arc::new(RwLock::new(Snapshot::new(name, self.get_memory_pages_for_scan(process_info))));
        self.snapshot = Some(snapshot.clone());

        return snapshot;
    }

    pub fn get_memory_pages_for_scan(&mut self, process_info: &ProcessInfo) -> Vec<SnapshotRegion> {
        let memory_pages = MemoryQueryer::get_memory_page_bounds(process_info, PageRetrievalMode::FROM_SETTINGS);
        let snapshot_regions = memory_pages.into_iter().map(|region| SnapshotRegion::new(region)).collect();

        return snapshot_regions;
    }
}
