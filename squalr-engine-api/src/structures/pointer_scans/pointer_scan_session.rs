use crate::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;
use crate::structures::pointer_scans::pointer_scan_level::PointerScanLevel;
use crate::structures::pointer_scans::pointer_scan_level_candidates::PointerScanLevelCandidates;
use crate::structures::pointer_scans::pointer_scan_level_summary::PointerScanLevelSummary;
use crate::structures::pointer_scans::pointer_scan_node::PointerScanNode;
use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use crate::structures::pointer_scans::pointer_scan_summary::PointerScanSummary;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PointerScanSession {
    session_id: u64,
    target_address: u64,
    pointer_size: PointerScanPointerSize,
    max_depth: u64,
    offset_radius: u64,
    root_node_count: u64,
    pointer_scan_levels: Vec<PointerScanLevel>,
    pointer_scan_level_candidates: Vec<PointerScanLevelCandidates>,
    total_node_count: u64,
    total_static_node_count: u64,
    total_heap_node_count: u64,
    expanded_root_node_ids: Vec<u64>,
    expanded_child_node_ids_by_parent_id: HashMap<u64, Vec<u64>>,
    expanded_pointer_scan_nodes: HashMap<u64, PointerScanNode>,
    next_expanded_node_id: u64,
}

impl PointerScanSession {
    pub fn new(
        session_id: u64,
        target_address: u64,
        pointer_size: PointerScanPointerSize,
        max_depth: u64,
        offset_radius: u64,
        pointer_scan_levels: Vec<PointerScanLevel>,
        pointer_scan_level_candidates: Vec<PointerScanLevelCandidates>,
        root_node_count: u64,
        total_static_node_count: u64,
        total_heap_node_count: u64,
    ) -> Self {
        let total_node_count = pointer_scan_levels
            .iter()
            .map(PointerScanLevel::get_node_count)
            .sum();

        Self {
            session_id,
            target_address,
            pointer_size,
            max_depth,
            offset_radius,
            root_node_count,
            pointer_scan_levels,
            pointer_scan_level_candidates,
            total_node_count,
            total_static_node_count,
            total_heap_node_count,
            expanded_root_node_ids: Vec::new(),
            expanded_child_node_ids_by_parent_id: HashMap::new(),
            expanded_pointer_scan_nodes: HashMap::new(),
            next_expanded_node_id: 1,
        }
    }

    pub fn get_session_id(&self) -> u64 {
        self.session_id
    }

    pub fn get_target_address(&self) -> u64 {
        self.target_address
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

    pub fn get_pointer_scan_levels(&self) -> &Vec<PointerScanLevel> {
        &self.pointer_scan_levels
    }

    pub fn get_pointer_scan_level_candidates(&self) -> &Vec<PointerScanLevelCandidates> {
        &self.pointer_scan_level_candidates
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
            self.target_address,
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

    pub fn get_expanded_nodes(
        &mut self,
        parent_node_id: Option<u64>,
    ) -> Vec<PointerScanNode> {
        let expanded_node_ids = match parent_node_id {
            Some(parent_node_id) => match self.expanded_child_node_ids_by_parent_id.get(&parent_node_id) {
                Some(expanded_node_ids) => expanded_node_ids.clone(),
                None => self.materialize_expanded_nodes_for_parent(Some(parent_node_id)),
            },
            None => {
                if self.expanded_root_node_ids.is_empty() {
                    self.materialize_expanded_nodes_for_parent(None)
                } else {
                    self.expanded_root_node_ids.clone()
                }
            }
        };

        expanded_node_ids
            .into_iter()
            .filter_map(|expanded_node_id| self.find_expanded_node(expanded_node_id).cloned())
            .collect()
    }

    fn materialize_expanded_nodes_for_parent(
        &mut self,
        parent_node_id: Option<u64>,
    ) -> Vec<u64> {
        let expanded_node_ids = match parent_node_id {
            Some(parent_node_id) => self.materialize_child_nodes_for_parent(parent_node_id),
            None => self.materialize_root_nodes(),
        };

        match parent_node_id {
            Some(parent_node_id) => {
                self.expanded_child_node_ids_by_parent_id
                    .insert(parent_node_id, expanded_node_ids.clone());
            }
            None => {
                self.expanded_root_node_ids = expanded_node_ids.clone();
            }
        }

        expanded_node_ids
    }

    fn materialize_root_nodes(&mut self) -> Vec<u64> {
        let mut expanded_node_ids = Vec::new();
        let static_candidates = self
            .pointer_scan_level_candidates
            .iter()
            .rev()
            .flat_map(|pointer_scan_level_candidates| {
                pointer_scan_level_candidates
                    .get_static_candidates()
                    .iter()
                    .cloned()
            })
            .collect::<Vec<_>>();

        for static_candidate in &static_candidates {
            expanded_node_ids.extend(self.materialize_display_nodes_for_candidate(static_candidate, 1, None));
        }

        expanded_node_ids
    }

    fn materialize_child_nodes_for_parent(
        &mut self,
        parent_node_id: u64,
    ) -> Vec<u64> {
        let Some(parent_expanded_node) = self.find_expanded_node(parent_node_id).cloned() else {
            return Vec::new();
        };
        let child_discovery_depth = parent_expanded_node.get_discovery_depth().saturating_sub(1);

        if child_discovery_depth == 0 {
            return Vec::new();
        }

        let Some(pointer_scan_level_candidates) = self.find_level_candidates(child_discovery_depth) else {
            return Vec::new();
        };
        let Some(heap_candidate) = pointer_scan_level_candidates
            .find_heap_candidate_by_address(parent_expanded_node.get_resolved_target_address())
            .cloned()
        else {
            return Vec::new();
        };

        self.materialize_display_nodes_for_candidate(&heap_candidate, parent_expanded_node.get_depth().saturating_add(1), Some(parent_node_id))
    }

    fn materialize_display_nodes_for_candidate(
        &mut self,
        pointer_scan_candidate: &PointerScanCandidate,
        display_depth: u64,
        parent_node_id: Option<u64>,
    ) -> Vec<u64> {
        if pointer_scan_candidate.get_discovery_depth() <= 1 {
            let Some(pointer_offset) = Self::calculate_pointer_offset(self.target_address, pointer_scan_candidate.get_pointer_value()) else {
                return Vec::new();
            };
            let expanded_pointer_scan_node = self.create_materialized_pointer_scan_node(
                pointer_scan_candidate,
                self.target_address,
                pointer_offset,
                false,
                display_depth,
                parent_node_id,
            );

            return vec![expanded_pointer_scan_node];
        }

        let lower_bound = pointer_scan_candidate
            .get_pointer_value()
            .saturating_sub(self.offset_radius);
        let upper_bound = pointer_scan_candidate
            .get_pointer_value()
            .saturating_add(self.offset_radius);
        let Some(next_pointer_scan_level_candidates) = self.find_level_candidates(pointer_scan_candidate.get_discovery_depth().saturating_sub(1)) else {
            return Vec::new();
        };
        let child_target_addresses = next_pointer_scan_level_candidates
            .find_heap_candidates_in_range(lower_bound, upper_bound)
            .iter()
            .map(PointerScanCandidate::get_pointer_address)
            .collect::<Vec<_>>();

        child_target_addresses
            .iter()
            .filter_map(|child_target_address| {
                let pointer_offset = Self::calculate_pointer_offset(*child_target_address, pointer_scan_candidate.get_pointer_value())?;

                Some(self.create_materialized_pointer_scan_node(
                    pointer_scan_candidate,
                    *child_target_address,
                    pointer_offset,
                    true,
                    display_depth,
                    parent_node_id,
                ))
            })
            .collect()
    }

    fn create_materialized_pointer_scan_node(
        &mut self,
        pointer_scan_candidate: &PointerScanCandidate,
        resolved_target_address: u64,
        pointer_offset: i64,
        has_children: bool,
        display_depth: u64,
        parent_node_id: Option<u64>,
    ) -> u64 {
        let expanded_node_id = self.next_expanded_node_id;
        self.next_expanded_node_id = self.next_expanded_node_id.saturating_add(1);
        let expanded_pointer_scan_node = PointerScanNode::new_materialized(
            expanded_node_id,
            pointer_scan_candidate.get_candidate_id(),
            pointer_scan_candidate.get_discovery_depth(),
            display_depth,
            parent_node_id,
            pointer_scan_candidate.get_pointer_scan_node_type(),
            pointer_scan_candidate.get_pointer_address(),
            pointer_scan_candidate.get_pointer_value(),
            resolved_target_address,
            pointer_offset,
            pointer_scan_candidate.get_module_name().to_string(),
            pointer_scan_candidate.get_module_offset(),
            has_children,
        );
        self.expanded_pointer_scan_nodes
            .insert(expanded_node_id, expanded_pointer_scan_node);

        expanded_node_id
    }

    fn find_level_candidates(
        &self,
        discovery_depth: u64,
    ) -> Option<&PointerScanLevelCandidates> {
        discovery_depth
            .checked_sub(1)
            .and_then(|level_index| self.pointer_scan_level_candidates.get(level_index as usize))
    }

    fn find_expanded_node(
        &self,
        node_id: u64,
    ) -> Option<&PointerScanNode> {
        self.expanded_pointer_scan_nodes.get(&node_id)
    }

    fn calculate_pointer_offset(
        target_address: u64,
        pointer_value: u64,
    ) -> Option<i64> {
        let pointer_offset = target_address as i128 - pointer_value as i128;

        i64::try_from(pointer_offset).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::PointerScanSession;
    use crate::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;
    use crate::structures::pointer_scans::pointer_scan_level::PointerScanLevel;
    use crate::structures::pointer_scans::pointer_scan_level_candidates::PointerScanLevelCandidates;
    use crate::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
    use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;

    fn create_pointer_scan_session() -> PointerScanSession {
        PointerScanSession::new(
            7,
            0x3010,
            PointerScanPointerSize::Pointer64,
            4,
            0x100,
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
                        "game.exe".to_string(),
                        0x1000,
                    )],
                    vec![PointerScanCandidate::new(
                        2,
                        1,
                        PointerScanNodeType::Heap,
                        0x2000,
                        0x3000,
                        String::new(),
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
                        "game.exe".to_string(),
                        0x1100,
                    )],
                    Vec::new(),
                ),
            ],
            2,
            1,
            1,
        )
    }

    fn create_shared_child_pointer_scan_session() -> PointerScanSession {
        PointerScanSession::new(
            8,
            0x4010,
            PointerScanPointerSize::Pointer64,
            4,
            0x100,
            vec![
                PointerScanLevel::new(1, 1, 0, 1),
                PointerScanLevel::new(2, 2, 2, 0),
            ],
            vec![
                PointerScanLevelCandidates::new(
                    1,
                    Vec::new(),
                    vec![PointerScanCandidate::new(
                        3,
                        1,
                        PointerScanNodeType::Heap,
                        0x3000,
                        0x4000,
                        String::new(),
                        0,
                    )],
                ),
                PointerScanLevelCandidates::new(
                    2,
                    vec![
                        PointerScanCandidate::new(1, 2, PointerScanNodeType::Static, 0x1000, 0x3000, "game.exe".to_string(), 0x1000),
                        PointerScanCandidate::new(2, 2, PointerScanNodeType::Static, 0x1100, 0x3000, "game.exe".to_string(), 0x1100),
                    ],
                    Vec::new(),
                ),
            ],
            2,
            2,
            1,
        )
    }

    #[test]
    fn pointer_scan_session_summary_tracks_level_and_node_counts() {
        let pointer_scan_session = create_pointer_scan_session();
        let pointer_scan_summary = pointer_scan_session.summarize();

        assert_eq!(pointer_scan_summary.get_session_id(), 7);
        assert_eq!(pointer_scan_summary.get_root_node_count(), 2);
        assert_eq!(pointer_scan_summary.get_total_node_count(), 3);
        assert_eq!(pointer_scan_summary.get_total_static_node_count(), 1);
        assert_eq!(pointer_scan_summary.get_total_heap_node_count(), 1);
        assert_eq!(pointer_scan_summary.get_pointer_scan_level_summaries().len(), 2);
    }

    #[test]
    fn pointer_scan_session_expands_root_and_child_nodes() {
        let mut pointer_scan_session = create_pointer_scan_session();

        let root_nodes = pointer_scan_session.get_expanded_nodes(None);

        assert_eq!(root_nodes.len(), 2);
        assert_eq!(root_nodes[0].get_graph_node_id(), 3);
        assert_eq!(root_nodes[0].get_resolved_target_address(), 0x2000);
        assert!(root_nodes[0].has_children());
        assert_eq!(root_nodes[1].get_graph_node_id(), 1);
        assert_eq!(root_nodes[1].get_resolved_target_address(), 0x3010);
        assert!(!root_nodes[1].has_children());

        let child_nodes = pointer_scan_session.get_expanded_nodes(Some(root_nodes[0].get_node_id()));
        assert_eq!(child_nodes.len(), 1);
        assert_eq!(child_nodes[0].get_graph_node_id(), 2);
        assert_eq!(child_nodes[0].get_parent_node_id(), Some(root_nodes[0].get_node_id()));
    }

    #[test]
    fn pointer_scan_session_round_trips_through_json() {
        let pointer_scan_session = create_pointer_scan_session();
        let serialized_session = serde_json::to_string(&pointer_scan_session).expect("Pointer scan session should serialize.");
        let deserialized_session: PointerScanSession = serde_json::from_str(&serialized_session).expect("Pointer scan session should deserialize.");

        assert_eq!(deserialized_session, pointer_scan_session);
    }

    #[test]
    fn pointer_scan_session_materializes_distinct_display_nodes_for_shared_children() {
        let mut pointer_scan_session = create_shared_child_pointer_scan_session();

        let root_nodes = pointer_scan_session.get_expanded_nodes(None);
        let first_child_nodes = pointer_scan_session.get_expanded_nodes(Some(root_nodes[0].get_node_id()));
        let second_child_nodes = pointer_scan_session.get_expanded_nodes(Some(root_nodes[1].get_node_id()));

        assert_eq!(root_nodes.len(), 2);
        assert_eq!(first_child_nodes.len(), 1);
        assert_eq!(second_child_nodes.len(), 1);
        assert_eq!(first_child_nodes[0].get_graph_node_id(), second_child_nodes[0].get_graph_node_id());
        assert_ne!(first_child_nodes[0].get_node_id(), second_child_nodes[0].get_node_id());
        assert_eq!(first_child_nodes[0].get_parent_node_id(), Some(root_nodes[0].get_node_id()));
        assert_eq!(second_child_nodes[0].get_parent_node_id(), Some(root_nodes[1].get_node_id()));
    }
}
