use crate::results::scan_result::ScanResult;
use crate::results::scan_results_index_map::ScanResultsIndexMap;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_common::values::data_type::DataType;
use std::collections::HashMap;
use std::mem::take;

#[derive(Debug)]
pub struct SnapshotScanResults {
    scan_result_lookup_tables: HashMap<DataType, ScanResultsIndexMap>,
    scan_filter_parameters: Vec<ScanFilterParameters>,
}

/// Fundamentally, we need to be able to quickly navigate to a specific page number and offset of scan results within a snapshot region.
/// We need to avoid 'seeking' implementations that require repeatedly iterating over the entire scan, and for this we need to use interval trees.
///
/// There are two steps of obtaining a scan result.
/// 1) Map the scan result index to a particular snapshot region.
/// 2) Map a local index (details later) to a particular scan result address within this region.
///
/// Scan result collections are separated by data type for improved parallelism.
impl SnapshotScanResults {
    pub fn new() -> Self {
        Self {
            scan_result_lookup_tables: HashMap::new(),
            scan_filter_parameters: vec![],
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

                    snapshot_region.get_filters();
                }
            }
        }

        return None;
    }

    pub fn set_scan_filter_parameters(
        &mut self,
        scan_filter_parameters: Vec<ScanFilterParameters>,
    ) {
        self.scan_filter_parameters = scan_filter_parameters;
    }

    pub fn get_scan_parameters_filters(&self) -> &Vec<ScanFilterParameters> {
        return &self.scan_filter_parameters;
    }

    pub fn take_scan_parameters_filters(&mut self) -> Vec<ScanFilterParameters> {
        return take(&mut self.scan_filter_parameters);
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
