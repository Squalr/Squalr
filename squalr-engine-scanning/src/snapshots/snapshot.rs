use std::time::SystemTime;
use crate::snapshots::snapshot_region::SnapshotRegion;

#[derive(Clone)]
pub struct Snapshot {
    pub snapshot_name: String,
    pub region_count: usize,
    pub byte_count: u64,
    pub element_count: u64,
    pub creation_time: SystemTime,
    pub snapshot_regions: Vec<SnapshotRegion>,
}

impl Snapshot {
    pub fn new(snapshot_name: String, snapshot_regions: Vec<SnapshotRegion>) -> Self {
        Self {
            snapshot_name,
            region_count: snapshot_regions.len(),
            byte_count: 0, // Placeholder, will be calculated
            element_count: 0, // Placeholder, will be calculated
            creation_time: SystemTime::now(),
            snapshot_regions,
        }
    }
    
    pub fn set_snapshot_regions(&mut self, snapshot_regions: Vec<SnapshotRegion>) {
        self.snapshot_regions = snapshot_regions;
        self.region_count = self.snapshot_regions.len();
        self.creation_time = SystemTime::now();
    }
}
