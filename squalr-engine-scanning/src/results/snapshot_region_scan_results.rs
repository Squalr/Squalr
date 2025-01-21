use crate::results::scan_result::ScanResult;
use crate::results::scan_results_index_map::ScanResultsIndexMap;
use crate::results::snapshot_region_filter::SnapshotRegionFilter;
use squalr_engine_common::values::data_type::DataType;
use squalr_engine_memory::memory_alignment::MemoryAlignment;

/// A custom type that defines a set of filters (scan results) discovered by scanners.
/// While this looks silly, it is better to have a vector of vectors for parallelization.
/// This is because when we scan a filter, it produces a list of filters. Combining these back into
/// one giant list would cost too much scan time, so it's better to keep it as a list of lists.
pub type SnapshotFilterCollection = Vec<Vec<SnapshotRegionFilter>>;

pub struct SnapshotRegionScanResults {
    // These should be combined into a single data structure
    scan_result_lookup_table: ScanResultsIndexMap,
    filters: SnapshotFilterCollection,
    filter_lowest_address: u64,
    filter_highest_address: u64,
}

/// Scan results are stored by using interval trees to map into snapshot regions / snapshot filters.
///
/// This solves two problems:
/// 1) We need to be able to quickly navigate to a specific page number and offset of scan results within a snapshot region.
/// 2) We need to avoid 'seeking' implementations that require large CPU costs, as well as any data structure that has high storage requirements.
///
/// We need to use two layers of interval trees to obtain a scan result:
/// 1) An interval tree to map the scan result index to a particular snapshot region.
/// 2) Offset this index to map into a particular scan result within this region.
///
/// Additionally, there are separate sets of scan results for each data type, as this helps substantially with parallalism in scans.
impl SnapshotRegionScanResults {
    pub fn new() -> Self {
        Self {
            scan_result_lookup_table: ScanResultsIndexMap::new(),
            filters: vec![vec![]],
            filter_lowest_address: 0,
            filter_highest_address: 0,
        }
    }

    pub fn new_from_filters(
        filters: SnapshotFilterCollection,
        data_type: &DataType,
        memory_alignment: MemoryAlignment,
    ) -> Self {
        let mut instance = Self {
            scan_result_lookup_table: ScanResultsIndexMap::new(),
            filters,
            filter_lowest_address: 0,
            filter_highest_address: 0,
        };

        // Set the filter lowest/highest address based on the given filter collection
        instance.update_filter_bounds();
        instance.build_region_scan_results(data_type, memory_alignment);

        return instance;
    }

    pub fn get_number_of_results(&self) -> u64 {
        return self.scan_result_lookup_table.get_number_of_results();
    }

    pub fn get_scan_result(
        &self,
        index: u64,
        memory_alignment: MemoryAlignment,
    ) -> Option<ScanResult> {
        // Get the index of the filter from the lookup table.
        if let Some((filter_range, snapshot_filter_index)) = self
            .scan_result_lookup_table
            .get_scan_result_range_map()
            .get_key_value(&index)
        {
            // Because our filters are a vector of vectors, we have to iterate to index into the filter we want.
            let mut iter = self
                .filters
                .iter()
                .flatten()
                .skip(*snapshot_filter_index as usize);

            // Get the filter to which the scan result is mapped.
            if let Some(filter) = iter.next() {
                // The index passed to this so far has only helped us identify the filter. We can use the mapping range
                // to determine which element specifically we are trying to fetch.
                let element_index = filter_range.end() - index;
                let scan_result_address = filter.get_base_address() + element_index * memory_alignment as u64;
                let scan_result = ScanResult::new(scan_result_address);

                return Some(scan_result);
            }
        }

        return None;
    }

    pub fn get_filters(&self) -> &SnapshotFilterCollection {
        return &self.filters;
    }

    pub fn build_region_scan_results(
        &mut self,
        data_type: &DataType,
        memory_alignment: MemoryAlignment,
    ) {
        self.scan_result_lookup_table.clear();
        let data_type_size = data_type.get_size_in_bytes();

        // Iterate every snapshot region contained by the snapshot.
        for (filter_index, filter) in self.filters.iter().flatten().enumerate() {
            let current_number_of_results = self.scan_result_lookup_table.get_number_of_results();
            let number_of_filter_results = filter.get_element_count(data_type_size, memory_alignment);

            // Simply map the result range onto a the index of a particular snapshot region.
            self.scan_result_lookup_table
                .insert(current_number_of_results, number_of_filter_results, filter_index as u64);
        }
    }

    pub fn get_filter_bounds(&self) -> (u64, u64) {
        return (self.filter_lowest_address, self.filter_highest_address);
    }

    /// Calculates and stores the bounds of all filters contained by the snapshot region. This helps snapshot regions cull unused bytes.
    fn update_filter_bounds(&mut self) {
        self.filter_lowest_address = 0u64;
        self.filter_highest_address = 0u64;

        if self.filters.is_empty() {
            return;
        }

        let mut run_once = true;

        // Flatten and check each filter to find the highest and lowest addresses.
        for filter in self.filters.iter().flatten() {
            let filter_base = filter.get_base_address();
            let filter_end = filter.get_end_address();

            if run_once {
                run_once = false;
                self.filter_lowest_address = filter_base;
                self.filter_highest_address = filter_end;
            } else {
                self.filter_lowest_address = self.filter_lowest_address.min(filter_base);
                self.filter_highest_address = self.filter_lowest_address.max(filter_end);
            }
        }
    }
}
