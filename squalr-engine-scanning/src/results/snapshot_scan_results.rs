use crate::results::snapshot_region_scan_results::SnapshotRegionScanResults;

/// Allows direct access of scan results for a given data type and alignment. Through the use of
/// sorted ranges and binary search lookups, efficient and low-footprint lookups of scan results are possible.
pub struct SnapshotScanResults {
    /// The scan results for each snapshot region in the snapshot.
    snapshot_region_scan_results_collection: Vec<SnapshotRegionScanResults>,
}

/// Fundamentally, we need to be able to quickly navigate to a specific page number and offset of scan results within a snapshot region.
/// We need to avoid 'seeking' implementations that require repeatedly iterating over the entire scan, and for this we use sorted ranges.
///
/// There are two steps to building these sorted ranges:
/// 1) For each snapshot region, map a local index to a particular scan result address. This can be parallelized and performed during scans.
/// 2) Map the scan result index (global index) to a particular snapshot region.
///
/// Scan result collections are separated by data type for improved parallelism.
impl SnapshotScanResults {
    pub fn new(snapshot_region_scan_results_collection: Vec<SnapshotRegionScanResults>) -> Self {
        Self {
            snapshot_region_scan_results_collection,
        }
    }

    pub fn get_scan_result_address(
        &self,
        scan_result_index: u64,
    ) -> Option<u64> {
        let mut scan_result_index = scan_result_index;

        for snapshot_region_scan_results in &self.snapshot_region_scan_results_collection {
            let number_of_region_results = snapshot_region_scan_results.get_number_of_results();

            if scan_result_index < number_of_region_results {
                return snapshot_region_scan_results.get_scan_result_address(scan_result_index);
            }

            scan_result_index = scan_result_index.saturating_sub(number_of_region_results);
        }

        None
    }

    pub fn get_number_of_results(&self) -> u64 {
        self.snapshot_region_scan_results_collection
            .iter()
            .map(|snapshot_region_scan_results| snapshot_region_scan_results.get_number_of_results())
            .sum()
    }
}
