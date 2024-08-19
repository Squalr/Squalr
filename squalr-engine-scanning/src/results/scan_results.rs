use crate::filters::snapshot_filter::SnapshotFilter;
use crate::snapshots::snapshot::Snapshot;
use squalr_engine_memory::memory_queryer::memory_queryer::{MemoryQueryer, PageRetrievalMode};
use squalr_engine_memory::normalized_region::NormalizedRegion;
use squalr_engine_processes::process_info::ProcessInfo;
use std::sync::{Arc, RwLock};

pub struct ScanResults {
    /// The snapshot of memory in another process, which contains all captured regions and their memory values.
    snapshot: Option<Arc<RwLock<Snapshot>>>,
    /// The filters over this snapshot produced by a scan. Generally, there will only be one filter containing the scan results.
    /// However, for advanced scans like "All data types", this will result in many indepdent filters over the snapshot.
    snapshot_filters: Option<Vec<SnapshotFilter>>
}

impl ScanResults {
    pub fn new() ->
    Self {
        Self {
            snapshot: None,
            snapshot_filters: None,
        }
    }

    pub fn set_snapshot(&mut self, snapshot: Arc<RwLock<Snapshot>>) {
        self.snapshot = Some(snapshot);

        if self.snapshot_filters.is_none() {
            self.create_default_filter();
        }
    }

    pub fn get_memory_pages_for_scan(&mut self, process_info: &ProcessInfo) -> Vec<NormalizedRegion> {
        // if let Some(snapshot) = &self.snapshot {
            // TODO: Return regions based on the existing filter bounds
            // return snapshot.write().unwrap().get_snapshot_regions();
        // }

        let memory_pages = MemoryQueryer::get_memory_page_bounds(process_info, PageRetrievalMode::FROM_SETTINGS);

        return memory_pages; // memory_pages.into_iter().map(|region| SnapshotRegion::new_from_normalized_region(region)).collect();
    }

    pub fn create_default_filter(&mut self) {
        let default_filter = SnapshotFilter::new(self.snapshot.as_mut().unwrap().clone());
        self.snapshot_filters = Some(vec![default_filter]);
    }

    /*
    pub fn get_element_count(&self, alignment: MemoryAlignment, data_type_size: u64) -> u64 {
        return self.snapshot_regions.iter().map(|region| region.get_element_count(alignment, data_type_size)).sum();
    } */
}
