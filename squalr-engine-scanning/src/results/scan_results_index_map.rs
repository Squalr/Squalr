use std::ops::RangeInclusive;

use rangemap::RangeInclusiveMap;

type ScanResultRangeMap = RangeInclusiveMap<u64, u64>;

#[derive(Debug)]
pub struct ScanResultsIndexMap {
    scan_result_range_map: ScanResultRangeMap,
    number_of_result_bytes: u64,
}

impl ScanResultsIndexMap {
    pub fn new() -> Self {
        Self {
            scan_result_range_map: ScanResultRangeMap::new(),
            number_of_result_bytes: 0,
        }
    }

    pub fn get_scan_result_range_map(&self) -> &ScanResultRangeMap {
        return &self.scan_result_range_map;
    }

    pub fn get_number_of_result_bytes(&self) -> u64 {
        return self.number_of_result_bytes;
    }

    pub fn insert(
        &mut self,
        range_start: u64,
        range_size: u64,
        value: u64,
    ) {
        self.scan_result_range_map
            .insert(RangeInclusive::new(range_start, range_start + range_size), value);

        self.number_of_result_bytes += range_size;
    }

    pub fn clear(&mut self) {
        self.scan_result_range_map.clear();
        self.number_of_result_bytes = 0;
    }
}
