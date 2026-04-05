use crate::structures::pointer_scans::pointer_scan_address_space::PointerScanAddressSpace;
use crate::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;
use crate::structures::pointer_scans::pointer_scan_level::PointerScanLevel;
use crate::structures::pointer_scans::pointer_scan_level_candidates::PointerScanLevelCandidates;
use crate::structures::pointer_scans::pointer_scan_level_summary::PointerScanLevelSummary;
use crate::structures::pointer_scans::pointer_scan_node::PointerScanNode;
use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use crate::structures::pointer_scans::pointer_scan_summary::PointerScanSummary;
use crate::structures::pointer_scans::pointer_scan_target_descriptor::PointerScanTargetDescriptor;
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

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct PointerScanSession {
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
    materialized_node_ids_by_page_key: HashMap<MaterializedPointerScanPageKey, Vec<u64>>,
    materialized_pointer_scan_nodes: HashMap<u64, PointerScanNode>,
    next_materialized_node_id: u64,
}

impl PointerScanSession {
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
            materialized_node_ids_by_page_key: HashMap::new(),
            materialized_pointer_scan_nodes: HashMap::new(),
            next_materialized_node_id: 1,
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
        let mut remaining_nodes_to_skip = page_start_index;
        let mut remaining_page_capacity = page_node_count;
        let mut materialized_node_ids = Vec::with_capacity(page_node_count as usize);

        for (candidate_id, discovery_depth, pointer_scan_node_type, pointer_address, pointer_value, module_index, module_offset) in root_page_candidates {
            if remaining_page_capacity == 0 {
                break;
            }

            let root_display_node_count = self.count_root_display_nodes(discovery_depth, pointer_value);

            if remaining_nodes_to_skip >= root_display_node_count {
                remaining_nodes_to_skip = remaining_nodes_to_skip.saturating_sub(root_display_node_count);
                continue;
            }

            let candidate_page_start_index = remaining_nodes_to_skip;
            let candidate_page_node_count = root_display_node_count
                .saturating_sub(candidate_page_start_index)
                .min(remaining_page_capacity);

            materialized_node_ids.extend(self.materialize_root_pointer_scan_nodes(
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

        let root_discovery_depth = parent_materialized_node.get_discovery_depth();

        if root_discovery_depth <= 1 {
            return MaterializedPointerScanNodePage::default();
        }

        let total_node_count = self.count_display_nodes_for_pointer_value(root_discovery_depth, parent_materialized_node.get_pointer_value());
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
            if !self.uses_multi_target_terminal_materialization() {
                let Some(target_address) = self.target_descriptor.get_target_address() else {
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
            if self.uses_multi_target_terminal_materialization() {
                return self.count_matching_target_addresses(pointer_value);
            }

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

    fn materialize_root_pointer_scan_nodes(
        &mut self,
        candidate_id: u64,
        discovery_depth: u64,
        pointer_scan_node_type: crate::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType,
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

        let Some(module_name) = self.get_module_name(module_index).map(str::to_string) else {
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

        if !self.uses_multi_target_terminal_materialization() {
            let Some(target_address) = self.target_descriptor.get_target_address() else {
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

        let matching_target_addresses = self.find_target_addresses_for_pointer_value(pointer_value);
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

    fn count_root_display_nodes(
        &self,
        discovery_depth: u64,
        pointer_value: u64,
    ) -> u64 {
        if !self.uses_multi_target_terminal_materialization() || discovery_depth > 1 {
            1
        } else {
            self.count_matching_target_addresses(pointer_value)
        }
    }

    fn uses_multi_target_terminal_materialization(&self) -> bool {
        matches!(self.target_descriptor, PointerScanTargetDescriptor::Value { .. })
    }

    fn count_matching_target_addresses(
        &self,
        pointer_value: u64,
    ) -> u64 {
        self.find_target_addresses_for_pointer_value(pointer_value)
            .len() as u64
    }

    fn find_target_addresses_for_pointer_value(
        &self,
        pointer_value: u64,
    ) -> &[u64] {
        self.find_target_addresses_in_range(
            pointer_value.saturating_sub(self.offset_radius),
            pointer_value.saturating_add(self.offset_radius),
        )
    }

    fn find_target_addresses_in_range(
        &self,
        lower_bound: u64,
        upper_bound: u64,
    ) -> &[u64] {
        let start_index = self
            .target_addresses
            .partition_point(|target_address| *target_address < lower_bound);
        let end_index = self
            .target_addresses
            .partition_point(|target_address| *target_address <= upper_bound);

        &self.target_addresses[start_index..end_index]
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
    use super::PointerScanSession;
    use crate::structures::pointer_scans::pointer_scan_address_space::PointerScanAddressSpace;
    use crate::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;
    use crate::structures::pointer_scans::pointer_scan_level::PointerScanLevel;
    use crate::structures::pointer_scans::pointer_scan_level_candidates::PointerScanLevelCandidates;
    use crate::structures::pointer_scans::pointer_scan_node::PointerScanNode;
    use crate::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
    use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
    use crate::structures::pointer_scans::pointer_scan_target_descriptor::PointerScanTargetDescriptor;

    fn create_pointer_scan_session() -> PointerScanSession {
        PointerScanSession::new(
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

    fn create_shared_child_pointer_scan_session() -> PointerScanSession {
        PointerScanSession::new(
            8,
            PointerScanTargetDescriptor::address(0x4010),
            vec![0x4010],
            PointerScanAddressSpace::EmulatorMemory,
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
            1,
        )
    }

    fn create_multi_target_leaf_pointer_scan_session() -> PointerScanSession {
        PointerScanSession::new(
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
    fn pointer_scan_session_summary_tracks_level_and_node_counts() {
        let pointer_scan_session = create_pointer_scan_session();
        let pointer_scan_summary = pointer_scan_session.summarize();

        assert_eq!(pointer_scan_summary.get_session_id(), 7);
        assert_eq!(pointer_scan_summary.get_root_node_count(), 2);
        assert_eq!(pointer_scan_summary.get_total_node_count(), 3);
        assert_eq!(pointer_scan_summary.get_total_static_node_count(), 2);
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
    fn pointer_scan_session_materializes_distinct_root_leaf_nodes_for_multiple_targets() {
        let mut pointer_scan_session = create_multi_target_leaf_pointer_scan_session();

        let root_nodes = pointer_scan_session.get_expanded_nodes(None);
        let resolved_target_addresses = root_nodes
            .iter()
            .map(PointerScanNode::get_resolved_target_address)
            .collect::<Vec<_>>();

        assert_eq!(pointer_scan_session.get_root_node_count(), 2);
        assert_eq!(root_nodes.len(), 2);
        assert_eq!(resolved_target_addresses, vec![0x3010, 0x3020]);
        assert!(root_nodes.iter().all(|root_node| !root_node.has_children()));
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
