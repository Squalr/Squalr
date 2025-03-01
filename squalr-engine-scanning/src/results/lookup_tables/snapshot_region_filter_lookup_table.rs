type ResultIndexBaseToFilterIndex = (u64, u64);

struct LookupTableEntry {
    local_index_base: u64,
    filter_index: u64,
}

/// Defines a mapping of scan result indicies onto the corresponding snapshot region containing the results.
pub struct SnapshotRegionFilterLookupTable {
    index_table: Vec<LookupTableEntry>,
    number_of_results: u64,
}

impl SnapshotRegionFilterLookupTable {
    pub fn new() -> Self {
        Self {
            index_table: vec![],
            number_of_results: 0,
        }
    }

    /// Gets the number of results contained in this lookup table.
    pub fn get_number_of_results(&self) -> u64 {
        self.number_of_results
    }

    pub fn append_lookup_mapping(
        &mut self,
        result_range_count: u64,
        filter_index: u64,
    ) {
        self.index_table.push(LookupTableEntry {
            local_index_base: self.number_of_results,
            filter_index,
        });

        self.number_of_results = self.number_of_results.saturating_add(result_range_count);
    }

    pub fn lookup_filter_base_address_and_index(
        &self,
        local_scan_result_index: u64,
    ) -> Option<ResultIndexBaseToFilterIndex> {
        // Binary search for the largest local_index_base <= local_scan_result_index.
        let binary_search_result = self
            .index_table
            .binary_search_by_key(&local_scan_result_index, |entry| entry.local_index_base);

        match binary_search_result {
            // If an exact match is found, use it directly.
            Ok(index) => {
                let entry = &self.index_table[index];
                Some((entry.filter_index, local_scan_result_index - entry.local_index_base))
            }
            // If no exact match, `Err(index)` tells us where it would be inserted.
            Err(index) if index > 0 => {
                let entry = &self.index_table[index - 1];
                Some((entry.filter_index, local_scan_result_index - entry.local_index_base))
            }
            // If index == 0, then no valid range exists
            _ => None,
        }
    }
}
