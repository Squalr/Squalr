use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::structures::scan_results::scan_result_base::ScanResultBase;

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

    /// Seeks to the
    pub fn get_scan_result(
        &self,
        scan_result_index: u64,
    ) -> Option<ScanResultBase> {
        let mut scan_result_index = scan_result_index;

        for snapshot_region in &self.snapshot_regions {
            let snapshot_region_scan_results = snapshot_region.get_scan_results();
            let number_of_region_results = snapshot_region_scan_results.get_number_of_results();

            if scan_result_index < number_of_region_results {
                return snapshot_region_scan_results.get_scan_result(snapshot_region, scan_result_index);
            }

            scan_result_index = scan_result_index.saturating_sub(number_of_region_results);
        }

        None
    }

    pub fn get_number_of_results(&self) -> u64 {
        self.snapshot_regions
            .iter()
            .map(|snapshot_region| snapshot_region.get_scan_results().get_number_of_results())
            .sum()
    }
}
