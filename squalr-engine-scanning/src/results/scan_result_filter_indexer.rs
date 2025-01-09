use crate::results::scan_results_index_map::ScanResultsIndexMap;
use crate::results::snapshot_region_filter::SnapshotRegionFilter;
use rangemap::RangeInclusiveMap;
use squalr_engine_memory::memory_alignment::MemoryAlignment;

/*
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

    pub fn get_number_of_results(
        &self,
        memory_alignment: MemoryAlignment,
    ) -> u64 {
        return self.filter_results_lookup_table.get_number_of_results() / (memory_alignment as u64);
    }

    pub fn get_scan_result_range_map(&self) -> &RangeInclusiveMap<u64, u64> {
        return &self.filter_results_lookup_table.get_scan_result_range_map();
    }

    pub fn build_lookup_table(
        &mut self,
        snapshot_filters: &Vec<Vec<SnapshotRegionFilter>>,
    ) {
        self.filter_results_lookup_table.clear();

        // Iterate all snapshot filters mapped over the same snapshot_region.
        for (filter_region_index, filter_region) in snapshot_filters.iter().flatten().enumerate() {
            let filter_size = filter_region.get_region_size();
            let current_number_of_result_bytes = self.filter_results_lookup_table.get_number_of_results();

            // Map the element range onto the filter region index.
            self.filter_results_lookup_table
                .insert(current_number_of_result_bytes, filter_size.saturating_sub(1), filter_region_index as u64);
        }
    }
}
 */
