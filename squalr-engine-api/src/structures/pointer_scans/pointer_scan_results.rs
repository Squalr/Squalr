use crate::structures::pointer_scans::pointer_scan_address_space::PointerScanAddressSpace;
use crate::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;
use crate::structures::pointer_scans::pointer_scan_level::PointerScanLevel;
use crate::structures::pointer_scans::pointer_scan_level_candidates::PointerScanLevelCandidates;
use crate::structures::pointer_scans::pointer_scan_level_summary::PointerScanLevelSummary;
use crate::structures::pointer_scans::pointer_scan_node::PointerScanNode;
use crate::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
use crate::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use crate::structures::pointer_scans::pointer_scan_summary::PointerScanSummary;
use crate::structures::pointer_scans::pointer_scan_target_descriptor::PointerScanTargetDescriptor;
use serde::{Deserialize, Serialize};
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

fn default_next_materialized_node_id() -> u64 {
    1
}

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
    #[serde(skip, default)]
    materialized_node_ids_by_page_key: HashMap<MaterializedPointerScanPageKey, Vec<u64>>,
    #[serde(skip, default)]
    materialized_pointer_scan_nodes: HashMap<u64, PointerScanNode>,
    #[serde(skip, default = "default_next_materialized_node_id")]
    next_materialized_node_id: u64,
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
        let total_node_count = self.get_root_node_count();
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
        let materialized_node_ids = if Self::uses_single_display_root_materialization(self) {
            self.materialize_root_node_page_single_display(page_start_index, page_node_count)
        } else {
            self.materialize_root_node_page_multi_display(page_start_index, page_node_count)
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
        page_start_index: u64,
        page_node_count: u64,
    ) -> Vec<u64> {
        let mut remaining_nodes_to_skip = page_start_index;
        let mut remaining_page_capacity = page_node_count;
        let mut materialized_node_ids = Vec::with_capacity(page_node_count as usize);

        for pointer_scan_level_index in 0..self.get_pointer_scan_level_candidates().len() {
            if remaining_page_capacity == 0 {
                break;
            }

            let static_candidate_count = self.get_pointer_scan_level_candidates()[pointer_scan_level_index]
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
                    let static_candidate = &self.get_pointer_scan_level_candidates()[pointer_scan_level_index].get_static_candidates()[static_candidate_index];

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
        page_start_index: u64,
        page_node_count: u64,
    ) -> Vec<u64> {
        let mut remaining_nodes_to_skip = page_start_index;
        let mut remaining_page_capacity = page_node_count;
        let mut materialized_node_ids = Vec::with_capacity(page_node_count as usize);

        for pointer_scan_level_index in 0..self.get_pointer_scan_level_candidates().len() {
            let static_candidate_count = self.get_pointer_scan_level_candidates()[pointer_scan_level_index]
                .get_static_candidates()
                .len();

            for static_candidate_index in 0..static_candidate_count {
                if remaining_page_capacity == 0 {
                    return materialized_node_ids;
                }

                let (candidate_id, discovery_depth, pointer_scan_node_type, pointer_address, pointer_value, module_index, module_offset) = {
                    let static_candidate = &self.get_pointer_scan_level_candidates()[pointer_scan_level_index].get_static_candidates()[static_candidate_index];

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
                let root_display_node_count = Self::count_root_display_nodes(self, discovery_depth, pointer_value);

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
        }

        materialized_node_ids
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

        let Some(heap_candidate) = Self::find_level_candidates_mut(self, child_discovery_depth).and_then(|pointer_scan_level_candidates| {
            pointer_scan_level_candidates
                .find_heap_candidate_by_address(parent_materialized_node.get_resolved_target_address())
                .cloned()
        }) else {
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
            if !Self::uses_multi_target_terminal_materialization(self) {
                let Some(target_address) = self.get_target_descriptor().get_target_address() else {
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

        let lower_bound = pointer_value.saturating_sub(self.get_offset_radius());
        let upper_bound = pointer_value.saturating_add(self.get_offset_radius());
        let Some(next_pointer_scan_level_candidates) = Self::find_level_candidates_mut(self, discovery_depth.saturating_sub(1)) else {
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
            if Self::uses_multi_target_terminal_materialization(self) {
                return Self::count_matching_target_addresses(self, pointer_value);
            }

            return 1;
        }

        let lower_bound = pointer_value.saturating_sub(self.get_offset_radius());
        let upper_bound = pointer_value.saturating_add(self.get_offset_radius());

        Self::find_level_candidates_mut(self, discovery_depth.saturating_sub(1))
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

        if !Self::uses_multi_target_terminal_materialization(self) {
            let Some(target_address) = self.get_target_descriptor().get_target_address() else {
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

        let matching_target_addresses = Self::find_target_addresses_for_pointer_value(self, pointer_value);
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
        pointer_scan_results: &mut PointerScanResults,
        discovery_depth: u64,
    ) -> Option<&mut PointerScanLevelCandidates> {
        discovery_depth.checked_sub(1).and_then(|level_index| {
            pointer_scan_results
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
        pointer_scan_results: &PointerScanResults,
        discovery_depth: u64,
        pointer_value: u64,
    ) -> u64 {
        if !Self::uses_multi_target_terminal_materialization(pointer_scan_results) || discovery_depth > 1 {
            1
        } else {
            Self::count_matching_target_addresses(pointer_scan_results, pointer_value)
        }
    }

    fn uses_multi_target_terminal_materialization(pointer_scan_results: &PointerScanResults) -> bool {
        matches!(pointer_scan_results.get_target_descriptor(), PointerScanTargetDescriptor::Value { .. })
    }

    fn uses_single_display_root_materialization(pointer_scan_results: &PointerScanResults) -> bool {
        !Self::uses_multi_target_terminal_materialization(pointer_scan_results)
    }

    fn count_matching_target_addresses(
        pointer_scan_results: &PointerScanResults,
        pointer_value: u64,
    ) -> u64 {
        Self::find_target_addresses_for_pointer_value(pointer_scan_results, pointer_value).len() as u64
    }

    fn find_target_addresses_for_pointer_value(
        pointer_scan_results: &PointerScanResults,
        pointer_value: u64,
    ) -> &[u64] {
        Self::find_target_addresses_in_range(
            pointer_scan_results.get_target_addresses(),
            pointer_value.saturating_sub(pointer_scan_results.get_offset_radius()),
            pointer_value.saturating_add(pointer_scan_results.get_offset_radius()),
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
