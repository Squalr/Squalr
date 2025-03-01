use rangemap::RangeInclusiveMap;
use std::ops::RangeInclusive;

type ScanResultRangeMap = RangeInclusiveMap<u64, u64>;

/// Defines a mapping of scan result indicies onto the corresponding snapshot region containing the results.
#[derive(Debug)]
pub struct SnapshotRegionFilterLookupTable {
    result_index_to_region_index_map: ScanResultRangeMap,
    number_of_results: u64,
}

impl SnapshotRegionFilterLookupTable {
    pub fn new() -> Self {
        Self {
            result_index_to_region_index_map: ScanResultRangeMap::new(),
            number_of_results: 0,
        }
    }

    /// Gets the internal interval tree mapping of the lookup table.
    pub fn get_lookup_mapping(&self) -> &ScanResultRangeMap {
        return &self.result_index_to_region_index_map;
    }

    /// Gets the number of scan results mapped by this lookup table.
    pub fn get_number_of_results(&self) -> u64 {
        return self.number_of_results;
    }

    /// Maps the next specified `n` scan results, denoted by `result_range_count` to a specific region index.
    pub fn add_lookup_mapping(
        &mut self,
        result_range_count: u64,
        region_index: u64,
    ) {
        let result_index_start = self.get_number_of_results();

        self.result_index_to_region_index_map.insert(
            RangeInclusive::new(result_index_start, result_index_start + result_range_count.saturating_sub(1)),
            region_index,
        );

        self.number_of_results = self.number_of_results.saturating_add(result_range_count);
    }

    /// Removes all entries from the lookup table.
    pub fn clear(&mut self) {
        self.result_index_to_region_index_map.clear();
        self.number_of_results = 0;
    }
}
