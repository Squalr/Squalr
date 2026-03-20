use crate::pointer_scans::pointer_scan_target_ranges::PointerScanTargetRangeSet;
use squalr_engine_api::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;

pub struct PointerScanRootTracker {
    offset_radius: u64,
    prior_heap_target_ranges: Option<PointerScanTargetRangeSet>,
    root_node_count: u64,
}

impl PointerScanRootTracker {
    pub fn new(offset_radius: u64) -> Self {
        Self {
            offset_radius,
            prior_heap_target_ranges: None,
            root_node_count: 0,
        }
    }

    pub fn record_static_candidate(
        &mut self,
        discovery_depth: u64,
        pointer_value: u64,
    ) {
        if Self::is_displayable_root_candidate(discovery_depth, pointer_value, self.prior_heap_target_ranges.as_ref()) {
            self.root_node_count = self.root_node_count.saturating_add(1);
        }
    }

    pub fn advance_to_next_level(
        &mut self,
        heap_candidates: &[PointerScanCandidate],
    ) {
        if heap_candidates.is_empty() {
            self.prior_heap_target_ranges = None;
            return;
        }

        let heap_candidate_addresses = heap_candidates
            .iter()
            .map(PointerScanCandidate::get_pointer_address)
            .collect::<Vec<_>>();

        self.prior_heap_target_ranges = Some(PointerScanTargetRangeSet::from_sorted_target_addresses(
            &heap_candidate_addresses,
            self.offset_radius,
        ));
    }

    pub fn get_root_node_count(&self) -> u64 {
        self.root_node_count
    }

    fn is_displayable_root_candidate(
        discovery_depth: u64,
        pointer_value: u64,
        prior_heap_target_ranges: Option<&PointerScanTargetRangeSet>,
    ) -> bool {
        if discovery_depth <= 1 {
            return true;
        }

        prior_heap_target_ranges
            .map(|prior_heap_target_ranges| prior_heap_target_ranges.contains_value_binary(pointer_value))
            .unwrap_or(false)
    }
}
