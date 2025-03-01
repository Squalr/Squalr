use crate::snapshots::snapshot_region::SnapshotRegion;

pub struct Snapshot {
    snapshot_regions: Vec<SnapshotRegion>,
}

/// Represents a snapshot of memory in an external process that contains current and previous values of memory pages.
impl Snapshot {
    /// Creates a new snapshot from the given collection of snapshot regions.
    /// This will automatically sort and remove invalid regions.
    pub fn new() -> Self {
        Self { snapshot_regions: vec![] }
    }

    /// Assigns new snapshot regions to this snapshot.
    pub fn set_snapshot_regions(
        &mut self,
        snapshot_regions: Vec<SnapshotRegion>,
    ) {
        self.snapshot_regions = snapshot_regions;
        self.discard_empty_regions();
        self.sort_regions();
    }

    /// Gets a reference to the snapshot regions contained by this snapshot.
    pub fn get_snapshot_regions(&self) -> &Vec<SnapshotRegion> {
        &self.snapshot_regions
    }

    /// Gets a mutable reference to the snapshot regions contained by this snapshot.
    pub fn get_snapshot_regions_mut(&mut self) -> &mut Vec<SnapshotRegion> {
        &mut self.snapshot_regions
    }

    /// Discards all snapshot regions with a size of zero.
    pub fn discard_empty_regions(&mut self) {
        self.snapshot_regions
            .retain(|region| region.get_region_size() > 0);
    }

    /// Sorts all snapshot regions by base address ascending.
    pub fn sort_regions(&mut self) {
        self.snapshot_regions
            .sort_by_key(|region| region.get_base_address());
    }

    /// Gets the total number of snapshot regions contained in this snapshot.
    pub fn get_region_count(&self) -> u64 {
        self.snapshot_regions.len() as u64
    }

    /// Gets the total number of bytes contained in this snapshot.
    pub fn get_byte_count(&self) -> u64 {
        self.snapshot_regions
            .iter()
            .map(|region| region.get_region_size())
            .sum()
    }
}
