use crate::snapshots::snapshot_region::SnapshotRegion;

use std::time::SystemTime;

#[derive(Clone)]
pub struct Snapshot {
    pub snapshot_name: String,
    pub byte_count: u64,
    pub element_count: u64,
    pub creation_time: SystemTime,
    pub snapshot_regions: Vec<SnapshotRegion>,
}

impl Snapshot {
    pub fn new(snapshot_name: String, snapshot_regions: Vec<SnapshotRegion>) -> Self {
        Self {
            snapshot_name,
            byte_count: 0, // Placeholder, will be calculated
            element_count: 0, // Placeholder, will be calculated
            creation_time: SystemTime::now(),
            snapshot_regions,
        }
    }
    
    pub fn set_snapshot_regions(&mut self, snapshot_regions: Vec<SnapshotRegion>) {
        self.snapshot_regions = snapshot_regions;
        self.creation_time = SystemTime::now();
    }

    pub fn get_region_count(&self) -> usize {
        return self.snapshot_regions.len();
    }

    pub fn get_optimal_sorted_snapshot_regions(&self) -> impl Iterator<Item = &SnapshotRegion> {
        let mut sorted_regions: Vec<&SnapshotRegion> = self.snapshot_regions.iter().collect();
        sorted_regions.sort_by_key(|region| -(region.get_region_size() as i64));
        return sorted_regions.into_iter();
    }
}
