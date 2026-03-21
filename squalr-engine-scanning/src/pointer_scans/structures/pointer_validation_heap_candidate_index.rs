use squalr_engine_api::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;

pub(crate) struct PointerValidationHeapCandidateIndex<'a> {
    sorted_heap_candidates: Vec<&'a PointerScanCandidate>,
    sorted_heap_addresses: Vec<u64>,
}

impl<'a> PointerValidationHeapCandidateIndex<'a> {
    pub(crate) fn new(heap_candidates: &'a [PointerScanCandidate]) -> Self {
        let mut sorted_heap_candidates = heap_candidates.iter().collect::<Vec<_>>();
        sorted_heap_candidates.sort_unstable_by_key(|pointer_scan_candidate| pointer_scan_candidate.get_pointer_address());
        let sorted_heap_addresses = sorted_heap_candidates
            .iter()
            .map(|pointer_scan_candidate| pointer_scan_candidate.get_pointer_address())
            .collect();

        Self {
            sorted_heap_candidates,
            sorted_heap_addresses,
        }
    }

    pub(crate) fn find_candidates_in_range(
        &self,
        lower_bound: u64,
        upper_bound: u64,
    ) -> &[&'a PointerScanCandidate] {
        if lower_bound > upper_bound || self.sorted_heap_candidates.is_empty() {
            return &[];
        }

        let start_index = self.find_first_candidate_index_at_or_above(lower_bound);
        let end_index = self.find_first_candidate_index_above(upper_bound);

        &self.sorted_heap_candidates[start_index..end_index]
    }

    fn find_first_candidate_index_at_or_above(
        &self,
        target_address: u64,
    ) -> usize {
        let mut lower_index = 0_usize;
        let mut upper_index = self.sorted_heap_addresses.len();

        while lower_index < upper_index {
            let middle_index = lower_index.saturating_add(upper_index.saturating_sub(lower_index) / 2);

            if self.sorted_heap_addresses[middle_index] < target_address {
                lower_index = middle_index.saturating_add(1);
            } else {
                upper_index = middle_index;
            }
        }

        lower_index
    }

    fn find_first_candidate_index_above(
        &self,
        target_address: u64,
    ) -> usize {
        let mut lower_index = 0_usize;
        let mut upper_index = self.sorted_heap_addresses.len();

        while lower_index < upper_index {
            let middle_index = lower_index.saturating_add(upper_index.saturating_sub(lower_index) / 2);

            if self.sorted_heap_addresses[middle_index] <= target_address {
                lower_index = middle_index.saturating_add(1);
            } else {
                upper_index = middle_index;
            }
        }

        lower_index
    }
}
