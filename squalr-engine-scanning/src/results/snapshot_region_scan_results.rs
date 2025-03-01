use crate::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
use crate::results::lookup_tables::snapshot_region_filter_lookup_table::SnapshotRegionFilterLookupTable;
use squalr_engine_common::structures::scan_result::ScanResult;

/// Tracks the scan results for a region, and builds a lookup table that maps a local index to each scan result.
/// This lookup table solves several problems efficiently:
/// 1) Support sharding on data type, to increase parallelism in scans.
/// 2) Support quickly navigating (without linear seeking or CPU heavy solutions!) to a specific scan result by local index.
/// 3) Interleave scan results by address for all data types, such that scan results appear sorted.
///
/// The solution is to use sorted ranges and modulo hashing the index to any overlapping filters.
/// Filters only are expected to overlap when two filters of differing data types both have a result in the same address.
/// For example, scanning for 0 across multiple data types could produce 1, 2, 4, and 8 byte integer matches on the same address.
pub struct SnapshotRegionScanResults {
    /// The collection of filters produced by a scan for a specific snapshot region.
    snapshot_region_filter_collections: Vec<SnapshotRegionFilterCollection>,

    /// A lookup table for fast and efficient querying of scan results.
    scan_results_local_lookup_table: SnapshotRegionFilterLookupTable,
}

impl SnapshotRegionScanResults {
    pub fn new(snapshot_region_filter_collections: Vec<SnapshotRegionFilterCollection>) -> Self {
        let mut instance: SnapshotRegionScanResults = Self {
            snapshot_region_filter_collections,
            scan_results_local_lookup_table: SnapshotRegionFilterLookupTable::new(),
        };

        instance.build_scan_results();

        instance
    }

    pub fn get_scan_result(
        &self,
        local_scan_index: u64,
    ) -> Option<ScanResult> {
        /*
        // Get the index of the filter from the lookup table.
        if let Some((filter_range, snapshot_filter_index)) = self
            .scan_results_local_lookup_table
            .get_lookup_mapping()
            .get_key_value(&index)
        {
            // Because our filters are a vector of vectors, we have to iterate to index into the filter we want.
            let mut iter = self
                .snapshot_region_filters
                .iter()
                .flatten()
                .skip(*snapshot_filter_index as usize);

            // Get the filter to which the scan result is mapped.
            if let Some(filter) = iter.next() {
                // The index passed to this so far has only helped us identify the filter. We can use the mapping range
                // to determine which element specifically we are trying to fetch.
                let element_index = filter_range.end() - index;
                let scan_result_address = filter.get_base_address() + element_index * memory_alignment as u64;

                return Some(scan_result_address);
            }
        }*/

        None
    }

    pub fn get_number_of_results(&self) -> u64 {
        self.scan_results_local_lookup_table.get_number_of_results()
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

    fn build_scan_results(&mut self) {
        for snapshot_region_filter_collection in &self.snapshot_region_filter_collections {
            let data_type = snapshot_region_filter_collection.get_data_type();
            let memory_alignment = snapshot_region_filter_collection.get_memory_alignment();

            for (filter_index, filter) in snapshot_region_filter_collection.iter().enumerate() {
                let filter_element_count = filter.get_element_count(&data_type, memory_alignment);
                self.scan_results_local_lookup_table
                    .append_lookup_mapping(filter_element_count, filter_index as u64);
            }
        }
    }
}
