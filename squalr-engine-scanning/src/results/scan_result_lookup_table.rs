use crate::snapshots::snapshot::Snapshot;
use rangemap::RangeInclusiveMap;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::ops::RangeInclusive;
use std::sync::Arc;
use std::sync::RwLock;

// Scan result index > snapshot subregion (within a snapshot region)
type ScanResultIndexToSubRegionMap = RangeInclusiveMap<u64, u64>;
// Scan result index > snapshot region
type ScanResultIndexToRegionMap = RangeInclusiveMap<u64, (u64, ScanResultIndexToSubRegionMap)>;

#[derive(Debug)]
pub struct ScanResultLookupTable {
    page_size: u64,
    scan_result_index_map: ScanResultIndexToRegionMap,
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
/// However, a snapshot region has many subregions, so a second interval tree would be needed to index into the correct subregion.
///     - pages 0 => subregion index 0
///     - pages 1-2 => subregion index 1
///     - pages 3-5 => subregion index 0
/// (Note that the subregion is in the context of the parent snapshot region)
/// 
/// Finally, we can finally offset into this subregion to get the discovered address.
impl ScanResultLookupTable {
    pub fn new(
        page_size: u64,
    ) ->
    Self {
        Self {
            page_size: page_size,
            scan_result_index_map: ScanResultIndexToRegionMap::new(),
        }
    }

    pub fn build_scan_results(
        &mut self,
        snapshot: Arc<RwLock<Snapshot>>,
        alignment: MemoryAlignment,
        data_type_size: u64,
    ) {
        let snapshot = snapshot.read().unwrap();
        let snapshot_regions = snapshot.get_snapshot_regions();
        let mut scan_result_index: u64 = 0;

        // TODO: Migrate from subregions to new filter struct
        /*
        for (region_index, region) in snapshot_regions.iter().enumerate() {
            let mut subregion_index_map = ScanResultIndexToSubRegionMap::new();
            let sub_regions = region.get_snapshot_sub_regions();
            let region_start_index = scan_result_index;

            for (sub_region_index, sub_region) in sub_regions.iter().enumerate() {
                let element_count = sub_region.get_element_count(alignment, data_type_size);

                // Map the range of scan result indices to the subregion index
                let subregion_range = RangeInclusive::new(scan_result_index, scan_result_index + element_count - 1);
                subregion_index_map.insert(subregion_range, sub_region_index as u64);

                // Update the scan result index for the next subregion
                scan_result_index += element_count;
            }

            // Now map the overall range of scan result indices for this region to the subregion map
            {
                let region_end_index = scan_result_index - 1;
                self.scan_result_index_map.insert(
                    RangeInclusive::new(region_start_index, region_end_index),
                    (region_index as u64, subregion_index_map),
                );
            }
        } */
    }
}
