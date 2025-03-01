use crate::results::lookup_tables::scan_results_lookup_table::ScanResultsLookupTable;
use crate::snapshots::snapshot_region::SnapshotRegion;
use dashmap::DashMap;
use squalr_engine_common::structures::memory_alignment::MemoryAlignment;
use squalr_engine_common::structures::process_info::OpenedProcessInfo;
use squalr_engine_common::structures::scan_filter_parameters::ScanFilterParameters;
use squalr_engine_common::values::data_type::DataType;

/// Allows direct access of scan results for a given data type and alignment. Through the use of
/// interval tree index mappings, efficient and low-footprint lookups of scan results are possible.
#[derive(Debug)]
pub struct SnapshotScanResults {
    // data_type: DataType,
    // memory_alignment: MemoryAlignment,
    scan_results_global_lookup_table: ScanResultsLookupTable,
    lookup_tables_by_data_type: DashMap<DataType, ScanResultsLookupTable>,
}

/// Fundamentally, we need to be able to quickly navigate to a specific page number and offset of scan results within a snapshot region.
/// We need to avoid 'seeking' implementations that require repeatedly iterating over the entire scan, and for this we use interval trees.
///
/// There are two steps to building these interval trees:
/// 1) Map the scan result index (global index) to a particular snapshot region.
/// 2) Map a local index to a particular scan result address within this region. This can be parallelized and performed during scans.
///
/// Scan result collections are separated by data type for improved parallelism.
impl SnapshotScanResults {
    pub fn new(
        data_type: DataType,
        memory_alignment: MemoryAlignment,
    ) -> Self {
        Self {
            // data_type,
            // memory_alignment,
            scan_results_global_lookup_table: ScanResultsLookupTable::new(),
            lookup_tables_by_data_type: DashMap::new(),
        }
    }

    pub fn get_scan_result_address(
        &self,
        index: u64,
        snapshot_regions: &Vec<SnapshotRegion>,
    ) -> Option<u64> {
        // Access the scan result lookup table to get the SnapshotRegion containing this scan result index.
        if let Some((scan_result_index_range, snapshot_region_index)) = self
            .scan_results_global_lookup_table
            .get_lookup_mapping()
            .get_key_value(&index)
        {
            if *snapshot_region_index < snapshot_regions.len() as u64 {
                let snapshot_region = &snapshot_regions[*snapshot_region_index as usize];
                let snapshot_region_scan_results_map = snapshot_region.get_region_scan_results();
                let snapshot_filter_index = scan_result_index_range.end() - index;

                if let Some(snapshot_region_scan_results) = snapshot_region_scan_results_map.get(&self.data_type) {
                    return snapshot_region_scan_results.get_scan_result_address(snapshot_filter_index, self.memory_alignment);
                }
            }
        }

        return None;
    }

    pub fn get_number_of_results(&self) -> u64 {
        return self.scan_results_global_lookup_table.get_number_of_results();
    }

    /*
    pub fn new_scan(
        &mut self,
        process_info: &OpenedProcessInfo,
        scan_filter_parameters: Vec<ScanFilterParameters>,
    ) {
        log::info!("Creating new scan...");

        self.create_initial_snapshot_regions(process_info);

        /*
        self.scan_results_by_data_type.clear();

        for scan_filter_parameter in scan_filter_parameters {
            self.scan_results_by_data_type.insert(
                scan_filter_parameter.get_data_type().clone(),
                SnapshotScanResults::new(
                    scan_filter_parameter.get_data_type().clone(),
                    scan_filter_parameter.get_memory_alignment_or_default(),
                ),
            );
            log::info!("Adding data type to new scan: {}", scan_filter_parameter.get_data_type());
        }*/

        log::info!("New scan created");
    }*/

    /*
       pub fn get_scan_result_address(
           &self,
           index: u64,
           data_type: &DataType,
       ) -> Option<u64> {
           if let Some(scan_results) = self.scan_results_by_data_type.get(data_type) {
               return scan_results.get_scan_result_address(index, &self.snapshot_regions);
           }
           return None;
       }

       pub fn get_memory_alignment_or_default_for_data_type(
           &self,
           data_type: &DataType,
       ) -> MemoryAlignment {
           if let Some(scan_results) = self.scan_results_by_data_type.get(data_type) {
               return scan_results.get_memory_alignment();
           }
           return MemoryAlignment::Alignment1;
       }

       pub fn get_scan_results_by_data_type(&self) -> &DashMap<DataType, SnapshotScanResults> {
           return &self.scan_results_by_data_type;
       }

       pub fn get_data_types_and_alignments(&self) -> Vec<(DataType, MemoryAlignment)> {
           let result: Vec<(DataType, MemoryAlignment)> = self
               .scan_results_by_data_type
               .iter()
               .map(|entry| {
                   let data_type = entry.key().clone();
                   let scan_result = entry.value();
                   let alignment = scan_result.get_memory_alignment();
                   (data_type, alignment)
               })
               .collect();

           result
       }

       pub fn build_scan_results_lookup_table(&mut self) {
           for mut scan_results in self.scan_results_by_data_type.iter_mut() {
               return scan_results
                   .value_mut()
                   .build_global_scan_results_lookup_table(&self.snapshot_regions);
           }
       }
    */

    /*

    pub fn set_scan_filter_parameters(
        &mut self,
        scan_filter_parameters: &ScanFilterParameters,
    ) {
        self.data_type = scan_filter_parameters.get_data_type().clone();
        self.memory_alignment = scan_filter_parameters.get_memory_alignment_or_default();
    }

    pub fn get_data_type(&self) -> &DataType {
        return &self.data_type;
    }

    pub fn get_memory_alignment(&self) -> MemoryAlignment {
        return self.memory_alignment;
    }

    /// Creates the lookup tables that allow for easily navigating to the scan results in the provided snapshot.
    pub fn build_global_scan_results_lookup_table(
        &mut self,
        snapshot_regions: &Vec<SnapshotRegion>,
    ) {
        self.scan_results_global_lookup_table.clear();

        // Iterate every snapshot region contained by the snapshot.
        for (region_index, snapshot_region) in snapshot_regions.iter().enumerate() {
            // The snapshot region should already have the scan results, so grab them. At this stage we are now just adding a global indexing.
            let snapshot_region_scan_results_map = snapshot_region.get_region_scan_results();

            // Create scan result lookup table for the data type in this particular set of scan results.
            // Note that there may be more than one SnapshotScanResults instance, each tracking a data type.
            if let Some(snapshot_region_scan_results) = snapshot_region_scan_results_map.get(&self.data_type) {
                let number_of_filter_results = snapshot_region_scan_results.get_number_of_results();

                // Simply map the result range (ie global scan result indicies) onto a the index of a particular snapshot region.
                self.scan_results_global_lookup_table
                    .add_lookup_mapping(number_of_filter_results, region_index as u64);
            }
        }
    }
    */
}
