use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use rangemap::RangeInclusiveMap;

type ScanResultIndexToFilterMap = RangeInclusiveMap<u64, u64>;

#[derive(Debug)]
pub struct SnapshotFilterCollection {
    /// While this looks silly, it is better to have a vector of vectors for parallelization.
    /// This is because when we scan a filter, it produces a list of filters. Combining these back into
    /// one giant list would cost too much scan time, so it's better to keep it as a list of lists.
    snapshot_filters: Vec<Vec<SnapshotRegionFilter>>,
    scan_results_lookup_table: ScanResultIndexToFilterMap,
}

impl SnapshotFilterCollection {
    pub fn new(snapshot_filters: Vec<Vec<SnapshotRegionFilter>>) -> Self {
        Self {
            snapshot_filters: snapshot_filters,
            scan_results_lookup_table: ScanResultIndexToFilterMap::new(),
        }
    }

    pub fn new_from_single_filter(snapshot_filter: SnapshotRegionFilter) -> Self {
        Self {
            snapshot_filters: vec![vec![snapshot_filter]],
            scan_results_lookup_table: ScanResultIndexToFilterMap::new(),
        }
    }

    pub fn get_filters(&self) -> &Vec<Vec<SnapshotRegionFilter>> {
        return &self.snapshot_filters;
    }
}
