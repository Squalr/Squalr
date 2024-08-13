use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::time::SystemTime;

pub struct Snapshot {
    name: String,
    byte_count: u64,
    element_count: u64,
    creation_time: SystemTime,
    pub snapshot_regions: Vec<SnapshotRegion>,
}

impl Snapshot {
    pub fn new(name: String, snapshot_regions: Vec<SnapshotRegion>) -> Self {
        Self {
            name,
            byte_count: 0, // Placeholder, will be calculated
            element_count: 0, // Placeholder, will be calculated
            creation_time: SystemTime::now(),
            snapshot_regions,
        }
    }

    pub fn get_name(&self) -> String {
        return self.name.clone();
    }

    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }
    
    /// Assigns snapshot regions, sorting them by base address ascending.
    pub fn set_snapshot_regions(&mut self, snapshot_regions: Vec<SnapshotRegion>, alignment: MemoryAlignment, data_type_size: usize) {
        self.creation_time = SystemTime::now();
        self.snapshot_regions = snapshot_regions;
        self.sort_regions_by_address();

        // Calculate the number of elements / total bytes based on the alignment and data type size
    }

    pub fn get_region_count(&self) -> usize {
        return self.snapshot_regions.len();
    }

    /// Sorts the regions by region size descending. This significantly improves scan speeds by introducing a greedy algorithm.
    /// Large regions require more work, so by processing them first, it is easier to distribute the remaining workload across threads.
    pub fn sort_regions_for_scans(&mut self) {
        self.snapshot_regions.sort_by_key(|region| -(region.get_region_size() as i64));
    }

    pub fn sort_regions_by_address(&mut self) {
        self.snapshot_regions.sort_by_key(|region| region.get_base_address());
    }

    pub fn get_byte_count(&self) -> u64 {
        return self.byte_count;
    }

    pub fn get_element_count(&self) -> u64 {
        return self.element_count;
    }
}
