use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
use crate::results::lookup_tables::scan_results_lookup_table::ScanResultsLookupTable;
use crate::snapshots::snapshot_region::SnapshotRegion;
use dashmap::DashMap;
use squalr_engine_common::structures::memory_alignment::MemoryAlignment;
use squalr_engine_common::structures::scan_result::ScanResult;
use squalr_engine_common::values::data_type::DataType;
use std::sync::Arc;

/// Tracks the scan results for a region, and builds a lookup table that maps a local index to each scan result.
/// This lookup table solves several problems efficiently:
/// 1) Support sharding on data type, to increase parallelism in scans.
/// 2) Support quickly navigating (without seeking or CPU heavy solutions!) to a specific scan result by local index.
/// 3) Interleave scan results by address for all data types, such that scan results appear sorted.
///
/// The solution is to use interval trees to map local scan result indicies onto the corresponding snapshot filter.
/// Additionally,
pub struct SnapshotRegionScanResults {
    scan_results_local_lookup_table: ScanResultsLookupTable,
    filters_by_data_type: Arc<DashMap<DataType, SnapshotRegionFilterCollection>>,
    snapshot_region_filter_collection: Vec<SnapshotRegionFilterCollection>,
}

impl SnapshotRegionScanResults {
    pub fn new(snapshot_region_filter_collection: Vec<SnapshotRegionFilterCollection>) -> Self {
        Self {
            scan_results_local_lookup_table: ScanResultsLookupTable::new(),
            filters_by_data_type: Arc::new(DashMap::new()),
            snapshot_region_filter_collection,
        }
    }

    pub fn get_scan_result(
        &self,
        index: u64,
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
        &self.snapshot_region_filter_collection
    }

    /*
    pub fn get_filter_bounds(&self) -> (u64, u64) {
        let mut filter_lowest_address = 0u64;
        let mut filter_highest_address = 0u64;
        let mut run_once = true;

        // Flatten and check each filter to find the highest and lowest addresses.
        for filter in self.snapshot_region_filters.iter().flatten() {
            let filter_base = filter.get_base_address();
            let filter_end = filter.get_end_address();

            if run_once {
                run_once = false;
                filter_lowest_address = filter_base;
                filter_highest_address = filter_end;
            } else {
                filter_lowest_address = filter_lowest_address.min(filter_base);
                filter_highest_address = filter_lowest_address.max(filter_end);
            }
        }

        (filter_lowest_address, filter_highest_address)
    }

    fn build_local_scan_results_lookup_table(
        &mut self,
        data_type: &DataType,
        memory_alignment: MemoryAlignment,
    ) {
        self.scan_results_local_lookup_table.clear();
        let data_type_size = data_type.get_size_in_bytes();

        // Iterate every snapshot region contained by the snapshot.
        for (filter_index, filter) in self.snapshot_region_filters.iter().flatten().enumerate() {
            let number_of_filter_results = filter.get_element_count(data_type_size, memory_alignment);

            // Simply map the result range onto a the index of a particular snapshot region.
            self.scan_results_local_lookup_table
                .add_lookup_mapping(number_of_filter_results, filter_index as u64);
        }
    }*/
}
