use crate::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;
use crate::structures::pointer_scans::pointer_scan_level_candidates::PointerScanLevelCandidates;
use crate::structures::pointer_scans::pointer_scan_node::PointerScanNode;
use crate::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
use crate::structures::pointer_scans::pointer_scan_session::PointerScanSession;
use crate::structures::pointer_scans::pointer_scan_target_descriptor::PointerScanTargetDescriptor;
use std::collections::HashMap;

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
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

/// Owns transient pointer-scan browse state such as paged materialization caches.
#[derive(Clone, Debug, Default)]
pub struct PointerScanBrowser {
    materialized_node_ids_by_page_key: HashMap<MaterializedPointerScanPageKey, Vec<u64>>,
    materialized_pointer_scan_nodes: HashMap<u64, PointerScanNode>,
    next_materialized_node_id: u64,
}

impl PointerScanBrowser {
    pub fn new() -> Self {
        Self {
            materialized_node_ids_by_page_key: HashMap::new(),
            materialized_pointer_scan_nodes: HashMap::new(),
            next_materialized_node_id: 1,
        }
    }

    pub fn get_expanded_node_page(
        &mut self,
        pointer_scan_session: &mut PointerScanSession,
        parent_node_id: Option<u64>,
        page_index: u64,
        page_size: u64,
    ) -> (Vec<PointerScanNode>, u64, u64, u64) {
        let bounded_page_size = page_size.max(1);
        let requested_page = match parent_node_id {
            Some(parent_node_id) => self.materialize_child_node_page(pointer_scan_session, parent_node_id, page_index, bounded_page_size),
            None => self.materialize_root_node_page(pointer_scan_session, page_index, bounded_page_size),
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
        pointer_scan_session: &mut PointerScanSession,
        parent_node_id: Option<u64>,
    ) -> Vec<PointerScanNode> {
        self.get_expanded_node_page(pointer_scan_session, parent_node_id, 0, u64::MAX)
            .0
    }

    fn materialize_root_node_page(
        &mut self,
        pointer_scan_session: &mut PointerScanSession,
        page_index: u64,
        page_size: u64,
    ) -> MaterializedPointerScanNodePage {
        let total_node_count = pointer_scan_session.get_root_node_count();
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
        let materialized_node_ids = if Self::uses_single_display_root_materialization(pointer_scan_session) {
            self.materialize_root_node_page_single_display(pointer_scan_session, page_start_index, page_node_count)
        } else {
            self.materialize_root_node_page_multi_display(pointer_scan_session, page_start_index, page_node_count)
        };
        self.materialized_node_ids_by_page_key
            .insert(page_key, materialized_node_ids.clone());

        MaterializedPointerScanNodePage {
            page_index: bounded_page_index,
            last_page_index,
            total_node_count,
            node_ids: materialized_node_ids,
        }
    }

    fn materialize_root_node_page_single_display(
        &mut self,
        pointer_scan_session: &mut PointerScanSession,
        page_start_index: u64,
        page_node_count: u64,
    ) -> Vec<u64> {
        let mut remaining_nodes_to_skip = page_start_index;
        let mut remaining_page_capacity = page_node_count;
        let mut materialized_node_ids = Vec::with_capacity(page_node_count as usize);

        for pointer_scan_level_index in 0..pointer_scan_session.get_pointer_scan_level_candidates().len() {
            if remaining_page_capacity == 0 {
                break;
            }

            let static_candidate_count = pointer_scan_session.get_pointer_scan_level_candidates()[pointer_scan_level_index]
                .get_static_candidates()
                .len() as u64;

            if remaining_nodes_to_skip >= static_candidate_count {
                remaining_nodes_to_skip = remaining_nodes_to_skip.saturating_sub(static_candidate_count);
                continue;
            }

            let static_candidate_start_index = remaining_nodes_to_skip as usize;
            let static_candidate_page_count = static_candidate_count
                .saturating_sub(remaining_nodes_to_skip)
                .min(remaining_page_capacity) as usize;

            for static_candidate_index in static_candidate_start_index..static_candidate_start_index.saturating_add(static_candidate_page_count) {
                let (candidate_id, discovery_depth, pointer_scan_node_type, pointer_address, pointer_value, module_index, module_offset) = {
                    let static_candidate =
                        &pointer_scan_session.get_pointer_scan_level_candidates()[pointer_scan_level_index].get_static_candidates()[static_candidate_index];

                    (
                        static_candidate.get_candidate_id(),
                        static_candidate.get_discovery_depth(),
                        static_candidate.get_pointer_scan_node_type(),
                        static_candidate.get_pointer_address(),
                        static_candidate.get_pointer_value(),
                        static_candidate.get_module_index(),
                        static_candidate.get_module_offset(),
                    )
                };
                materialized_node_ids.extend(self.materialize_root_pointer_scan_nodes(
                    pointer_scan_session,
                    candidate_id,
                    discovery_depth,
                    pointer_scan_node_type,
                    pointer_address,
                    pointer_value,
                    module_index,
                    module_offset,
                    0,
                    1,
                ));
            }

            remaining_nodes_to_skip = 0;
            remaining_page_capacity = remaining_page_capacity.saturating_sub(static_candidate_page_count as u64);
        }

        materialized_node_ids
    }

    fn materialize_root_node_page_multi_display(
        &mut self,
        pointer_scan_session: &mut PointerScanSession,
        page_start_index: u64,
        page_node_count: u64,
    ) -> Vec<u64> {
        let mut remaining_nodes_to_skip = page_start_index;
        let mut remaining_page_capacity = page_node_count;
        let mut materialized_node_ids = Vec::with_capacity(page_node_count as usize);

        for pointer_scan_level_index in 0..pointer_scan_session.get_pointer_scan_level_candidates().len() {
            let static_candidate_count = pointer_scan_session.get_pointer_scan_level_candidates()[pointer_scan_level_index]
                .get_static_candidates()
                .len();

            for static_candidate_index in 0..static_candidate_count {
                if remaining_page_capacity == 0 {
                    return materialized_node_ids;
                }

                let (candidate_id, discovery_depth, pointer_scan_node_type, pointer_address, pointer_value, module_index, module_offset) = {
                    let static_candidate =
                        &pointer_scan_session.get_pointer_scan_level_candidates()[pointer_scan_level_index].get_static_candidates()[static_candidate_index];

                    (
                        static_candidate.get_candidate_id(),
                        static_candidate.get_discovery_depth(),
                        static_candidate.get_pointer_scan_node_type(),
                        static_candidate.get_pointer_address(),
                        static_candidate.get_pointer_value(),
                        static_candidate.get_module_index(),
                        static_candidate.get_module_offset(),
                    )
                };
                let root_display_node_count = Self::count_root_display_nodes(pointer_scan_session, discovery_depth, pointer_value);

                if remaining_nodes_to_skip >= root_display_node_count {
                    remaining_nodes_to_skip = remaining_nodes_to_skip.saturating_sub(root_display_node_count);
                    continue;
                }

                let candidate_page_start_index = remaining_nodes_to_skip;
                let candidate_page_node_count = root_display_node_count
                    .saturating_sub(candidate_page_start_index)
                    .min(remaining_page_capacity);

                materialized_node_ids.extend(self.materialize_root_pointer_scan_nodes(
                    pointer_scan_session,
                    candidate_id,
                    discovery_depth,
                    pointer_scan_node_type,
                    pointer_address,
                    pointer_value,
                    module_index,
                    module_offset,
                    candidate_page_start_index,
                    candidate_page_node_count,
                ));

                remaining_nodes_to_skip = 0;
                remaining_page_capacity = remaining_page_capacity.saturating_sub(candidate_page_node_count);
            }
        }

        materialized_node_ids
    }

    fn materialize_child_node_page(
        &mut self,
        pointer_scan_session: &mut PointerScanSession,
        parent_node_id: u64,
        page_index: u64,
        page_size: u64,
    ) -> MaterializedPointerScanNodePage {
        let Some(parent_materialized_node) = self.find_materialized_node(parent_node_id).cloned() else {
            return MaterializedPointerScanNodePage::default();
        };

        if parent_materialized_node.get_parent_node_id().is_none() {
            return self.materialize_root_child_node_page(pointer_scan_session, parent_node_id, &parent_materialized_node, page_index, page_size);
        }

        let child_discovery_depth = parent_materialized_node.get_discovery_depth().saturating_sub(1);

        if child_discovery_depth == 0 {
            return MaterializedPointerScanNodePage::default();
        }

        let Some(heap_candidate) = Self::find_level_candidates_mut(pointer_scan_session, child_discovery_depth).and_then(|pointer_scan_level_candidates| {
            pointer_scan_level_candidates
                .find_heap_candidate_by_address(parent_materialized_node.get_resolved_target_address())
                .cloned()
        }) else {
            return MaterializedPointerScanNodePage::default();
        };
        let total_node_count =
            self.count_display_nodes_for_pointer_value(pointer_scan_session, heap_candidate.get_discovery_depth(), heap_candidate.get_pointer_value());
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
            pointer_scan_session,
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
        pointer_scan_session: &mut PointerScanSession,
        parent_node_id: u64,
        parent_materialized_node: &PointerScanNode,
        page_index: u64,
        page_size: u64,
    ) -> MaterializedPointerScanNodePage {
        if !parent_materialized_node.has_children() {
            return MaterializedPointerScanNodePage::default();
        }

        let root_discovery_depth = parent_materialized_node.get_discovery_depth();

        if root_discovery_depth <= 1 {
            return MaterializedPointerScanNodePage::default();
        }

        let total_node_count =
            self.count_display_nodes_for_pointer_value(pointer_scan_session, root_discovery_depth, parent_materialized_node.get_pointer_value());
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
            pointer_scan_session,
            parent_materialized_node.get_graph_node_id(),
            root_discovery_depth,
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
        pointer_scan_session: &mut PointerScanSession,
        candidate_id: u64,
        discovery_depth: u64,
        branch_total_depth: u64,
        pointer_scan_node_type: PointerScanNodeType,
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
            if !Self::uses_multi_target_terminal_materialization(pointer_scan_session) {
                let Some(target_address) = pointer_scan_session
                    .get_target_descriptor()
                    .get_target_address()
                else {
                    return Vec::new();
                };

                if page_start_index > 0 {
                    return Vec::new();
                }
                let Some(pointer_offset) = Self::calculate_pointer_offset(target_address, pointer_value) else {
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
                    target_address,
                    pointer_offset,
                    false,
                    display_depth,
                    parent_node_id,
                );

                return vec![materialized_pointer_scan_node];
            }

            return self.materialize_terminal_target_node_page(
                pointer_scan_session,
                candidate_id,
                discovery_depth,
                branch_total_depth,
                pointer_scan_node_type,
                pointer_address,
                pointer_value,
                module_name,
                module_offset,
                display_depth,
                parent_node_id,
                page_start_index,
                page_node_count,
            );
        }

        let lower_bound = pointer_value.saturating_sub(pointer_scan_session.get_offset_radius());
        let upper_bound = pointer_value.saturating_add(pointer_scan_session.get_offset_radius());
        let Some(next_pointer_scan_level_candidates) = Self::find_level_candidates_mut(pointer_scan_session, discovery_depth.saturating_sub(1)) else {
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
        pointer_scan_session: &mut PointerScanSession,
        discovery_depth: u64,
        pointer_value: u64,
    ) -> u64 {
        if discovery_depth <= 1 {
            if Self::uses_multi_target_terminal_materialization(pointer_scan_session) {
                return Self::count_matching_target_addresses(pointer_scan_session, pointer_value);
            }

            return 1;
        }

        let lower_bound = pointer_value.saturating_sub(pointer_scan_session.get_offset_radius());
        let upper_bound = pointer_value.saturating_add(pointer_scan_session.get_offset_radius());

        Self::find_level_candidates_mut(pointer_scan_session, discovery_depth.saturating_sub(1))
            .map(|next_pointer_scan_level_candidates| {
                next_pointer_scan_level_candidates
                    .find_heap_candidates_in_range(lower_bound, upper_bound)
                    .len() as u64
            })
            .unwrap_or(0)
    }

    fn materialize_root_pointer_scan_nodes(
        &mut self,
        pointer_scan_session: &mut PointerScanSession,
        candidate_id: u64,
        discovery_depth: u64,
        pointer_scan_node_type: PointerScanNodeType,
        pointer_address: u64,
        pointer_value: u64,
        module_index: usize,
        module_offset: u64,
        page_start_index: u64,
        page_node_count: u64,
    ) -> Vec<u64> {
        if page_node_count == 0 {
            return Vec::new();
        }

        let Some(module_name) = pointer_scan_session
            .get_module_name(module_index)
            .map(str::to_string)
        else {
            return Vec::new();
        };
        let has_children = discovery_depth > 1;
        if has_children {
            if page_start_index > 0 {
                return Vec::new();
            }

            return vec![self.create_materialized_pointer_scan_node(
                candidate_id,
                discovery_depth,
                discovery_depth,
                pointer_scan_node_type,
                pointer_address,
                pointer_value,
                &module_name,
                module_offset,
                pointer_value,
                0,
                true,
                1,
                None,
            )];
        }

        if !Self::uses_multi_target_terminal_materialization(pointer_scan_session) {
            let Some(target_address) = pointer_scan_session
                .get_target_descriptor()
                .get_target_address()
            else {
                return Vec::new();
            };
            let Some(pointer_offset) = Self::calculate_pointer_offset(target_address, pointer_value) else {
                return Vec::new();
            };

            return vec![self.create_materialized_pointer_scan_node(
                candidate_id,
                discovery_depth,
                discovery_depth,
                pointer_scan_node_type,
                pointer_address,
                pointer_value,
                &module_name,
                module_offset,
                target_address,
                pointer_offset,
                false,
                1,
                None,
            )];
        }

        self.materialize_terminal_target_node_page(
            pointer_scan_session,
            candidate_id,
            discovery_depth,
            discovery_depth,
            pointer_scan_node_type,
            pointer_address,
            pointer_value,
            &module_name,
            module_offset,
            1,
            None,
            page_start_index,
            page_node_count,
        )
    }

    fn materialize_terminal_target_node_page(
        &mut self,
        pointer_scan_session: &mut PointerScanSession,
        candidate_id: u64,
        discovery_depth: u64,
        branch_total_depth: u64,
        pointer_scan_node_type: PointerScanNodeType,
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

        let matching_target_addresses = Self::find_target_addresses_for_pointer_value(pointer_scan_session, pointer_value);
        let page_start_index = page_start_index as usize;
        let page_end_index = page_start_index
            .saturating_add(page_node_count as usize)
            .min(matching_target_addresses.len());
        let resolved_target_addresses = matching_target_addresses[page_start_index..page_end_index].to_vec();
        let mut materialized_node_ids = Vec::with_capacity(resolved_target_addresses.len());

        for resolved_target_address in resolved_target_addresses {
            let Some(pointer_offset) = Self::calculate_pointer_offset(resolved_target_address, pointer_value) else {
                continue;
            };

            materialized_node_ids.push(self.create_materialized_pointer_scan_node(
                candidate_id,
                discovery_depth,
                branch_total_depth,
                pointer_scan_node_type,
                pointer_address,
                pointer_value,
                module_name,
                module_offset,
                resolved_target_address,
                pointer_offset,
                false,
                display_depth,
                parent_node_id,
            ));
        }

        materialized_node_ids
    }

    fn create_materialized_pointer_scan_node(
        &mut self,
        candidate_id: u64,
        discovery_depth: u64,
        branch_total_depth: u64,
        pointer_scan_node_type: PointerScanNodeType,
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
        pointer_scan_session: &mut PointerScanSession,
        discovery_depth: u64,
    ) -> Option<&mut PointerScanLevelCandidates> {
        discovery_depth.checked_sub(1).and_then(|level_index| {
            pointer_scan_session
                .get_pointer_scan_level_candidates_mut()
                .get_mut(level_index as usize)
        })
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

    fn count_root_display_nodes(
        pointer_scan_session: &PointerScanSession,
        discovery_depth: u64,
        pointer_value: u64,
    ) -> u64 {
        if !Self::uses_multi_target_terminal_materialization(pointer_scan_session) || discovery_depth > 1 {
            1
        } else {
            Self::count_matching_target_addresses(pointer_scan_session, pointer_value)
        }
    }

    fn uses_multi_target_terminal_materialization(pointer_scan_session: &PointerScanSession) -> bool {
        matches!(pointer_scan_session.get_target_descriptor(), PointerScanTargetDescriptor::Value { .. })
    }

    fn uses_single_display_root_materialization(pointer_scan_session: &PointerScanSession) -> bool {
        !Self::uses_multi_target_terminal_materialization(pointer_scan_session)
    }

    fn count_matching_target_addresses(
        pointer_scan_session: &PointerScanSession,
        pointer_value: u64,
    ) -> u64 {
        Self::find_target_addresses_for_pointer_value(pointer_scan_session, pointer_value).len() as u64
    }

    fn find_target_addresses_for_pointer_value(
        pointer_scan_session: &PointerScanSession,
        pointer_value: u64,
    ) -> &[u64] {
        Self::find_target_addresses_in_range(
            pointer_scan_session.get_target_addresses(),
            pointer_value.saturating_sub(pointer_scan_session.get_offset_radius()),
            pointer_value.saturating_add(pointer_scan_session.get_offset_radius()),
        )
    }

    fn find_target_addresses_in_range(
        target_addresses: &[u64],
        lower_bound: u64,
        upper_bound: u64,
    ) -> &[u64] {
        let start_index = target_addresses.partition_point(|target_address| *target_address < lower_bound);
        let end_index = target_addresses.partition_point(|target_address| *target_address <= upper_bound);

        &target_addresses[start_index..end_index]
    }
}
