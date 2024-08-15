use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::time::SystemTime;
use std::sync::{Arc, RwLock};

pub struct Snapshot {
    name: String,
    creation_time: SystemTime,
    snapshot_regions: Vec<Arc<RwLock<SnapshotRegion>>>,
}

impl Snapshot {
    pub fn new(name: String, snapshot_regions: Vec<SnapshotRegion>) -> Self {
        Self {
            name,
            creation_time: SystemTime::now(),
            snapshot_regions: snapshot_regions.into_iter().map(|region| Arc::new(RwLock::new(region))).collect(), // Fixed
        }
    }

    pub fn get_name(&self) -> String {
        return self.name.clone();
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    
    /// Assigns snapshot regions, sorting them by base address ascending.
    pub fn set_snapshot_regions(&mut self, snapshot_regions: Vec<Arc<RwLock<SnapshotRegion>>>) {
        self.creation_time = SystemTime::now();
        self.snapshot_regions = snapshot_regions;
        self.sort_regions_by_address();
    }

    pub fn get_snapshot_regions(&self) -> Vec<Arc<RwLock<SnapshotRegion>>> {
        return self.snapshot_regions.clone(); // Fixed
    }

    pub fn get_region_count(&self) -> usize {
        return self.snapshot_regions.len();
    }

    /// Sorts the regions by region size descending. This significantly improves scan speeds by introducing a greedy algorithm.
    /// Large regions require more work, so by processing them first, it is easier to distribute the remaining workload across threads.
    pub fn sort_regions_for_scans(&mut self) {
        self.snapshot_regions.sort_by_key(|region| -(region.read().unwrap().get_region_size() as i64)); // Fixed
    }

    pub fn sort_regions_by_address(&mut self) {
        self.snapshot_regions.sort_by_key(|region| region.read().unwrap().get_base_address()); // Fixed
    }

    pub fn get_byte_count(&self) -> u64 {
        return self.snapshot_regions.iter().map(|sub_region| sub_region.read().unwrap().get_region_size()).sum();
    }

    pub fn get_element_count(&self, alignment: MemoryAlignment, data_type_size: usize) -> u64 {
        return self.snapshot_regions.iter().map(|sub_region| sub_region.read().unwrap().get_element_count(alignment, data_type_size)).sum();
    }
}
