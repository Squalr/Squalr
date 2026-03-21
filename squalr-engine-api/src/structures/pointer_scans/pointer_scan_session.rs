use crate::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;
use crate::structures::pointer_scans::pointer_scan_level::PointerScanLevel;
use crate::structures::pointer_scans::pointer_scan_level_candidates::PointerScanLevelCandidates;
use crate::structures::pointer_scans::pointer_scan_level_summary::PointerScanLevelSummary;
use crate::structures::pointer_scans::pointer_scan_node::PointerScanNode;
use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use crate::structures::pointer_scans::pointer_scan_summary::PointerScanSummary;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
struct MaterializedPointerScanPageKey {
    parent_node_id: Option<u64>,
    page_index: u64,
    page_size: u64,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct MaterializedPointerScanNodePage {
    page_index: u64,
    last_page_index: u64,
    total_node_count: u64,
    node_ids: Vec<u64>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct PointerScanSession {
    session_id: u64,
    target_address: u64,
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
    materialized_node_ids_by_page_key: HashMap<MaterializedPointerScanPageKey, Vec<u64>>,
    materialized_pointer_scan_nodes: HashMap<u64, PointerScanNode>,
    next_materialized_node_id: u64,
}

impl PointerScanSession {
    pub fn new(
        session_id: u64,
        target_address: u64,
        pointer_size: PointerScanPointerSize,
        max_depth: u64,
        offset_radius: u64,
        module_names: Vec<String>,
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
            module_names,
            pointer_scan_levels,
            pointer_scan_level_candidates,
            total_node_count,
            total_static_node_count,
            total_heap_node_count,
            materialized_node_ids_by_page_key: HashMap::new(),
            materialized_pointer_scan_nodes: HashMap::new(),
            next_materialized_node_id: 1,
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

    pub fn get_expanded_node_page(
        &mut self,
        parent_node_id: Option<u64>,
        page_index: u64,
        page_size: u64,
    ) -> (Vec<PointerScanNode>, u64, u64, u64) {
        let bounded_page_size = page_size.max(1);
        let requested_page = match parent_node_id {
            Some(parent_node_id) => self.materialize_child_node_page(parent_node_id, page_index, bounded_page_size),
            None => self.materialize_root_node_page(page_index, bounded_page_size),
        };

        let pointer_scan_nodes = requested_page
            .node_ids
            .iter()
            .filter_map(|node_id| self.find_materialized_node(*node_id).cloned())
            .collect::<Vec<_>>();

        (
            pointer_scan_nodes,
            requested_page.page_index,
            requested_page.last_page_index,
            requested_page.total_node_count,
        )
    }

    pub fn get_expanded_nodes(
        &mut self,
        parent_node_id: Option<u64>,
    ) -> Vec<PointerScanNode> {
        self.get_expanded_node_page(parent_node_id, 0, u64::MAX).0
    }

    fn materialize_root_node_page(
        &mut self,
        page_index: u64,
        page_size: u64,
    ) -> MaterializedPointerScanNodePage {
        let total_node_count = self.root_node_count;
        let last_page_index = Self::calculate_last_page_index(total_node_count, page_size);
        let bounded_page_index = page_index.clamp(0, last_page_index);
        let page_key = MaterializedPointerScanPageKey {
            parent_node_id: None,
            page_index: bounded_page_index,
            page_size,
        };

        if let Some(materialized_node_ids) = self.materialized_node_ids_by_page_key.get(&page_key) {
            return MaterializedPointerScanNodePage {
                page_index: bounded_page_index,
                last_page_index,
                total_node_count,
                node_ids: materialized_node_ids.clone(),
            };
        }

        let page_start_index = bounded_page_index.saturating_mul(page_size);
        let page_node_count = total_node_count.saturating_sub(page_start_index).min(page_size);
        let root_page_candidates = self
            .pointer_scan_level_candidates
            .iter()
            .flat_map(|pointer_scan_level_candidates| pointer_scan_level_candidates.get_static_candidates().iter())
            .skip(page_start_index as usize)
            .take(page_node_count as usize)
            .map(|static_candidate| {
                (
                    static_candidate.get_candidate_id(),
                    static_candidate.get_discovery_depth(),
                    static_candidate.get_pointer_scan_node_type(),
                    static_candidate.get_pointer_address(),
                    static_candidate.get_pointer_value(),
                    static_candidate.get_module_index(),
                    static_candidate.get_module_offset(),
                )
            })
            .collect::<Vec<_>>();
        let mut materialized_node_ids = Vec::with_capacity(root_page_candidates.len());

        for (candidate_id, discovery_depth, pointer_scan_node_type, pointer_address, pointer_value, module_index, module_offset) in root_page_candidates {
            let Some(materialized_root_node_id) = self.materialize_root_pointer_scan_node(
                candidate_id,
                discovery_depth,
                pointer_scan_node_type,
                pointer_address,
                pointer_value,
                module_index,
                module_offset,
            ) else {
                continue;
            };

            materialized_node_ids.push(materialized_root_node_id);
        }

        self.materialized_node_ids_by_page_key
            .insert(page_key, materialized_node_ids.clone());

        MaterializedPointerScanNodePage {
            page_index: bounded_page_index,
            last_page_index,
            total_node_count,
            node_ids: materialized_node_ids,
        }
    }

    fn materialize_child_node_page(
        &mut self,
        parent_node_id: u64,
        page_index: u64,
        page_size: u64,
    ) -> MaterializedPointerScanNodePage {
        let Some(parent_materialized_node) = self.find_materialized_node(parent_node_id).cloned() else {
            return MaterializedPointerScanNodePage::default();
        };

        if parent_materialized_node.get_parent_node_id().is_none() {
            return self.materialize_root_child_node_page(parent_node_id, &parent_materialized_node, page_index, page_size);
        }

        let child_discovery_depth = parent_materialized_node.get_discovery_depth().saturating_sub(1);

        if child_discovery_depth == 0 {
            return MaterializedPointerScanNodePage::default();
        }

        let Some(heap_candidate) = self
            .find_level_candidates_mut(child_discovery_depth)
            .and_then(|pointer_scan_level_candidates| {
                pointer_scan_level_candidates
                    .find_heap_candidate_by_address(parent_materialized_node.get_resolved_target_address())
                    .cloned()
            })
        else {
            return MaterializedPointerScanNodePage::default();
        };
        let total_node_count = self.count_display_nodes_for_pointer_value(heap_candidate.get_discovery_depth(), heap_candidate.get_pointer_value());
        let last_page_index = Self::calculate_last_page_index(total_node_count, page_size);
        let bounded_page_index = page_index.clamp(0, last_page_index);
        let page_key = MaterializedPointerScanPageKey {
            parent_node_id: Some(parent_node_id),
            page_index: bounded_page_index,
            page_size,
        };

        if let Some(materialized_node_ids) = self.materialized_node_ids_by_page_key.get(&page_key) {
            return MaterializedPointerScanNodePage {
                page_index: bounded_page_index,
                last_page_index,
                total_node_count,
                node_ids: materialized_node_ids.clone(),
            };
        }

        let page_start_index = bounded_page_index.saturating_mul(page_size);
        let page_node_count = total_node_count.saturating_sub(page_start_index).min(page_size);
        let materialized_node_ids = self.materialize_display_node_page(
            heap_candidate.get_candidate_id(),
            child_discovery_depth,
            parent_materialized_node.get_branch_total_depth(),
            heap_candidate.get_pointer_scan_node_type(),
            heap_candidate.get_pointer_address(),
            heap_candidate.get_pointer_value(),
            "",
            heap_candidate.get_module_offset(),
            parent_materialized_node.get_depth().saturating_add(1),
            Some(parent_node_id),
            page_start_index,
            page_node_count,
        );

        self.materialized_node_ids_by_page_key
            .insert(page_key, materialized_node_ids.clone());

        MaterializedPointerScanNodePage {
            page_index: bounded_page_index,
            last_page_index,
            total_node_count,
            node_ids: materialized_node_ids,
        }
    }

    fn materialize_root_child_node_page(
        &mut self,
        parent_node_id: u64,
        parent_materialized_node: &PointerScanNode,
        page_index: u64,
        page_size: u64,
    ) -> MaterializedPointerScanNodePage {
        if !parent_materialized_node.has_children() {
            return MaterializedPointerScanNodePage::default();
        }

        let child_discovery_depth = parent_materialized_node.get_discovery_depth().saturating_sub(1);

        if child_discovery_depth == 0 {
            return MaterializedPointerScanNodePage::default();
        }

        let total_node_count = self.count_display_nodes_for_pointer_value(child_discovery_depth, parent_materialized_node.get_pointer_value());
        let last_page_index = Self::calculate_last_page_index(total_node_count, page_size);
        let bounded_page_index = page_index.clamp(0, last_page_index);
        let page_key = MaterializedPointerScanPageKey {
            parent_node_id: Some(parent_node_id),
            page_index: bounded_page_index,
            page_size,
        };

        if let Some(materialized_node_ids) = self.materialized_node_ids_by_page_key.get(&page_key) {
            return MaterializedPointerScanNodePage {
                page_index: bounded_page_index,
                last_page_index,
                total_node_count,
                node_ids: materialized_node_ids.clone(),
            };
        }

        let page_start_index = bounded_page_index.saturating_mul(page_size);
        let page_node_count = total_node_count.saturating_sub(page_start_index).min(page_size);
        let materialized_node_ids = self.materialize_display_node_page(
            parent_materialized_node.get_graph_node_id(),
            child_discovery_depth,
            parent_materialized_node.get_branch_total_depth(),
            parent_materialized_node.get_pointer_scan_node_type(),
            parent_materialized_node.get_pointer_address(),
            parent_materialized_node.get_pointer_value(),
            parent_materialized_node.get_module_name(),
            parent_materialized_node.get_module_offset(),
            parent_materialized_node.get_depth().saturating_add(1),
            Some(parent_node_id),
            page_start_index,
            page_node_count,
        );

        self.materialized_node_ids_by_page_key
            .insert(page_key, materialized_node_ids.clone());

        MaterializedPointerScanNodePage {
            page_index: bounded_page_index,
            last_page_index,
            total_node_count,
            node_ids: materialized_node_ids,
        }
    }

    fn materialize_display_node_page(
        &mut self,
        candidate_id: u64,
        discovery_depth: u64,
        branch_total_depth: u64,
        pointer_scan_node_type: crate::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType,
        pointer_address: u64,
        pointer_value: u64,
        module_name: &str,
        module_offset: u64,
        display_depth: u64,
        parent_node_id: Option<u64>,
        page_start_index: u64,
        page_node_count: u64,
    ) -> Vec<u64> {
        if page_node_count == 0 {
            return Vec::new();
        }

        if discovery_depth <= 1 {
            if page_start_index > 0 {
                return Vec::new();
            }
            let Some(pointer_offset) = Self::calculate_pointer_offset(self.target_address, pointer_value) else {
                return Vec::new();
            };
            let materialized_pointer_scan_node = self.create_materialized_pointer_scan_node(
                candidate_id,
                discovery_depth,
                branch_total_depth,
                pointer_scan_node_type,
                pointer_address,
                pointer_value,
                module_name,
                module_offset,
                self.target_address,
                pointer_offset,
                false,
                display_depth,
                parent_node_id,
            );

            return vec![materialized_pointer_scan_node];
        }

        let lower_bound = pointer_value.saturating_sub(self.offset_radius);
        let upper_bound = pointer_value.saturating_add(self.offset_radius);
        let Some(next_pointer_scan_level_candidates) = self.find_level_candidates_mut(discovery_depth.saturating_sub(1)) else {
            return Vec::new();
        };
        let matching_child_candidates = next_pointer_scan_level_candidates.find_heap_candidates_in_range(lower_bound, upper_bound);
        let page_start_index = page_start_index as usize;
        let page_end_index = page_start_index
            .saturating_add(page_node_count as usize)
            .min(matching_child_candidates.len());
        let resolved_target_addresses = matching_child_candidates[page_start_index..page_end_index]
            .iter()
            .map(PointerScanCandidate::get_pointer_address)
            .collect::<Vec<_>>();

        resolved_target_addresses
            .iter()
            .filter_map(|resolved_target_address| {
                let pointer_offset = Self::calculate_pointer_offset(*resolved_target_address, pointer_value)?;

                Some(self.create_materialized_pointer_scan_node(
                    candidate_id,
                    discovery_depth,
                    branch_total_depth,
                    pointer_scan_node_type,
                    pointer_address,
                    pointer_value,
                    module_name,
                    module_offset,
                    *resolved_target_address,
                    pointer_offset,
                    true,
                    display_depth,
                    parent_node_id,
                ))
            })
            .collect()
    }

    fn count_display_nodes_for_pointer_value(
        &mut self,
        discovery_depth: u64,
        pointer_value: u64,
    ) -> u64 {
        if discovery_depth <= 1 {
            return 1;
        }

        let lower_bound = pointer_value.saturating_sub(self.offset_radius);
        let upper_bound = pointer_value.saturating_add(self.offset_radius);

        self.find_level_candidates_mut(discovery_depth.saturating_sub(1))
            .map(|next_pointer_scan_level_candidates| {
                next_pointer_scan_level_candidates
                    .find_heap_candidates_in_range(lower_bound, upper_bound)
                    .len() as u64
            })
            .unwrap_or(0)
    }

    fn materialize_root_pointer_scan_node(
        &mut self,
        candidate_id: u64,
        discovery_depth: u64,
        pointer_scan_node_type: crate::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType,
        pointer_address: u64,
        pointer_value: u64,
        module_index: usize,
        module_offset: u64,
    ) -> Option<u64> {
        let module_name = self.get_module_name(module_index)?.to_string();
        let has_children = discovery_depth > 1;
        let (resolved_target_address, pointer_offset) = if has_children {
            (pointer_value, 0)
        } else {
            let pointer_offset = Self::calculate_pointer_offset(self.target_address, pointer_value)?;

            (self.target_address, pointer_offset)
        };

        Some(self.create_materialized_pointer_scan_node(
            candidate_id,
            discovery_depth,
            discovery_depth,
            pointer_scan_node_type,
            pointer_address,
            pointer_value,
            &module_name,
            module_offset,
            resolved_target_address,
            pointer_offset,
            has_children,
            1,
            None,
        ))
    }

    fn create_materialized_pointer_scan_node(
        &mut self,
        candidate_id: u64,
        discovery_depth: u64,
        branch_total_depth: u64,
        pointer_scan_node_type: crate::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType,
        pointer_address: u64,
        pointer_value: u64,
        module_name: &str,
        module_offset: u64,
        resolved_target_address: u64,
        pointer_offset: i64,
        has_children: bool,
        display_depth: u64,
        parent_node_id: Option<u64>,
    ) -> u64 {
        let materialized_node_id = self.next_materialized_node_id;
        self.next_materialized_node_id = self.next_materialized_node_id.saturating_add(1);
        let materialized_pointer_scan_node = PointerScanNode::new_materialized(
            materialized_node_id,
            candidate_id,
            discovery_depth,
            branch_total_depth,
            display_depth,
            parent_node_id,
            pointer_scan_node_type,
            pointer_address,
            pointer_value,
            resolved_target_address,
            pointer_offset,
            module_name.to_string(),
            module_offset,
            has_children,
        );
        self.materialized_pointer_scan_nodes
            .insert(materialized_node_id, materialized_pointer_scan_node);

        materialized_node_id
    }

    fn find_level_candidates_mut(
        &mut self,
        discovery_depth: u64,
    ) -> Option<&mut PointerScanLevelCandidates> {
        discovery_depth
            .checked_sub(1)
            .and_then(|level_index| self.pointer_scan_level_candidates.get_mut(level_index as usize))
    }

    fn find_materialized_node(
        &self,
        node_id: u64,
    ) -> Option<&PointerScanNode> {
        self.materialized_pointer_scan_nodes.get(&node_id)
    }

    fn calculate_last_page_index(
        total_node_count: u64,
        page_size: u64,
    ) -> u64 {
        if total_node_count == 0 || page_size == 0 {
            0
        } else {
            total_node_count
                .saturating_sub(1)
                .checked_div(page_size)
                .unwrap_or(0)
        }
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
    use crate::structures::pointer_scans::pointer_scan_node::PointerScanNode;
    use crate::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
    use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;

    fn create_pointer_scan_session() -> PointerScanSession {
        PointerScanSession::new(
            7,
            0x3010,
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
            vec!["game.exe".to_string()],
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
                        0,
                        0,
                    )],
                ),
                PointerScanLevelCandidates::new(
                    2,
                    vec![
                        PointerScanCandidate::new(1, 2, PointerScanNodeType::Static, 0x1000, 0x3000, 0, 0x1000),
                        PointerScanCandidate::new(2, 2, PointerScanNodeType::Static, 0x1100, 0x3000, 0, 0x1100),
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
        assert_eq!(root_nodes[0].get_graph_node_id(), 1);
        assert_eq!(root_nodes[0].get_resolved_target_address(), 0x3010);
        assert!(!root_nodes[0].has_children());
        assert_eq!(root_nodes[1].get_graph_node_id(), 3);
        assert_eq!(root_nodes[1].get_resolved_target_address(), 0x2000);
        assert!(root_nodes[1].has_children());

        let child_nodes = pointer_scan_session.get_expanded_nodes(Some(root_nodes[1].get_node_id()));
        assert_eq!(child_nodes.len(), 1);
        assert_eq!(child_nodes[0].get_graph_node_id(), 3);
        assert_eq!(child_nodes[0].get_parent_node_id(), Some(root_nodes[1].get_node_id()));

        let grandchild_nodes = pointer_scan_session.get_expanded_nodes(Some(child_nodes[0].get_node_id()));
        assert_eq!(grandchild_nodes.len(), 1);
        assert_eq!(grandchild_nodes[0].get_graph_node_id(), 2);
        assert_eq!(grandchild_nodes[0].get_parent_node_id(), Some(child_nodes[0].get_node_id()));
    }

    #[test]
    fn pointer_scan_session_orders_root_pages_by_shortest_chain_first() {
        let mut pointer_scan_session = create_pointer_scan_session();

        let root_nodes = pointer_scan_session.get_expanded_nodes(None);
        let root_discovery_depths = root_nodes
            .iter()
            .map(PointerScanNode::get_discovery_depth)
            .collect::<Vec<_>>();

        assert_eq!(root_discovery_depths, vec![1, 2]);
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
        assert_ne!(first_child_nodes[0].get_graph_node_id(), second_child_nodes[0].get_graph_node_id());
        assert_eq!(first_child_nodes[0].get_parent_node_id(), Some(root_nodes[0].get_node_id()));
        assert_eq!(second_child_nodes[0].get_parent_node_id(), Some(root_nodes[1].get_node_id()));

        let first_grandchild_nodes = pointer_scan_session.get_expanded_nodes(Some(first_child_nodes[0].get_node_id()));
        let second_grandchild_nodes = pointer_scan_session.get_expanded_nodes(Some(second_child_nodes[0].get_node_id()));

        assert_eq!(first_grandchild_nodes.len(), 1);
        assert_eq!(second_grandchild_nodes.len(), 1);
        assert_eq!(first_grandchild_nodes[0].get_graph_node_id(), second_grandchild_nodes[0].get_graph_node_id());
        assert_ne!(first_grandchild_nodes[0].get_node_id(), second_grandchild_nodes[0].get_node_id());
        assert_eq!(first_grandchild_nodes[0].get_parent_node_id(), Some(first_child_nodes[0].get_node_id()));
        assert_eq!(second_grandchild_nodes[0].get_parent_node_id(), Some(second_child_nodes[0].get_node_id()));
    }
}
