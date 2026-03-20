#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerScanTargetRange {
    lower_bound: u64,
    upper_bound: u64,
}

const TARGET_RANGE_BUCKET_SHIFT: u32 = 16;
const TARGET_RANGE_BUCKET_LINEAR_SEARCH_THRESHOLD: usize = 8;

impl PointerScanTargetRange {
    pub fn new(
        lower_bound: u64,
        upper_bound: u64,
    ) -> Self {
        Self { lower_bound, upper_bound }
    }

    pub fn get_lower_bound(&self) -> u64 {
        self.lower_bound
    }

    pub fn get_upper_bound(&self) -> u64 {
        self.upper_bound
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PointerScanTargetRangeBucket {
    bucket_key: u64,
    start_range_index: usize,
    end_range_index_exclusive: usize,
}

impl PointerScanTargetRangeBucket {
    fn new(
        bucket_key: u64,
        start_range_index: usize,
        end_range_index_exclusive: usize,
    ) -> Self {
        Self {
            bucket_key,
            start_range_index,
            end_range_index_exclusive,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PointerScanTargetRangeSet {
    source_target_count: usize,
    target_ranges: Vec<PointerScanTargetRange>,
    lower_bounds: Vec<u64>,
    upper_bounds: Vec<u64>,
    target_range_buckets: Vec<PointerScanTargetRangeBucket>,
}

impl PointerScanTargetRangeSet {
    pub fn from_target_addresses(
        target_addresses: &[u64],
        offset_radius: u64,
    ) -> Self {
        if target_addresses.is_empty() {
            return Self::default();
        }

        let source_target_count = target_addresses.len();
        let mut sorted_target_addresses = target_addresses.to_vec();
        sorted_target_addresses.sort_unstable();
        sorted_target_addresses.dedup();

        let mut target_ranges: Vec<PointerScanTargetRange> = Vec::with_capacity(sorted_target_addresses.len());

        for target_address in sorted_target_addresses {
            let next_target_range = PointerScanTargetRange::new(target_address.saturating_sub(offset_radius), target_address.saturating_add(offset_radius));

            if let Some(last_target_range) = target_ranges.last_mut() {
                if next_target_range.get_lower_bound() <= last_target_range.get_upper_bound().saturating_add(1) {
                    *last_target_range = PointerScanTargetRange::new(
                        last_target_range.get_lower_bound(),
                        last_target_range
                            .get_upper_bound()
                            .max(next_target_range.get_upper_bound()),
                    );
                    continue;
                }
            }

            target_ranges.push(next_target_range);
        }

        let lower_bounds = target_ranges
            .iter()
            .map(PointerScanTargetRange::get_lower_bound)
            .collect::<Vec<_>>();
        let upper_bounds = target_ranges
            .iter()
            .map(PointerScanTargetRange::get_upper_bound)
            .collect::<Vec<_>>();
        let target_range_buckets = Self::build_target_range_buckets(&target_ranges);

        Self {
            source_target_count,
            target_ranges,
            lower_bounds,
            upper_bounds,
            target_range_buckets,
        }
    }

    pub fn get_source_target_count(&self) -> usize {
        self.source_target_count
    }

    pub fn get_range_count(&self) -> usize {
        self.target_ranges.len()
    }

    pub fn get_target_ranges(&self) -> &[PointerScanTargetRange] {
        &self.target_ranges
    }

    pub fn is_empty(&self) -> bool {
        self.target_ranges.is_empty()
    }

    #[inline(always)]
    pub fn contains_value_linear(
        &self,
        pointer_value: u64,
    ) -> bool {
        self.find_matching_range_index_linear(pointer_value).is_some()
    }

    #[inline(always)]
    pub fn contains_value_binary(
        &self,
        pointer_value: u64,
    ) -> bool {
        self.find_matching_range_index_binary(pointer_value).is_some()
    }

    #[inline(always)]
    pub fn find_matching_range_index_linear(
        &self,
        pointer_value: u64,
    ) -> Option<usize> {
        self.find_matching_range_index_linear_in_bounds(pointer_value, 0, self.target_ranges.len())
    }

    #[inline(always)]
    pub fn find_matching_range_index_binary(
        &self,
        pointer_value: u64,
    ) -> Option<usize> {
        let matching_bucket_index = self.find_matching_bucket_index(pointer_value)?;
        let target_range_bucket = self.target_range_buckets.get(matching_bucket_index)?;
        let bucket_range_count = target_range_bucket
            .end_range_index_exclusive
            .saturating_sub(target_range_bucket.start_range_index);

        if bucket_range_count <= TARGET_RANGE_BUCKET_LINEAR_SEARCH_THRESHOLD {
            return self.find_matching_range_index_linear_in_bounds(
                pointer_value,
                target_range_bucket.start_range_index,
                target_range_bucket.end_range_index_exclusive,
            );
        }

        self.find_matching_range_index_binary_in_bounds(
            pointer_value,
            target_range_bucket.start_range_index,
            target_range_bucket.end_range_index_exclusive,
        )
    }

    fn build_target_range_buckets(target_ranges: &[PointerScanTargetRange]) -> Vec<PointerScanTargetRangeBucket> {
        let mut target_range_buckets: Vec<PointerScanTargetRangeBucket> = Vec::new();

        for (target_range_index, target_range) in target_ranges.iter().enumerate() {
            let lower_bucket_key = target_range.get_lower_bound() >> TARGET_RANGE_BUCKET_SHIFT;
            let upper_bucket_key = target_range.get_upper_bound() >> TARGET_RANGE_BUCKET_SHIFT;

            for bucket_key in lower_bucket_key..=upper_bucket_key {
                if let Some(last_target_range_bucket) = target_range_buckets.last_mut() {
                    if last_target_range_bucket.bucket_key == bucket_key {
                        last_target_range_bucket.end_range_index_exclusive = target_range_index.saturating_add(1);
                        continue;
                    }
                }

                target_range_buckets.push(PointerScanTargetRangeBucket::new(
                    bucket_key,
                    target_range_index,
                    target_range_index.saturating_add(1),
                ));
            }
        }

        target_range_buckets
    }

    #[inline(always)]
    fn find_matching_bucket_index(
        &self,
        pointer_value: u64,
    ) -> Option<usize> {
        let bucket_key = pointer_value >> TARGET_RANGE_BUCKET_SHIFT;
        let matching_bucket_index = self
            .target_range_buckets
            .partition_point(|target_range_bucket| target_range_bucket.bucket_key < bucket_key);
        let target_range_bucket = self.target_range_buckets.get(matching_bucket_index)?;

        (target_range_bucket.bucket_key == bucket_key).then_some(matching_bucket_index)
    }

    #[inline(always)]
    fn find_matching_range_index_linear_in_bounds(
        &self,
        pointer_value: u64,
        search_start_index: usize,
        search_end_index_exclusive: usize,
    ) -> Option<usize> {
        for target_range_index in search_start_index..search_end_index_exclusive {
            let lower_bound = *self.lower_bounds.get(target_range_index)?;

            if pointer_value < lower_bound {
                return None;
            }

            let upper_bound = *self.upper_bounds.get(target_range_index)?;

            if pointer_value <= upper_bound {
                return Some(target_range_index);
            }
        }

        None
    }

    #[inline(always)]
    fn find_matching_range_index_binary_in_bounds(
        &self,
        pointer_value: u64,
        search_start_index: usize,
        search_end_index_exclusive: usize,
    ) -> Option<usize> {
        let lower_bounds = self
            .lower_bounds
            .get(search_start_index..search_end_index_exclusive)?;
        let matching_range_index = lower_bounds.partition_point(|lower_bound| *lower_bound <= pointer_value);

        if matching_range_index == 0 {
            return None;
        }

        let target_range_index = search_start_index.saturating_add(matching_range_index.saturating_sub(1));
        let upper_bound = *self.upper_bounds.get(target_range_index)?;

        (pointer_value <= upper_bound).then_some(target_range_index)
    }
}

#[cfg(test)]
mod tests {
    use super::{PointerScanTargetRange, PointerScanTargetRangeSet};

    #[test]
    fn target_range_set_merges_overlapping_expanded_frontiers() {
        let target_range_set = PointerScanTargetRangeSet::from_target_addresses(&[0x3000, 0x3080, 0x4000], 0x100);

        assert_eq!(target_range_set.get_source_target_count(), 3);
        assert_eq!(target_range_set.get_range_count(), 2);
        assert_eq!(
            target_range_set.get_target_ranges(),
            &[
                PointerScanTargetRange::new(0x2F00, 0x3180),
                PointerScanTargetRange::new(0x3F00, 0x4100),
            ],
        );
    }

    #[test]
    fn target_range_set_binary_and_linear_checks_agree() {
        let target_range_set = PointerScanTargetRangeSet::from_target_addresses(&[0x3000, 0x3400, 0x3800], 0x20);

        assert!(target_range_set.contains_value_linear(0x3000));
        assert!(target_range_set.contains_value_binary(0x3000));
        assert!(target_range_set.contains_value_linear(0x3410));
        assert!(target_range_set.contains_value_binary(0x3410));
        assert!(!target_range_set.contains_value_linear(0x3600));
        assert!(!target_range_set.contains_value_binary(0x3600));
    }

    #[test]
    fn target_range_set_bucketed_binary_checks_agree_with_linear_checks() {
        let target_addresses = (0_u64..256)
            .map(|target_index| 0x1000_0000_u64.saturating_add(target_index.saturating_mul(0x18_000)))
            .collect::<Vec<_>>();
        let target_range_set = PointerScanTargetRangeSet::from_target_addresses(&target_addresses, 0x80);
        let probe_values = [
            0x0FFF_FFFF,
            0x1000_0040,
            0x1001_7F80,
            0x1001_8100,
            0x1018_0000,
            0x102A_8010,
            0x11FF_FFFF,
        ];

        for probe_value in probe_values {
            assert_eq!(
                target_range_set.contains_value_binary(probe_value),
                target_range_set.contains_value_linear(probe_value)
            );
        }
    }
}
