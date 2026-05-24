use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;

pub fn merge_memory_regions_into_snapshot_regions(memory_regions: Vec<NormalizedRegion>) -> Vec<SnapshotRegion> {
    let mut merged_snapshot_regions = Vec::new();
    let mut page_boundaries = Vec::new();
    let mut memory_region_iterator = memory_regions.into_iter();
    let Some(mut current_memory_region) = memory_region_iterator.next() else {
        return merged_snapshot_regions;
    };

    for next_memory_region in memory_region_iterator {
        if current_memory_region.get_end_address() == next_memory_region.get_base_address() {
            current_memory_region.set_end_address(next_memory_region.get_end_address());
            page_boundaries.push(next_memory_region.get_base_address());
        } else {
            merged_snapshot_regions.push(SnapshotRegion::new(current_memory_region, std::mem::take(&mut page_boundaries)));
            current_memory_region = next_memory_region;
        }
    }

    merged_snapshot_regions.push(SnapshotRegion::new(current_memory_region, page_boundaries));

    merged_snapshot_regions
}

#[cfg(test)]
mod tests {
    use super::merge_memory_regions_into_snapshot_regions;
    use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;

    #[test]
    fn merge_memory_regions_into_snapshot_regions_merges_adjacent_regions_and_tracks_boundaries() {
        let merged_snapshot_regions = merge_memory_regions_into_snapshot_regions(vec![
            NormalizedRegion::new(0x1000, 0x1000),
            NormalizedRegion::new(0x2000, 0x1000),
            NormalizedRegion::new(0x4000, 0x1000),
        ]);

        assert_eq!(merged_snapshot_regions.len(), 2);
        assert_eq!(merged_snapshot_regions[0].get_base_address(), 0x1000);
        assert_eq!(merged_snapshot_regions[0].get_region_size(), 0x2000);
        assert_eq!(merged_snapshot_regions[0].page_boundaries, vec![0x2000]);
        assert_eq!(merged_snapshot_regions[1].get_base_address(), 0x4000);
        assert_eq!(merged_snapshot_regions[1].get_region_size(), 0x1000);
        assert!(merged_snapshot_regions[1].page_boundaries.is_empty());
    }

    #[test]
    fn merge_memory_regions_into_snapshot_regions_returns_empty_when_no_regions_are_provided() {
        assert!(merge_memory_regions_into_snapshot_regions(Vec::new()).is_empty());
    }
}
