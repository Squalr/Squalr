use crate::registries::data_types::data_type_registry::DataTypeRegistry;
use crate::structures::scan_results::scan_result_ref::ScanResultRef;
use crate::structures::snapshots::snapshot_region::SnapshotRegion;
use crate::structures::{
    data_types::data_type_ref::DataTypeRef, scan_results::scan_result_valued::ScanResultValued,
    scanning::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection,
};
use std::sync::{Arc, RwLock};
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

    /// Gets the scan results (as a snapshot region filter collection) corresponding to the provided data type.
    pub fn get_scan_results_by_data_type(
        &self,
        data_type: &DataTypeRef,
    ) -> Option<&SnapshotRegionFilterCollection> {
        for collection in &self.snapshot_region_filter_collections {
            if collection.get_data_type_ref() == data_type {
                return Some(&collection);
            }
        }

        None
    }

    /// Performs a binary search to find the specified scan result by index.
    pub fn get_scan_result(
        &self,
        data_type_registry: &Arc<RwLock<DataTypeRegistry>>,
        snapshot_region: &SnapshotRegion,
        mut scan_result_index: u64,
    ) -> Option<ScanResultValued> {
        let data_type_registry_guard = match data_type_registry.read() {
            Ok(registry) => registry,
            Err(error) => {
                log::error!("Failed to acquire read lock on DataTypeRegistry: {}", error);

                return None;
            }
        };
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

        // JIRA: Incomplete solution that processes 1 data type at a time. We need to zipper the results together by address.
        // Edit: Actually, we want to track scan results for each data type separate, and show these as tabs or something in the GUI.
        while let Some(Reverse((collection_index, filter_index))) = heap.pop() {
            let iterator = &mut iterators[collection_index];
            let filter = iterator.next().unwrap(); // JIRA: Should be always safe, although I'd prefer to eliminate this.
            let collection = &self.snapshot_region_filter_collections[collection_index];
            let memory_alignment = collection.get_memory_alignment();
            let data_type_ref = collection.get_data_type_ref();
            let result_count = filter.get_element_count(data_type_registry, data_type_ref, memory_alignment);

            if scan_result_index < result_count {
                // The desired result is within this filter.
                let scan_result_address = filter
                    .get_base_address()
                    .saturating_add(scan_result_index * memory_alignment as u64);
                let current_value = snapshot_region.get_current_value(data_type_registry, scan_result_address, data_type_ref);
                let previous_value = snapshot_region.get_previous_value(data_type_registry, scan_result_address, data_type_ref);
                let current_display_values = if let Some(data_value) = current_value.as_ref() {
                    Some(data_type_registry_guard.create_display_values(data_type_ref, data_value.get_value_bytes()))
                } else {
                    None
                };
                let previous_display_values = if let Some(data_value) = previous_value.as_ref() {
                    Some(data_type_registry_guard.create_display_values(data_type_ref, data_value.get_value_bytes()))
                } else {
                    None
                };
                let icon_id = data_type_registry_guard.get_icon_id(data_type_ref);

                return Some(ScanResultValued::new(
                    scan_result_address,
                    data_type_ref.clone(),
                    icon_id,
                    current_value,
                    current_display_values,
                    previous_value,
                    previous_display_values,
                    ScanResultRef::new(scan_result_index),
                ));
            }

            // Decrease the index as we've skipped this entire filter's elements.
            scan_result_index = scan_result_index.saturating_sub(result_count);

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
        // Just sum the results for each collection. At most we would expect about ~10 collections, so this is fine.
        self.snapshot_region_filter_collections
            .iter()
            .map(|collection| collection.get_number_of_results())
            .sum()
    }

    /// Gets the collections of snapshot filters contained by this snapshot region. Generally one collection per data type scanned.
    pub fn get_filter_collections(&self) -> &Vec<SnapshotRegionFilterCollection> {
        &self.snapshot_region_filter_collections
    }

    /// Gets the minimum and maximum bounds across every filter contained by this snapshot region.
    pub fn get_filter_bounds(&self) -> (u64, u64) {
        let mut filter_min_address = u64::MAX;
        let mut filter_max_address = 0u64;

        // Collect the minimum and maximum filter bounds. These are used to efficiently build our lookup table.
        for snapshot_region_filter_collection in &self.snapshot_region_filter_collections {
            filter_min_address = filter_min_address.min(snapshot_region_filter_collection.get_filter_minimum_address());
            filter_max_address = filter_max_address.max(snapshot_region_filter_collection.get_filter_maximum_address());
        }

        // In the case where there are no filters (or something gone horribly wrong), correct the min to be <= max.
        filter_min_address = filter_min_address.clamp(0u64, filter_max_address);

        (filter_min_address, filter_max_address)
    }
}
