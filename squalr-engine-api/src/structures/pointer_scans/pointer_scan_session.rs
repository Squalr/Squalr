use crate::structures::pointer_scans::pointer_scan_level::PointerScanLevel;
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
    root_node_ids: Vec<u64>,
    pointer_scan_levels: Vec<PointerScanLevel>,
    pointer_scan_nodes: Vec<PointerScanNode>,
    total_node_count: u64,
    total_static_node_count: u64,
    total_heap_node_count: u64,
    expanded_root_node_ids: Vec<u64>,
    expanded_child_node_ids_by_parent_id: HashMap<u64, Vec<u64>>,
    expanded_pointer_scan_nodes: Vec<PointerScanNode>,
    next_expanded_node_id: u64,
}

impl PointerScanSession {
    pub fn new(
        session_id: u64,
        target_address: u64,
        pointer_size: PointerScanPointerSize,
        max_depth: u64,
        offset_radius: u64,
        root_node_ids: Vec<u64>,
        pointer_scan_levels: Vec<PointerScanLevel>,
        pointer_scan_nodes: Vec<PointerScanNode>,
        total_static_node_count: u64,
        total_heap_node_count: u64,
    ) -> Self {
        let total_node_count = pointer_scan_nodes.len() as u64;
        let next_expanded_node_id = total_node_count.saturating_add(1);

        Self {
            session_id,
            target_address,
            pointer_size,
            max_depth,
            offset_radius,
            root_node_ids,
            pointer_scan_levels,
            pointer_scan_nodes,
            total_node_count,
            total_static_node_count,
            total_heap_node_count,
            expanded_root_node_ids: Vec::new(),
            expanded_child_node_ids_by_parent_id: HashMap::new(),
            expanded_pointer_scan_nodes: Vec::new(),
            next_expanded_node_id,
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

    pub fn get_root_node_ids(&self) -> &Vec<u64> {
        &self.root_node_ids
    }

    pub fn get_pointer_scan_levels(&self) -> &Vec<PointerScanLevel> {
        &self.pointer_scan_levels
    }

    pub fn get_pointer_scan_nodes(&self) -> &Vec<PointerScanNode> {
        &self.pointer_scan_nodes
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
            self.root_node_ids.len() as u64,
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
        let graph_child_node_ids = match parent_node_id {
            Some(parent_node_id) => {
                let Some(parent_expanded_node) = self.find_expanded_node(parent_node_id).cloned() else {
                    return Vec::new();
                };
                let Some(parent_graph_node) = self.find_graph_node(parent_expanded_node.get_graph_node_id()) else {
                    return Vec::new();
                };

                parent_graph_node.get_child_node_ids().clone()
            }
            None => self.root_node_ids.clone(),
        };

        let mut expanded_node_ids = Vec::with_capacity(graph_child_node_ids.len());

        for graph_child_node_id in graph_child_node_ids {
            let Some(graph_child_node) = self.find_graph_node(graph_child_node_id).cloned() else {
                continue;
            };
            let expanded_depth = parent_node_id
                .and_then(|parent_node_id| {
                    self.find_expanded_node(parent_node_id)
                        .map(PointerScanNode::get_depth)
                })
                .unwrap_or(0)
                .saturating_add(1);
            let expanded_node_id = self.next_expanded_node_id;
            self.next_expanded_node_id = self.next_expanded_node_id.saturating_add(1);
            let expanded_pointer_scan_node = PointerScanNode::new_materialized(expanded_node_id, expanded_depth, parent_node_id, &graph_child_node);

            expanded_node_ids.push(expanded_node_id);
            self.expanded_pointer_scan_nodes
                .push(expanded_pointer_scan_node);
        }

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

    fn find_graph_node(
        &self,
        node_id: u64,
    ) -> Option<&PointerScanNode> {
        self.pointer_scan_nodes
            .binary_search_by_key(&node_id, PointerScanNode::get_node_id)
            .ok()
            .and_then(|node_index| self.pointer_scan_nodes.get(node_index))
    }

    fn find_expanded_node(
        &self,
        node_id: u64,
    ) -> Option<&PointerScanNode> {
        self.expanded_pointer_scan_nodes
            .iter()
            .find(|pointer_scan_node| pointer_scan_node.get_node_id() == node_id)
    }
}

#[cfg(test)]
mod tests {
    use super::PointerScanSession;
    use crate::structures::pointer_scans::pointer_scan_level::PointerScanLevel;
    use crate::structures::pointer_scans::pointer_scan_node::PointerScanNode;
    use crate::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
    use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;

    fn create_pointer_scan_session() -> PointerScanSession {
        let root_node = PointerScanNode::new(
            1,
            None,
            PointerScanNodeType::Static,
            1,
            0x1000,
            0x2000,
            0x2020,
            0x20,
            "game.exe".to_string(),
            0x1000,
            vec![2],
        );
        let child_node = PointerScanNode::new(
            2,
            Some(1),
            PointerScanNodeType::Heap,
            2,
            0x2000,
            0x3000,
            0x3010,
            0x10,
            String::new(),
            0,
            Vec::new(),
        );

        PointerScanSession::new(
            7,
            0x3010,
            PointerScanPointerSize::Pointer64,
            4,
            0x100,
            vec![1],
            vec![
                PointerScanLevel::new(1, vec![1], 1, 0),
                PointerScanLevel::new(2, vec![2], 0, 1),
            ],
            vec![root_node, child_node],
            1,
            1,
        )
    }

    fn create_shared_child_pointer_scan_session() -> PointerScanSession {
        let first_root_node = PointerScanNode::new(
            1,
            None,
            PointerScanNodeType::Static,
            1,
            0x1000,
            0x2000,
            0x3000,
            0x10,
            "game.exe".to_string(),
            0x1000,
            vec![3],
        );
        let second_root_node = PointerScanNode::new(
            2,
            None,
            PointerScanNodeType::Static,
            1,
            0x1100,
            0x2000,
            0x3000,
            0x10,
            "game.exe".to_string(),
            0x1100,
            vec![3],
        );
        let shared_child_node = PointerScanNode::new(
            3,
            None,
            PointerScanNodeType::Heap,
            2,
            0x3000,
            0x4000,
            0x4010,
            0x10,
            String::new(),
            0,
            Vec::new(),
        );

        PointerScanSession::new(
            8,
            0x4010,
            PointerScanPointerSize::Pointer64,
            4,
            0x100,
            vec![1, 2],
            vec![
                PointerScanLevel::new(1, vec![1, 2], 2, 0),
                PointerScanLevel::new(2, vec![3], 0, 1),
            ],
            vec![first_root_node, second_root_node, shared_child_node],
            2,
            1,
        )
    }

    #[test]
    fn pointer_scan_session_summary_tracks_level_and_node_counts() {
        let pointer_scan_session = create_pointer_scan_session();
        let pointer_scan_summary = pointer_scan_session.summarize();

        assert_eq!(pointer_scan_summary.get_session_id(), 7);
        assert_eq!(pointer_scan_summary.get_root_node_count(), 1);
        assert_eq!(pointer_scan_summary.get_total_node_count(), 2);
        assert_eq!(pointer_scan_summary.get_total_static_node_count(), 1);
        assert_eq!(pointer_scan_summary.get_total_heap_node_count(), 1);
        assert_eq!(pointer_scan_summary.get_pointer_scan_level_summaries().len(), 2);
    }

    #[test]
    fn pointer_scan_session_expands_root_and_child_nodes() {
        let mut pointer_scan_session = create_pointer_scan_session();

        let root_nodes = pointer_scan_session.get_expanded_nodes(None);
        let child_nodes = pointer_scan_session.get_expanded_nodes(Some(root_nodes[0].get_node_id()));

        assert_eq!(root_nodes.len(), 1);
        assert_eq!(root_nodes[0].get_graph_node_id(), 1);
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
