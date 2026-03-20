#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerScanTargetRange {
    lower_bound: u64,
    upper_bound: u64,
}

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

    pub fn contains_value(
        &self,
        pointer_value: u64,
    ) -> bool {
        pointer_value >= self.lower_bound && pointer_value <= self.upper_bound
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PointerScanTargetRangeSet {
    source_target_count: usize,
    target_ranges: Vec<PointerScanTargetRange>,
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

        Self {
            source_target_count,
            target_ranges,
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

    pub fn contains_value_linear(
        &self,
        pointer_value: u64,
    ) -> bool {
        self.find_matching_range_index_linear(pointer_value).is_some()
    }

    pub fn contains_value_binary(
        &self,
        pointer_value: u64,
    ) -> bool {
        self.find_matching_range_index_binary(pointer_value).is_some()
    }

    pub fn find_matching_range_index_linear(
        &self,
        pointer_value: u64,
    ) -> Option<usize> {
        for (target_range_index, target_range) in self.target_ranges.iter().enumerate() {
            if pointer_value < target_range.get_lower_bound() {
                return None;
            }

            if target_range.contains_value(pointer_value) {
                return Some(target_range_index);
            }
        }

        None
    }

    pub fn find_matching_range_index_binary(
        &self,
        pointer_value: u64,
    ) -> Option<usize> {
        let matching_range_index = self
            .target_ranges
            .partition_point(|target_range| target_range.get_lower_bound() <= pointer_value);

        if matching_range_index == 0 {
            return None;
        }

        let target_range_index = matching_range_index.saturating_sub(1);
        let target_range = self.target_ranges.get(target_range_index)?;

        target_range
            .contains_value(pointer_value)
            .then_some(target_range_index)
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
}
