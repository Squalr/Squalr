use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::results::scan_result_lookup_table::ScanResultLookupTable;
use crate::snapshots::snapshot::Snapshot;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::sync::{Arc, RwLock};

// Work in progress to remove SnapshotSubRegion and replace it with SnapshotRegionFilter
// The idea being that a snapshot and any filtering on that snapshot are independent

// This will allow for complex operations later down the line like scanning for all values -- the idea being that there is this
// base immutable snapshot, but we run it through a filter for each data type and store & track those filters.

// We can then get scan results for each of those filters separately, and even interleave the results (this is annoying but doable)
// The idea being that the scan result index would be passed through each filter sequentially or something. Although this is ass
// and ruins any parallelization, so probably best to keep these segregated by type or something.

// Either way, scans then become filters on filters on filters on filters. Scan results work on filters. We only would ever
// care to make the snapshot mutable again if we decided that we want to prune any regions that are no longer represented by
// any of the filters.

// Boundary scans need to be thought about, but I don't care right now.

// Need to think about the lifetime of filters. In a simpler world, they would be highly transient and used to create new snapshots.
// Then scan results would operate on a snapshot level. Easy peasy.
// However, since we want to be able to support important UX features like "All data types", we clearly need the filters to not be transient
// Additionally, scan results would now be a function of filters, which in turn rely on the snapshot.
// Ok fine, so snapshot filters are no longer transient. But where are they stored?
// I suppose we need a struct (perhaps lumped into scan results) that has the snapshot, and an array of filters (which will often be size 1).

// Alright guess we're doing that.

pub struct SnapshotFilter {
    parent_snapshot: Arc<RwLock<Snapshot>>,
    region_filters: Vec<SnapshotRegionFilter>,
    scan_result_lookup_table: ScanResultLookupTable,
}

impl SnapshotFilter {
    pub fn new(parent_snapshot: Arc<RwLock<Snapshot>>) -> Self {
        Self {
            parent_snapshot: parent_snapshot,
            region_filters: Vec::new(),
            scan_result_lookup_table: ScanResultLookupTable::new(128),
        }
    }

    pub fn get_base_address(&self) -> u64 {
        return 0;
    }

    pub fn get_end_address(&self) -> u64 {
        return 0;
    }
    
    pub fn get_size_in_bytes(&self) -> u64 {
        return 0;
    }

    pub fn get_byte_count_for_data_type_size(&self, data_type_size: u64) -> u64 {
        return 0;
    }

    pub fn get_element_count(&self, alignment: MemoryAlignment, data_type_size: u64) -> u64 {
        return 0;
    }
    /*
    

    pub fn get_byte_count(&self) -> u64 {
        return self.snapshot_sub_regions.iter().map(|sub_region| sub_region.get_byte_count()).sum();
    }

    pub fn get_element_count(&self, alignment: MemoryAlignment, data_type_size: u64) -> u64 {
        return self.snapshot_sub_regions.iter().map(|sub_region| sub_region.get_element_count(alignment, data_type_size)).sum();
    }
    
    pub fn set_snapshot_sub_regions(&mut self, snapshot_sub_regions: Vec<SnapshotSubRegion>) {
        self.snapshot_sub_regions = snapshot_sub_regions;
    }

    pub fn get_snapshot_sub_regions(&self) -> &Vec<SnapshotSubRegion> {
        return &self.snapshot_sub_regions;
    }
    
    pub fn get_snapshot_sub_regions_create_if_none(&mut self) -> Vec<SnapshotSubRegion> {
        if self.snapshot_sub_regions.is_empty() && self.get_region_size() > 0 {
            self.snapshot_sub_regions.push(SnapshotSubRegion::new(self));
        }

        return self.snapshot_sub_regions.clone();
    }

    

    pub fn can_compare_with_constraint(&self, constraints: &ScanConstraint) -> bool {
        if !constraints.is_valid() || !self.has_current_values() {
            return false;
        }

        if !constraints.is_immediate_constraint() && !self.has_previous_values() {
            return false;
        }

        return true;
    }
     */
}
