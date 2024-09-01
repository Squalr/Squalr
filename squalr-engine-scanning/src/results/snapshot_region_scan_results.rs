use crate::results::scan_result::ScanResult;
use crate::results::scan_results_index_map::ScanResultsIndexMap;
use crate::results::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use squalr_engine_common::values::data_type::DataType;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::collections::HashMap;

/// A custom type that defines a set of filters (scan results) discovered by scanners.
/// While this looks silly, it is better to have a vector of vectors for parallelization.
/// This is because when we scan a filter, it produces a list of filters. Combining these back into
/// one giant list would cost too much scan time, so it's better to keep it as a list of lists.
pub type SnapshotFilterCollection = Vec<Vec<SnapshotRegionFilter>>;

#[derive(Debug, Default)]
pub struct SnapshotRegionScanResults {
    scan_result_lookup_tables: HashMap<DataType, ScanResultsIndexMap>,
    filters: HashMap<DataType, SnapshotFilterCollection>,
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
            scan_result_lookup_tables: HashMap::new(),
            filters: HashMap::new(),
        }
    }

    pub fn new_from_filters(filters: HashMap<DataType, SnapshotFilterCollection>) -> Self {
        Self {
            scan_result_lookup_tables: HashMap::new(),
            filters: filters,
        }
    }

    pub fn get_number_of_result_bytes(
        &self,
        data_type: &DataType,
    ) -> u64 {
        if let Some(scan_filter_map) = self.scan_result_lookup_tables.get(&data_type) {
            return scan_filter_map.get_number_of_result_bytes();
        }

        return 0;
    }

    pub fn get_scan_result(
        &self,
        index: u64,
        data_type: &DataType,
        memory_alignment: MemoryAlignment,
    ) -> Option<ScanResult> {
        // Select the set of results for the provided data type.
        if let Some(scan_filter_map) = self.scan_result_lookup_tables.get(&data_type) {
            // Select the list of filters for the given data type.
            if let Some(filter) = self.filters.get(&data_type) {
                // Get the index of the filter from the lookup table.
                if let Some((filter_range, snapshot_filter_index)) = scan_filter_map
                    .get_scan_result_range_map()
                    .get_key_value(&index)
                {
                    // Because our filters are a vector of vectors, we have to iterate to index into the filter we want.
                    let mut iter = filter
                        .into_iter()
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
            }
        }

        return None;
    }

    /// Creates the initial set of filters for the given set of scan filter parameters.
    /// At first, these filters are equal in size to the entire snapshot region.
    pub fn create_initial_scan_results(
        &mut self,
        base_address: u64,
        region_size: u64,
        scan_filter_parameters: &Vec<ScanFilterParameters>,
    ) {
        for scan_filter_parameter in scan_filter_parameters {
            self.filters
                .entry(scan_filter_parameter.get_data_type().clone())
                .or_insert_with(|| vec![vec![SnapshotRegionFilter::new(base_address, region_size)]]);
        }
    }

    /// Updates the set of filters over this snapshot region. Filters are essentially scan results for a given data type.
    pub fn set_all_filters(
        &mut self,
        filters: HashMap<DataType, SnapshotFilterCollection>,
    ) {
        self.filters = filters;
    }

    pub fn get_filters(&self) -> &HashMap<DataType, SnapshotFilterCollection> {
        return &self.filters;
    }

    pub fn build_scan_results(&mut self) {
        // Build the scan results for each data type being scanned.
        for (_, (data_type, filters)) in self.filters.iter().enumerate() {
            let mut scan_results_lookup_table = ScanResultsIndexMap::new();

            // Iterate every snapshot region contained by the snapshot.
            for (filter_index, filter) in filters.into_iter().flatten().enumerate() {
                let current_number_of_result_bytes = scan_results_lookup_table.get_number_of_result_bytes();
                let filter_size = filter.get_region_size();

                // Simply map the result range onto a the index of a particular snapshot region.
                scan_results_lookup_table.insert(current_number_of_result_bytes, filter_size, filter_index as u64);
            }

            self.scan_result_lookup_tables
                .insert(data_type.clone(), scan_results_lookup_table);
        }
    }

    /// Additionally, we resize this region to reduce wasted memory (ie data outside the min/max filter addresses).
    pub fn get_filter_bounds(
        &self,
        original_base_address: u64,
        original_end_address: u64,
    ) -> Option<(u64, u64)> {
        if self.filters.is_empty() {
            return None;
        }

        let mut new_base_address = u64::MAX;
        let mut new_end_address = 0u64;
        let mut found_valid_filter = false;

        // Iterate filter collections for all data types
        for filter_collection in self.filters.values() {
            // Flatten and check each filter to find the highest and lowest addresses.
            for filter in filter_collection.into_iter().flatten() {
                let filter_base = filter.get_base_address();
                let filter_end = filter.get_end_address();

                if filter_base < new_base_address {
                    new_base_address = filter_base;
                }

                if filter_end > new_end_address {
                    new_end_address = filter_end;
                }

                found_valid_filter = true;
            }
        }

        if !found_valid_filter {
            return None;
        }

        new_base_address = new_base_address.max(original_base_address);
        new_end_address = new_end_address.min(original_end_address);

        return Some((new_base_address, new_end_address));
    }
}
