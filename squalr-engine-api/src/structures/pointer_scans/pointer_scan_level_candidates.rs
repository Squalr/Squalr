use crate::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PointerScanLevelCandidates {
    discovery_depth: u64,
    static_candidates: Vec<PointerScanCandidate>,
    heap_candidates: Vec<PointerScanCandidate>,
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
            heap_candidates,
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
            heap_candidates,
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

    pub fn get_heap_candidates(&self) -> &Vec<PointerScanCandidate> {
        &self.heap_candidates
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
        self.ensure_heap_candidates_sorted();
        self.heap_candidates
            .binary_search_by_key(&pointer_address, PointerScanCandidate::get_pointer_address)
            .ok()
            .and_then(|candidate_index| self.heap_candidates.get(candidate_index))
    }

    pub fn find_heap_candidates_in_range(
        &mut self,
        lower_bound: u64,
        upper_bound: u64,
    ) -> &[PointerScanCandidate] {
        self.ensure_heap_candidates_sorted();
        let start_index = self
            .heap_candidates
            .partition_point(|candidate| candidate.get_pointer_address() < lower_bound);
        let end_index = self
            .heap_candidates
            .partition_point(|candidate| candidate.get_pointer_address() <= upper_bound);

        &self.heap_candidates[start_index..end_index]
    }

    fn ensure_heap_candidates_sorted(&mut self) {
        if self.heap_candidates_sorted {
            return;
        }

        self.heap_candidates
            .sort_unstable_by_key(PointerScanCandidate::get_pointer_address);
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
}
