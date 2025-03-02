use crate::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
use std::{cmp::Reverse, collections::BinaryHeap};

/// Tracks the scan results for a region, and builds a lookup table that allows mapping a local index to a scan result.
/// This lookup table solves several problems efficiently:
/// 1) Support sharding on data type, to increase parallelism in scans.
/// 2) Support quickly navigating (without linear seeking or CPU heavy solutions) to a specific scan result by local index.
/// 3) Interleave scan results by address, such that scan results appear sorted across all data types.
///
/// For example, scanning for 0 across multiple data types could produce 1, 2, 4, and 8 byte integer matches on the same address.
/// The solution is TBD
pub struct SnapshotRegionScanResults {
    /// The collection of filters produced by a scan for a specific snapshot region.
    snapshot_region_filter_collections: Vec<SnapshotRegionFilterCollection>,
}

impl SnapshotRegionScanResults {
    pub fn new(snapshot_region_filter_collections: Vec<SnapshotRegionFilterCollection>) -> Self {
        Self {
            snapshot_region_filter_collections,
        }
    }

    pub fn get_scan_result_address(
        &self,
        mut scan_result_index: u64,
    ) -> Option<u64> {
        let mut heap: BinaryHeap<Reverse<(usize, usize)>> = BinaryHeap::new();

        // Each entry in heap is (address, collection_index, filter_index).
        let mut iterators: Vec<_> = self
            .snapshot_region_filter_collections
            .iter()
            .map(|collection| collection.iter().peekable())
            .collect();

        // Initialize heap with the first address from each iterator.
        for (collection_index, iterator) in iterators.iter_mut().enumerate() {
            if let Some(_) = iterator.peek() {
                heap.push(Reverse((collection_index, 0)));
            }
        }

        // TODO: Incomplete solution that processes 1 data type at a time. We need to zipper the results together by address.
        while let Some(Reverse((collection_index, filter_index))) = heap.pop() {
            let iterator = &mut iterators[collection_index];
            let filter = iterator.next().unwrap();
            let collection = &self.snapshot_region_filter_collections[collection_index];
            let result_count = filter.get_element_count(collection.get_data_type(), collection.get_memory_alignment());

            if scan_result_index < result_count {
                // The desired result is within this filter.
                return Some(filter.get_base_address() + scan_result_index * collection.get_memory_alignment() as u64);
            }

            // Decrease the index as we've skipped this entire filter's elements.
            scan_result_index -= result_count;

            // If the iterator still has filters, add the next one to the heap.
            if let Some(_) = iterator.peek() {
                heap.push(Reverse((collection_index, filter_index + 1)));
            }
        }

        // No result found at this index.
        None
    }

    /// Gets the number of results contained in this lookup table.
    pub fn get_number_of_results(&self) -> u64 {
        // Just sum the results for each collection. At most we would expect about 10 collections, so this is fine.
        self.snapshot_region_filter_collections
            .iter()
            .map(|collection| collection.get_number_of_results())
            .sum()
    }

    pub fn get_filter_collections(&self) -> &Vec<SnapshotRegionFilterCollection> {
        &self.snapshot_region_filter_collections
    }

    pub fn get_filter_bounds(&self) -> (u64, u64) {
        let mut filter_min_address = 0u64;
        let mut filter_max_address = 0u64;

        // Collect the minimum and maximum filter bounds. These are used to efficiently build our lookup table.
        for snapshot_region_filter_collection in &self.snapshot_region_filter_collections {
            filter_min_address = filter_min_address.min(snapshot_region_filter_collection.get_filter_minimum_address());
            filter_max_address = filter_max_address.min(snapshot_region_filter_collection.get_filter_minimum_address());
        }

        (filter_min_address, filter_max_address)
    }
}
