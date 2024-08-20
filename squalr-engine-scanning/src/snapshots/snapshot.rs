use crate::snapshots::snapshot_region::SnapshotRegion;

#[derive(Debug)]
pub struct Snapshot {
    snapshot_regions: Vec<SnapshotRegion>,
}

/// Represents a snapshot of memory in an external process that contains current and previous values of memory pages.
impl Snapshot {
    pub fn new(mut snapshot_regions: Vec<SnapshotRegion>) -> Self {
        // Remove empty regions and sort them ascending
        snapshot_regions.retain(|region| region.get_region_size() > 0);
        snapshot_regions.sort_by_key(|region| region.get_base_address());

        Self {
            snapshot_regions,
        }
    }

    pub fn get_snapshot_regions(&self) -> &Vec<SnapshotRegion> {
        return &self.snapshot_regions;
    }

    pub fn get_snapshot_regions_for_update(&mut self) -> &mut Vec<SnapshotRegion> {
        return &mut self.snapshot_regions;
    }

    pub fn get_region_count(&self) -> u64 {
        return self.snapshot_regions.len() as u64;
    }
    
    pub fn get_byte_count(&self) -> u64 {
        return self.snapshot_regions.iter().map(|region| region.get_region_size()).sum();
    }
}
