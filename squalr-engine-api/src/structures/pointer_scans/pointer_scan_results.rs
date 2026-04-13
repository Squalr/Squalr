use crate::structures::pointer_scans::pointer_scan_address_space::PointerScanAddressSpace;
use crate::structures::pointer_scans::pointer_scan_level::PointerScanLevel;
use crate::structures::pointer_scans::pointer_scan_level_candidates::PointerScanLevelCandidates;
use crate::structures::pointer_scans::pointer_scan_level_summary::PointerScanLevelSummary;
use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use crate::structures::pointer_scans::pointer_scan_summary::PointerScanSummary;
use crate::structures::pointer_scans::pointer_scan_target_descriptor::PointerScanTargetDescriptor;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct PointerScanResults {
    session_id: u64,
    target_descriptor: PointerScanTargetDescriptor,
    target_addresses: Vec<u64>,
    #[serde(default)]
    address_space: PointerScanAddressSpace,
    pointer_size: PointerScanPointerSize,
    max_depth: u64,
    offset_radius: u64,
    root_node_count: u64,
    module_names: Vec<String>,
    pointer_scan_levels: Vec<PointerScanLevel>,
    pointer_scan_level_candidates: Vec<PointerScanLevelCandidates>,
    total_node_count: u64,
    total_static_node_count: u64,
    total_heap_node_count: u64,
}

impl PointerScanResults {
    pub fn new(
        session_id: u64,
        target_descriptor: PointerScanTargetDescriptor,
        mut target_addresses: Vec<u64>,
        address_space: PointerScanAddressSpace,
        pointer_size: PointerScanPointerSize,
        max_depth: u64,
        offset_radius: u64,
        module_names: Vec<String>,
        pointer_scan_levels: Vec<PointerScanLevel>,
        pointer_scan_level_candidates: Vec<PointerScanLevelCandidates>,
        total_static_node_count: u64,
        total_heap_node_count: u64,
    ) -> Self {
        target_addresses.sort_unstable();
        target_addresses.dedup();
        let total_node_count = pointer_scan_levels
            .iter()
            .map(PointerScanLevel::get_node_count)
            .sum();
        let root_node_count = Self::calculate_root_node_count(&target_descriptor, &pointer_scan_level_candidates, &target_addresses, offset_radius);

        Self {
            session_id,
            target_descriptor,
            target_addresses,
            address_space,
            pointer_size,
            max_depth,
            offset_radius,
            root_node_count,
            module_names,
            pointer_scan_levels,
            pointer_scan_level_candidates,
            total_node_count,
            total_static_node_count,
            total_heap_node_count,
        }
    }

    pub fn get_session_id(&self) -> u64 {
        self.session_id
    }

    pub fn get_target_descriptor(&self) -> &PointerScanTargetDescriptor {
        &self.target_descriptor
    }

    pub fn get_target_addresses(&self) -> &Vec<u64> {
        &self.target_addresses
    }

    pub fn get_address_space(&self) -> PointerScanAddressSpace {
        self.address_space
    }

    pub fn get_pointer_size(&self) -> PointerScanPointerSize {
        self.pointer_size
    }

    pub fn get_max_depth(&self) -> u64 {
        self.max_depth
    }

    pub fn get_offset_radius(&self) -> u64 {
        self.offset_radius
    }

    pub fn get_root_node_count(&self) -> u64 {
        self.root_node_count
    }

    pub fn get_module_name(
        &self,
        module_index: usize,
    ) -> Option<&str> {
        self.module_names.get(module_index).map(String::as_str)
    }

    pub fn get_module_names(&self) -> &Vec<String> {
        &self.module_names
    }

    pub fn get_module_index_by_name(
        &self,
        module_name: &str,
    ) -> Option<usize> {
        self.module_names
            .iter()
            .position(|candidate_module_name| candidate_module_name == module_name)
    }

    pub fn get_pointer_scan_levels(&self) -> &Vec<PointerScanLevel> {
        &self.pointer_scan_levels
    }

    pub fn get_pointer_scan_level_candidates(&self) -> &Vec<PointerScanLevelCandidates> {
        &self.pointer_scan_level_candidates
    }

    /// Gets mutable level candidates for privileged-side materialization.
    pub fn get_pointer_scan_level_candidates_mut(&mut self) -> &mut Vec<PointerScanLevelCandidates> {
        &mut self.pointer_scan_level_candidates
    }

    pub fn get_total_node_count(&self) -> u64 {
        self.total_node_count
    }

    pub fn get_total_static_node_count(&self) -> u64 {
        self.total_static_node_count
    }

    pub fn get_total_heap_node_count(&self) -> u64 {
        self.total_heap_node_count
    }

    pub fn summarize(&self) -> PointerScanSummary {
        let pointer_scan_level_summaries = self
            .pointer_scan_levels
            .iter()
            .map(|pointer_scan_level| {
                PointerScanLevelSummary::new(
                    pointer_scan_level.get_depth(),
                    pointer_scan_level.get_node_count(),
                    pointer_scan_level.get_static_node_count(),
                    pointer_scan_level.get_heap_node_count(),
                )
            })
            .collect();

        PointerScanSummary::new(
            self.session_id,
            self.target_descriptor.clone(),
            self.address_space,
            self.pointer_size,
            self.max_depth,
            self.offset_radius,
            self.root_node_count,
            self.total_node_count,
            self.total_static_node_count,
            self.total_heap_node_count,
            pointer_scan_level_summaries,
        )
    }

    fn calculate_root_node_count(
        target_descriptor: &PointerScanTargetDescriptor,
        pointer_scan_level_candidates: &[PointerScanLevelCandidates],
        target_addresses: &[u64],
        offset_radius: u64,
    ) -> u64 {
        if !matches!(target_descriptor, PointerScanTargetDescriptor::Value { .. }) {
            return pointer_scan_level_candidates
                .iter()
                .map(PointerScanLevelCandidates::get_static_node_count)
                .sum();
        }

        pointer_scan_level_candidates
            .iter()
            .flat_map(|pointer_scan_level_candidates| pointer_scan_level_candidates.get_static_candidates().iter())
            .map(|static_candidate| {
                if static_candidate.get_discovery_depth() > 1 {
                    1
                } else {
                    Self::count_target_addresses_in_range(
                        target_addresses,
                        static_candidate
                            .get_pointer_value()
                            .saturating_sub(offset_radius),
                        static_candidate
                            .get_pointer_value()
                            .saturating_add(offset_radius),
                    )
                }
            })
            .sum()
    }

    fn count_target_addresses_in_range(
        target_addresses: &[u64],
        lower_bound: u64,
        upper_bound: u64,
    ) -> u64 {
        let start_index = target_addresses.partition_point(|target_address| *target_address < lower_bound);
        let end_index = target_addresses.partition_point(|target_address| *target_address <= upper_bound);

        end_index.saturating_sub(start_index) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::PointerScanResults;
    use crate::structures::pointer_scans::pointer_scan_address_space::PointerScanAddressSpace;
    use crate::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;
    use crate::structures::pointer_scans::pointer_scan_level::PointerScanLevel;
    use crate::structures::pointer_scans::pointer_scan_level_candidates::PointerScanLevelCandidates;
    use crate::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
    use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
    use crate::structures::pointer_scans::pointer_scan_target_descriptor::PointerScanTargetDescriptor;

    fn create_pointer_scan_results() -> PointerScanResults {
        PointerScanResults::new(
            7,
            PointerScanTargetDescriptor::address(0x3010),
            vec![0x3010],
            PointerScanAddressSpace::EmulatorMemory,
            PointerScanPointerSize::Pointer64,
            4,
            0x100,
            vec!["game.exe".to_string()],
            vec![
                PointerScanLevel::new(1, 2, 1, 1),
                PointerScanLevel::new(2, 1, 1, 0),
            ],
            vec![
                PointerScanLevelCandidates::new(
                    1,
                    vec![PointerScanCandidate::new(
                        1,
                        1,
                        PointerScanNodeType::Static,
                        0x1000,
                        0x2000,
                        0,
                        0x1000,
                    )],
                    vec![PointerScanCandidate::new(
                        2,
                        1,
                        PointerScanNodeType::Heap,
                        0x2000,
                        0x3000,
                        0,
                        0,
                    )],
                ),
                PointerScanLevelCandidates::new(
                    2,
                    vec![PointerScanCandidate::new(
                        3,
                        2,
                        PointerScanNodeType::Static,
                        0x1100,
                        0x2000,
                        0,
                        0x1100,
                    )],
                    Vec::new(),
                ),
            ],
            2,
            1,
        )
    }

    fn create_multi_target_leaf_pointer_scan_results() -> PointerScanResults {
        PointerScanResults::new(
            9,
            PointerScanTargetDescriptor::value(
                crate::structures::data_values::anonymous_value_string::AnonymousValueString::new(
                    "123".to_string(),
                    crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat::Decimal,
                    crate::structures::data_values::container_type::ContainerType::None,
                ),
                crate::structures::data_types::data_type_ref::DataTypeRef::new("u32"),
                2,
            ),
            vec![0x3010, 0x3020],
            PointerScanAddressSpace::EmulatorMemory,
            PointerScanPointerSize::Pointer64,
            1,
            0x20,
            vec!["game.exe".to_string()],
            vec![PointerScanLevel::new(1, 1, 1, 0)],
            vec![PointerScanLevelCandidates::new(
                1,
                vec![PointerScanCandidate::new(
                    1,
                    1,
                    PointerScanNodeType::Static,
                    0x1010,
                    0x3000,
                    0,
                    0x10,
                )],
                Vec::new(),
            )],
            1,
            0,
        )
    }

    #[test]
    fn pointer_scan_results_summary_tracks_level_and_node_counts() {
        let pointer_scan_results = create_pointer_scan_results();
        let pointer_scan_summary = pointer_scan_results.summarize();

        assert_eq!(pointer_scan_summary.get_session_id(), 7);
        assert_eq!(pointer_scan_summary.get_root_node_count(), 2);
        assert_eq!(pointer_scan_summary.get_total_node_count(), 3);
        assert_eq!(pointer_scan_summary.get_total_static_node_count(), 2);
        assert_eq!(pointer_scan_summary.get_total_heap_node_count(), 1);
        assert_eq!(pointer_scan_summary.get_pointer_scan_level_summaries().len(), 2);
    }

    #[test]
    fn pointer_scan_results_round_trip_through_json() {
        let pointer_scan_results = create_pointer_scan_results();
        let serialized_results = serde_json::to_string(&pointer_scan_results).expect("Pointer scan results should serialize.");
        let deserialized_results: PointerScanResults = serde_json::from_str(&serialized_results).expect("Pointer scan results should deserialize.");

        assert_eq!(deserialized_results, pointer_scan_results);
    }

    #[test]
    fn pointer_scan_results_count_multi_target_roots_without_materializer_state() {
        let pointer_scan_results = create_multi_target_leaf_pointer_scan_results();

        assert_eq!(pointer_scan_results.get_root_node_count(), 2);
        assert_eq!(pointer_scan_results.get_total_node_count(), 1);
    }
}
