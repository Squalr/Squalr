use std::ops::RangeInclusive;

use rangemap::RangeInclusiveMap;

type ScanResultRangeMap = RangeInclusiveMap<u64, u64>;

#[derive(Debug)]
pub struct ScanResultsIndexMap {
    scan_result_range_map: ScanResultRangeMap,
    number_of_results: u64,
}

impl ScanResultsIndexMap {
    pub fn new() -> Self {
        Self {
            scan_result_range_map: ScanResultRangeMap::new(),
            number_of_results: 0,
        }
    }

    pub fn get_scan_result_range_map(&self) -> &ScanResultRangeMap {
        return &self.scan_result_range_map;
    }

    pub fn get_number_of_results(&self) -> u64 {
        return self.number_of_results;
    }

    pub fn insert(
        &mut self,
        element_range_start: u64,
        element_range_size: u64,
        region_index: u64,
    ) {
        self.scan_result_range_map.insert(
            RangeInclusive::new(element_range_start, element_range_start + element_range_size.saturating_sub(1)),
            region_index,
        );

        self.number_of_results += element_range_size;
    }

    pub fn clear(&mut self) {
        self.scan_result_range_map.clear();
        self.number_of_results = 0;
    }
}
