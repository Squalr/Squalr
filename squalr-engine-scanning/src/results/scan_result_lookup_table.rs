use crate::scanners::constraints::scan_filter_constraint::ScanFilterConstraint;
use crate::snapshots::snapshot_region::SnapshotRegion;
use rangemap::RangeInclusiveMap;
use squalr_engine_common::values::data_type::DataType;
use std::mem::take;
use std::ops::RangeInclusive;

// Scan result index > snapshot filter (within a snapshot region)
type ScanResultIndexToSubRegionMap = RangeInclusiveMap<u64, u64>;
// Scan result index > snapshot region
type ScanResultIndexToRegionMap = RangeInclusiveMap<u64, (u64, ScanResultIndexToSubRegionMap)>;

#[derive(Debug)]
pub struct ScanResultLookupTable {
    page_size: u64,
    scan_result_index_map: ScanResultIndexToRegionMap,
    scan_filter_constraints: Vec<ScanFilterConstraint>
}

/// Fundamentally, we need to be able to quickly navigate to a specific page number and offset of scan results within a snapshot region.
/// We need to avoid 'seeking' implementations that require repeatedly iterating over the entire scan, and for this we need to use interval trees.
/// 
/// The first interval tree maps a range of scan result indexes to a snapshot region index. As a simplified example, we could have:
///     - pages 0-2 => snapshot region index 0
///     - pages 3-8 => snapshot region index 1
///     - pages 9-9 => snapshot region index 2
/// (Simplified for clarity, we would actually be operating on result indexes, not pages)
/// 
/// However, a snapshot region has many filters, so a second interval tree would be needed to index into the correct filter.
///     - pages 0 => filter index 0
///     - pages 1-2 => filter index 1
///     - pages 3-5 => filter index 0
/// (Note that the filter is in the context of the parent snapshot region)
/// 
/// Finally, we can finally offset into this filter to get the discovered address.
impl ScanResultLookupTable {
    pub fn new(
        page_size: u64,
    ) ->
    Self {
        Self {
            page_size: page_size,
            scan_result_index_map: ScanResultIndexToRegionMap::new(),
            scan_filter_constraints: vec![],
        }
    }

    pub fn set_scan_filter_constraints(
        &mut self,
        scan_filter_constraints: Vec<ScanFilterConstraint>,
    ) {
        self.scan_filter_constraints = scan_filter_constraints;
    }

    pub fn get_scan_constraint_filters(
        &self,
    ) -> &Vec<ScanFilterConstraint> {
        return &self.scan_filter_constraints;
    }

    pub fn take_scan_constraint_filters(
        &mut self,
    ) -> Vec<ScanFilterConstraint> {
        return take(&mut self.scan_filter_constraints);
    }

    pub fn build_scan_results(
        &mut self,
        snapshot_regions: &Vec<SnapshotRegion>,
    ) {
        let mut scan_result_index: u64 = 0;

        for (_, filter_constraint) in self.scan_filter_constraints.iter().enumerate() {
            let data_type: &DataType = filter_constraint.get_data_type();
            let memory_alignment = filter_constraint.get_memory_alignment_or_default(data_type);

            for (region_index, region) in snapshot_regions.iter().enumerate() {
                if !region.get_filters().contains_key(data_type) {
                    continue;
                }

                let mut filter_index_map = ScanResultIndexToSubRegionMap::new();
                let filter_regions = region.get_filters().get(data_type).unwrap();
                let region_start_index = scan_result_index;
    
                for (filter_region_index, filter_region) in filter_regions.iter().enumerate() {
                    let element_count = filter_region.get_element_count(memory_alignment, data_type.size_in_bytes());
    
                    // Map the range of scan result indices to the filter index
                    let filter_range = RangeInclusive::new(scan_result_index, scan_result_index + element_count - 1);
                    filter_index_map.insert(filter_range, filter_region_index as u64);
    
                    // Update the scan result index for the next filter
                    scan_result_index += element_count;
                }
    
                // Now map the overall range of scan result indices for this region to the filter map
                {
                    let region_end_index = scan_result_index - 1;
                    self.scan_result_index_map.insert(
                        RangeInclusive::new(region_start_index, region_end_index),
                        (region_index as u64, filter_index_map),
                    );
                }
            }
        }
    }
}
