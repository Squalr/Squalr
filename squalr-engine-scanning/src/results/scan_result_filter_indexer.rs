use crate::results::scan_results_index_map::ScanResultsIndexMap;
use crate::results::snapshot_region_filter::SnapshotRegionFilter;
use rangemap::RangeInclusiveMap;
use squalr_engine_common::values::data_type::DataType;
use squalr_engine_memory::memory_alignment::MemoryAlignment;

#[derive(Debug)]
pub struct SnapshotFilterIndexer {
    filter_results_lookup_table: ScanResultsIndexMap,
}

impl SnapshotFilterIndexer {
    pub fn new() -> Self {
        Self {
            filter_results_lookup_table: ScanResultsIndexMap::new(),
        }
    }

    pub fn get_number_of_results(&self) -> u64 {
        return self.filter_results_lookup_table.get_number_of_results();
    }

    pub fn get_scan_result_range_map(&self) -> &RangeInclusiveMap<u64, u64> {
        return &self.filter_results_lookup_table.get_scan_result_range_map();
    }

    pub fn build_lookup_table(
        &mut self,
        snapshot_filters: &Vec<Vec<SnapshotRegionFilter>>,
        data_type: &DataType,
        memory_alignment: MemoryAlignment,
    ) {
        self.filter_results_lookup_table.clear();

        // Iterate all snapshot filters mapped over the same snapshot_region.
        for (filter_region_index, filter_region) in snapshot_filters.iter().flatten().enumerate() {
            let element_count = filter_region.get_element_count(memory_alignment, data_type.get_size_in_bytes());
            let current_number_of_results = self.filter_results_lookup_table.get_number_of_results();

            // Map the element range onto the filter region index.
            self.filter_results_lookup_table
                .insert(current_number_of_results, element_count - 1, filter_region_index as u64);
        }
    }
}
