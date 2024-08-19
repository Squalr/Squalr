use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::time::SystemTime;

pub struct Snapshot {
    name: String,
    creation_time: SystemTime,
    snapshot_regions: Vec<SnapshotRegion>,
}

/// Represents a snapshot of memory in another process. By design, a snapshot is entirely immutable to avoid resource contention.
impl Snapshot {
    pub fn new(name: String, mut snapshot_regions: Vec<SnapshotRegion>) -> Self {
        // Remove empty regions and sort them ascending
        snapshot_regions.retain(|region| region.get_byte_count() > 0);
        snapshot_regions.sort_by_key(|region| region.get_base_address());

        Self {
            name,
            creation_time: SystemTime::now(),
            snapshot_regions,
        }
    }

    // TODO: Breaks immutability contract but not important, still...
    pub fn set_name(&mut self, name: String) {
        self.name = name;
    }

    pub fn get_name(&self) -> String {
        return self.name.clone();
    }
    pub fn get_creation_time(&self) -> &SystemTime {
        return &self.creation_time;
    }

    pub fn get_snapshot_regions(&self) -> &Vec<SnapshotRegion> {
        return &self.snapshot_regions;
    }

    pub fn get_region_count(&self) -> u64 {
        return self.snapshot_regions.len() as u64;
    }

    pub fn get_byte_count(&self) -> u64 {
        return self.snapshot_regions.iter().map(|region| region.get_byte_count()).sum();
    }

    pub fn get_element_count(&self, alignment: MemoryAlignment, data_type_size: u64) -> u64 {
        return self.snapshot_regions.iter().map(|region| region.get_element_count(alignment, data_type_size)).sum();
    }
}
