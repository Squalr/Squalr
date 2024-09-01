use crate::results::scan_result::ScanResult;
use crate::results::scan_results_index_map::ScanResultsIndexMap;
use crate::scanners::parameters::scan_filter_parameters::ScanFilterParameters;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_common::values::data_type::DataType;
use squalr_engine_memory::memory_alignment::MemoryAlignment;

#[derive(Debug)]
pub struct SnapshotScanResults {
    data_type: DataType,
    memory_alignment: MemoryAlignment,
    lookup_table: ScanResultsIndexMap,
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
    pub fn new(
        data_type: DataType,
        memory_alignment: MemoryAlignment,
    ) -> Self {
        Self {
            data_type: data_type,
            memory_alignment: memory_alignment,
            lookup_table: ScanResultsIndexMap::new(),
        }
    }

    pub fn get_scan_result(
        &self,
        index: u64,
        snapshot_regions: &Vec<SnapshotRegion>,
    ) -> Option<ScanResult> {
        if let Some((snapshot_region_range, snapshot_region_index)) = self
            .lookup_table
            .get_scan_result_range_map()
            .get_key_value(&index)
        {
            if *snapshot_region_index < snapshot_regions.len() as u64 {
                let snapshot_region = &snapshot_regions[*snapshot_region_index as usize];
                let snapshot_region_scan_results = snapshot_region.get_scan_results();
                let snapshot_filter_index = snapshot_region_range.end() - index;

                return snapshot_region_scan_results.get_scan_result(snapshot_filter_index, &self.data_type, self.memory_alignment);
            }
        }

        return None;
    }

    pub fn get_number_of_results(&self) -> u64 {
        return self.lookup_table.get_number_of_result_bytes() / (self.memory_alignment as u64);
    }

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

    pub fn build_scan_results(
        &mut self,
        snapshot_regions: &Vec<SnapshotRegion>,
    ) {
        self.lookup_table.clear();

        // Iterate every snapshot region contained by the snapshot.
        for (region_index, snapshot_region) in snapshot_regions.iter().enumerate() {
            let snapshot_region_scan_results = snapshot_region.get_scan_results();

            if !snapshot_region_scan_results
                .get_filters()
                .contains_key(&self.data_type)
            {
                continue;
            }

            let number_of_filter_result_bytes = snapshot_region_scan_results.get_number_of_result_bytes(&self.data_type);
            let current_number_of_result_bytes = self.lookup_table.get_number_of_result_bytes();

            // Simply map the result range onto a the index of a particular snapshot region.
            self.lookup_table
                .insert(current_number_of_result_bytes, number_of_filter_result_bytes, region_index as u64);
        }
    }
}
