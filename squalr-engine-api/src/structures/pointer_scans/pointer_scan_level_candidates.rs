use super::{heap_candidates::HeapCandidates, pointer_scan_heap_slot::PointerScanHeapSlot};
use crate::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;
use serde::{Deserialize, Serialize};
use std::ops::Range;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PointerScanLevelCandidates {
    discovery_depth: u64,
    static_candidates: Vec<PointerScanCandidate>,
    heap_candidates: HeapCandidates,
    #[serde(skip, default)]
    heap_candidates_sorted: bool,
}

impl PointerScanLevelCandidates {
    pub fn new(
        discovery_depth: u64,
        mut static_candidates: Vec<PointerScanCandidate>,
        heap_candidates: Vec<PointerScanCandidate>,
    ) -> Self {
        static_candidates
            .sort_unstable_by_key(|pointer_scan_candidate| (pointer_scan_candidate.get_module_index(), pointer_scan_candidate.get_module_offset()));

        Self {
            discovery_depth,
            static_candidates,
            heap_candidates: heap_candidates.into(),
            heap_candidates_sorted: false,
        }
    }

    pub fn new_presorted(
        discovery_depth: u64,
        static_candidates: Vec<PointerScanCandidate>,
        heap_candidates: Vec<PointerScanCandidate>,
    ) -> Self {
        Self {
            discovery_depth,
            static_candidates,
            heap_candidates: heap_candidates.into(),
            heap_candidates_sorted: true,
        }
    }

    /// Builds a level from scanner output, with heap slots in ascending address order.
    pub fn with_sorted_heap_slots(
        discovery_depth: u64,
        static_candidates: Vec<PointerScanCandidate>,
        first_heap_id: u64,
        heap_slots: Vec<PointerScanHeapSlot>,
    ) -> Self {
        Self {
            discovery_depth,
            static_candidates,
            heap_candidates: HeapCandidates::packed(discovery_depth, first_heap_id, heap_slots),
            heap_candidates_sorted: true,
        }
    }

    pub fn get_discovery_depth(&self) -> u64 {
        self.discovery_depth
    }

    pub fn get_static_candidates(&self) -> &Vec<PointerScanCandidate> {
        &self.static_candidates
    }

    pub fn contains_static_candidate(
        &self,
        module_index: usize,
        module_offset: u64,
    ) -> bool {
        self.static_candidates
            .binary_search_by_key(&(module_index, module_offset), |pointer_scan_candidate| {
                (pointer_scan_candidate.get_module_index(), pointer_scan_candidate.get_module_offset())
            })
            .is_ok()
    }

    /// Materializes full heap records on first use. Use the iterator for allocation-free inspection.
    pub fn get_heap_candidates(&self) -> &Vec<PointerScanCandidate> {
        self.heap_candidates.full_records()
    }

    pub fn iter_heap_candidates(&self) -> impl ExactSizeIterator<Item = PointerScanCandidate> + '_ {
        (0..self.heap_candidates.len()).map(|slot_index| self.heap_candidates.candidate(slot_index))
    }

    pub fn get_node_count(&self) -> u64 {
        self.get_static_node_count()
            .saturating_add(self.get_heap_node_count())
    }

    pub fn get_static_node_count(&self) -> u64 {
        self.static_candidates.len() as u64
    }

    pub fn get_heap_node_count(&self) -> u64 {
        self.heap_candidates.len() as u64
    }

    pub fn find_heap_candidate_by_address(
        &mut self,
        pointer_address: u64,
    ) -> Option<&PointerScanCandidate> {
        let slot_index = self
            .heap_candidate_range(pointer_address, pointer_address)
            .next_back()?;
        self.heap_candidates.full_records().get(slot_index)
    }

    pub fn find_heap_candidates_in_range(
        &mut self,
        lower_bound: u64,
        upper_bound: u64,
    ) -> &[PointerScanCandidate] {
        let range = self.heap_candidate_range(lower_bound, upper_bound);
        &self.heap_candidates.full_records()[range]
    }

    pub(super) fn heap_candidate_at_address(
        &mut self,
        pointer_address: u64,
    ) -> Option<PointerScanCandidate> {
        let slot_index = self
            .heap_candidate_range(pointer_address, pointer_address)
            .next_back()?;
        Some(self.heap_candidates.candidate(slot_index))
    }

    pub(super) fn heap_candidate_range(
        &mut self,
        lower_bound: u64,
        upper_bound: u64,
    ) -> Range<usize> {
        self.ensure_heap_candidates_sorted();
        self.heap_candidates.address_range(lower_bound, upper_bound)
    }

    pub(super) fn heap_address(
        &self,
        slot_index: usize,
    ) -> u64 {
        self.heap_candidates.address(slot_index)
    }

    pub(super) fn heap_slot_value(
        &self,
        slot_index: usize,
    ) -> u64 {
        self.heap_candidates.candidate(slot_index).get_pointer_value()
    }

    fn ensure_heap_candidates_sorted(&mut self) {
        if self.heap_candidates_sorted {
            return;
        }

        self.heap_candidates.sort_by_address();
        self.heap_candidates_sorted = true;
    }
}

#[cfg(test)]
mod tests {
    use super::PointerScanLevelCandidates;
    use crate::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;
    use crate::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;

    #[test]
    fn new_sorts_static_candidates_for_binary_search() {
        let pointer_scan_level_candidates = PointerScanLevelCandidates::new(
            2,
            vec![
                PointerScanCandidate::new(2, 2, PointerScanNodeType::Static, 0x1200, 0x3000, 1, 0x40),
                PointerScanCandidate::new(1, 2, PointerScanNodeType::Static, 0x1100, 0x2000, 0, 0x10),
                PointerScanCandidate::new(3, 2, PointerScanNodeType::Static, 0x1300, 0x4000, 0, 0x30),
            ],
            Vec::new(),
        );

        assert!(pointer_scan_level_candidates.contains_static_candidate(0, 0x10));
        assert!(pointer_scan_level_candidates.contains_static_candidate(0, 0x30));
        assert!(pointer_scan_level_candidates.contains_static_candidate(1, 0x40));
        assert!(!pointer_scan_level_candidates.contains_static_candidate(0, 0x20));
    }

    #[test]
    fn packed_heap_keeps_wire_records_and_does_not_expand_for_browsing() -> Result<(), serde_json::Error> {
        use super::{HeapCandidates, PointerScanHeapSlot};
        assert_eq!(std::mem::size_of::<PointerScanHeapSlot>(), 16);
        let slots = vec![
            PointerScanHeapSlot {
                pointer_address: 0x2000,
                pointer_value: 0x3000,
            },
            PointerScanHeapSlot {
                pointer_address: 0x2018,
                pointer_value: 0x4000,
            },
        ];
        let mut level = PointerScanLevelCandidates::with_sorted_heap_slots(2, Vec::new(), 41, slots);
        let expected = vec![
            PointerScanCandidate::new(41, 2, PointerScanNodeType::Heap, 0x2000, 0x3000, 0, 0),
            PointerScanCandidate::new(42, 2, PointerScanNodeType::Heap, 0x2018, 0x4000, 0, 0),
        ];
        assert_eq!(level.heap_candidate_range(0x2000, 0x2018), 0..2);
        assert_eq!(level.heap_candidate_at_address(0x2018).as_ref(), expected.get(1));
        assert_eq!(level.heap_candidate_at_address(0x2008), None);
        assert_eq!(level.iter_heap_candidates().collect::<Vec<_>>(), expected);
        let encoded = serde_json::to_value(&level)?;
        assert_eq!(encoded["heap_candidates"], serde_json::to_value(&expected)?);
        let restored: PointerScanLevelCandidates = serde_json::from_value(encoded)?;
        assert!(matches!(&level.heap_candidates, HeapCandidates::Packed {expanded,..} if expanded.get().is_none()));
        assert!(matches!(&restored.heap_candidates, HeapCandidates::Packed {expanded,..} if expanded.get().is_none()));
        assert_eq!(restored.get_heap_candidates(), &expected);
        Ok(())
    }

    #[test]
    fn imported_unsorted_heap_keeps_original_ids_after_range_lookup() -> Result<(), serde_json::Error> {
        let expected = vec![
            PointerScanCandidate::new(9, 1, PointerScanNodeType::Heap, 0x3000, 0x5000, 0, 0),
            PointerScanCandidate::new(17, 1, PointerScanNodeType::Heap, 0x2000, 0x4000, 0, 0),
        ];
        let mut level: PointerScanLevelCandidates = serde_json::from_value(serde_json::json!({
            "discovery_depth": 1, "static_candidates": [], "heap_candidates": expected
        }))?;
        assert_eq!(level.get_heap_candidates(), &expected);
        assert_eq!(level.heap_candidate_at_address(0x2000).as_ref(), expected.get(1));
        assert_eq!(level.find_heap_candidates_in_range(0x3000, 0x3000), &expected[..1]);
        Ok(())
    }
}
