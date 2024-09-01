use crate::results::scan_result::ScanResult;
use crate::results::scan_results_index_map::ScanResultsIndexMap;
use crate::results::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_common::values::data_type::DataType;
use std::collections::HashMap;

/// While this looks silly, it is better to have a vector of vectors for parallelization.
/// This is because when we scan a filter, it produces a list of filters. Combining these back into
/// one giant list would cost too much scan time, so it's better to keep it as a list of lists.
pub type SnapshotFilterCollection = Vec<Vec<SnapshotRegionFilter>>;

#[derive(Debug, Default)]
pub struct SnapshotRegionScanResults {
    scan_result_lookup_tables: HashMap<DataType, ScanResultsIndexMap>,
    filters: HashMap<DataType, SnapshotFilterCollection>,
}

/// Fundamentally, we need to be able to quickly navigate to a specific page number and offset of scan results within a snapshot region.
/// We need to avoid 'seeking' implementations that require repeatedly iterating over the entire scan, and for this we need to use interval trees.
///
/// There are two steps of obtaining a scan result.
/// 1) Map the scan result index to a particular snapshot region.
/// 2) Map a local index (details later) to a particular scan result address within this region.
///
/// Scan result collections are separated by data type for improved parallelism.
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

    pub fn get_scan_result(
        &self,
        index: u64,
        snapshot_regions: &Vec<SnapshotRegion>,
        data_type: &DataType,
    ) -> Option<ScanResult> {
        if let Some(scan_results_collection) = self.scan_result_lookup_tables.get(&data_type) {
            if let Some(snapshot_region_index) = scan_results_collection.get_scan_result_range_map().get(&index) {
                if *snapshot_region_index < snapshot_regions.len() as u64 {
                    let snapshot_region = &snapshot_regions[*snapshot_region_index as usize];

                    // snapshot_region.get_filters();
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

    pub fn get_filters(&self) -> &HashMap<DataType, SnapshotFilterCollection> {
        return &self.filters;
    }

    pub fn build_scan_results(
        &mut self,
        snapshot_regions: &Vec<SnapshotRegion>,
    ) {
        // Build the scan results for each data type being scanned.
        for (_, scan_filter_parameters) in self.scan_filter_parameters.iter().enumerate() {
            let data_type = scan_filter_parameters.get_data_type();

            let scan_results_lookup_table = ScanResultsIndexMap::new();

            // Iterate every snapshot region contained by the snapshot.
            for (region_index, region) in snapshot_regions.iter().enumerate() {
                if !region.get_filters().contains_key(data_type) {
                    continue;
                }

                let filter_regions = region.get_filters().get(data_type).unwrap();
                let number_of_filter_results = filter_regions.get_number_of_results();
                let current_number_of_results = scan_results_lookup_table.get_number_of_results();

                // Simply map the result range onto a the index of a particular snapshot region.
                scan_results_lookup_table.insert(current_number_of_results, number_of_filter_results, region_index as u64);
            }

            self.scan_result_lookup_tables
                .insert(*data_type, scan_results_lookup_table);
        }
    }
}
