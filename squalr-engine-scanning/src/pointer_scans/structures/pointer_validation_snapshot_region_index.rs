use crate::pointer_scans::search_kernels::pointer_scan_pointer_value_reader::read_pointer_value_unchecked;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;

const SNAPSHOT_REGION_BUCKET_SHIFT: u32 = 16;

#[derive(Clone, Copy, Debug)]
struct PointerValidationSnapshotRegionBucket {
    bucket_key: u64,
    start_region_index: usize,
    end_region_index_exclusive: usize,
}

impl PointerValidationSnapshotRegionBucket {
    fn new(
        bucket_key: u64,
        start_region_index: usize,
        end_region_index_exclusive: usize,
    ) -> Self {
        Self {
            bucket_key,
            start_region_index,
            end_region_index_exclusive,
        }
    }
}

pub(crate) struct PointerValidationSnapshotRegionIndex<'a> {
    snapshot_regions: &'a [SnapshotRegion],
    snapshot_region_buckets: Vec<PointerValidationSnapshotRegionBucket>,
}

impl<'a> PointerValidationSnapshotRegionIndex<'a> {
    pub(crate) fn new(snapshot_regions: &'a [SnapshotRegion]) -> Self {
        Self {
            snapshot_regions,
            snapshot_region_buckets: Self::build_snapshot_region_buckets(snapshot_regions),
        }
    }

    pub(crate) fn read_pointer_value_at_address(
        &self,
        pointer_address: u64,
        pointer_size: PointerScanPointerSize,
    ) -> Option<u64> {
        let pointer_size_in_bytes = pointer_size.get_size_in_bytes() as usize;
        let snapshot_region = self.find_snapshot_region(pointer_address, pointer_size_in_bytes)?;
        let byte_offset = pointer_address.saturating_sub(snapshot_region.get_base_address()) as usize;
        let pointer_bytes = snapshot_region
            .get_current_values()
            .get(byte_offset..byte_offset.saturating_add(pointer_size_in_bytes))?;

        // Snapshot bytes are already in host memory, so decode them directly without slice copies.
        Some(unsafe { read_pointer_value_unchecked(pointer_bytes.as_ptr(), pointer_size) })
    }

    fn find_snapshot_region(
        &self,
        pointer_address: u64,
        pointer_size_in_bytes: usize,
    ) -> Option<&'a SnapshotRegion> {
        let matching_bucket_index = self.find_matching_bucket_index(pointer_address)?;
        let matching_bucket = self.snapshot_region_buckets.get(matching_bucket_index)?;
        let pointer_end_address = pointer_address.checked_add(pointer_size_in_bytes as u64)?;

        for snapshot_region_index in matching_bucket.start_region_index..matching_bucket.end_region_index_exclusive {
            let snapshot_region = self.snapshot_regions.get(snapshot_region_index)?;

            if pointer_address < snapshot_region.get_base_address() {
                return None;
            }

            if pointer_end_address <= snapshot_region.get_end_address() {
                return Some(snapshot_region);
            }
        }

        None
    }

    fn build_snapshot_region_buckets(snapshot_regions: &[SnapshotRegion]) -> Vec<PointerValidationSnapshotRegionBucket> {
        let mut snapshot_region_buckets: Vec<PointerValidationSnapshotRegionBucket> = Vec::new();

        for (snapshot_region_index, snapshot_region) in snapshot_regions.iter().enumerate() {
            if snapshot_region.get_current_values().is_empty() {
                continue;
            }

            let start_bucket_key = snapshot_region.get_base_address() >> SNAPSHOT_REGION_BUCKET_SHIFT;
            let end_bucket_key = snapshot_region.get_end_address().saturating_sub(1) >> SNAPSHOT_REGION_BUCKET_SHIFT;

            for bucket_key in start_bucket_key..=end_bucket_key {
                if let Some(last_snapshot_region_bucket) = snapshot_region_buckets.last_mut() {
                    if last_snapshot_region_bucket.bucket_key == bucket_key {
                        last_snapshot_region_bucket.end_region_index_exclusive = snapshot_region_index.saturating_add(1);
                        continue;
                    }
                }

                snapshot_region_buckets.push(PointerValidationSnapshotRegionBucket::new(
                    bucket_key,
                    snapshot_region_index,
                    snapshot_region_index.saturating_add(1),
                ));
            }
        }

        snapshot_region_buckets
    }

    fn find_matching_bucket_index(
        &self,
        pointer_address: u64,
    ) -> Option<usize> {
        let bucket_key = pointer_address >> SNAPSHOT_REGION_BUCKET_SHIFT;
        let mut lower_index = 0_usize;
        let mut upper_index = self.snapshot_region_buckets.len();

        while lower_index < upper_index {
            let middle_index = lower_index.saturating_add(upper_index.saturating_sub(lower_index) / 2);
            let candidate_bucket_key = self.snapshot_region_buckets[middle_index].bucket_key;

            if candidate_bucket_key < bucket_key {
                lower_index = middle_index.saturating_add(1);
            } else {
                upper_index = middle_index;
            }
        }

        self.snapshot_region_buckets
            .get(lower_index)
            .and_then(|snapshot_region_bucket| (snapshot_region_bucket.bucket_key == bucket_key).then_some(lower_index))
    }
}
