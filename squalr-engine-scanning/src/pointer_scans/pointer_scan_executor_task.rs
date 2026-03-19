use crate::scanners::scan_execution_context::ScanExecutionContext;
use crate::scanners::value_collector_task::ValueCollector;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::pointer_scans::pointer_scan_level::PointerScanLevel;
use squalr_engine_api::structures::pointer_scans::pointer_scan_node::PointerScanNode;
use squalr_engine_api::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::pointer_scans::pointer_scan_session::PointerScanSession;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::scanning::plans::pointer_scan::pointer_scan_parameters::PointerScanParameters;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};

pub struct PointerScanExecutor;

#[derive(Clone, Debug, Eq, PartialEq)]
struct DiscoveredPointerNode {
    pointer_scan_node_type: PointerScanNodeType,
    pointer_address: u64,
    pointer_value: u64,
    resolved_target_address: u64,
    pointer_offset: i64,
    module_name: String,
    module_offset: u64,
}

#[derive(Clone, Debug, Default)]
struct PointerScanLevelAccumulator {
    node_ids: Vec<u64>,
    static_node_count: u64,
    heap_node_count: u64,
}

impl PointerScanLevelAccumulator {
    fn track_node(
        &mut self,
        node_id: u64,
        pointer_scan_node_type: PointerScanNodeType,
    ) {
        self.node_ids.push(node_id);

        match pointer_scan_node_type {
            PointerScanNodeType::Static => {
                self.static_node_count = self.static_node_count.saturating_add(1);
            }
            PointerScanNodeType::Heap => {
                self.heap_node_count = self.heap_node_count.saturating_add(1);
            }
        }
    }
}

/// Implementation of a task that discovers pointer chains against the provided snapshot values.
impl PointerScanExecutor {
    pub fn execute_scan(
        process_info: OpenedProcessInfo,
        statics_snapshot: Arc<RwLock<Snapshot>>,
        heaps_snapshot: Arc<RwLock<Snapshot>>,
        pointer_scan_session_id: u64,
        pointer_scan_parameters: PointerScanParameters,
        modules: &[NormalizedModule],
        with_logging: bool,
        scan_execution_context: &ScanExecutionContext,
    ) -> PointerScanSession {
        Self::scan_task(
            process_info,
            statics_snapshot,
            heaps_snapshot,
            pointer_scan_session_id,
            pointer_scan_parameters,
            modules,
            with_logging,
            scan_execution_context,
        )
    }

    fn scan_task(
        process_info: OpenedProcessInfo,
        statics_snapshot: Arc<RwLock<Snapshot>>,
        heaps_snapshot: Arc<RwLock<Snapshot>>,
        pointer_scan_session_id: u64,
        pointer_scan_parameters: PointerScanParameters,
        modules: &[NormalizedModule],
        with_logging: bool,
        scan_execution_context: &ScanExecutionContext,
    ) -> PointerScanSession {
        if with_logging {
            log::info!(
                "Performing pointer scan for target 0x{:X} using {} pointers, max depth {}, and offset {}.",
                pointer_scan_parameters.get_target_address(),
                pointer_scan_parameters.get_pointer_size(),
                pointer_scan_parameters.get_max_depth(),
                pointer_scan_parameters.get_offset_radius(),
            );
        }

        Self::collect_pointer_scan_values(
            process_info.clone(),
            statics_snapshot.clone(),
            heaps_snapshot.clone(),
            with_logging,
            scan_execution_context,
        );

        let pointer_scan_session = if Arc::ptr_eq(&statics_snapshot, &heaps_snapshot) {
            let snapshot_guard = match statics_snapshot.read() {
                Ok(snapshot_guard) => snapshot_guard,
                Err(error) => {
                    if with_logging {
                        log::error!("Failed to acquire read lock on pointer scan snapshot: {}", error);
                    }

                    return Self::create_empty_session(pointer_scan_session_id, &pointer_scan_parameters);
                }
            };

            Self::build_pointer_scan_session(vec![&*snapshot_guard], pointer_scan_session_id, &pointer_scan_parameters, modules, with_logging)
        } else {
            let statics_snapshot_guard = match statics_snapshot.read() {
                Ok(statics_snapshot_guard) => statics_snapshot_guard,
                Err(error) => {
                    if with_logging {
                        log::error!("Failed to acquire read lock on static pointer scan snapshot: {}", error);
                    }

                    return Self::create_empty_session(pointer_scan_session_id, &pointer_scan_parameters);
                }
            };
            let heaps_snapshot_guard = match heaps_snapshot.read() {
                Ok(heaps_snapshot_guard) => heaps_snapshot_guard,
                Err(error) => {
                    if with_logging {
                        log::error!("Failed to acquire read lock on heap pointer scan snapshot: {}", error);
                    }

                    return Self::create_empty_session(pointer_scan_session_id, &pointer_scan_parameters);
                }
            };

            Self::build_pointer_scan_session(
                vec![&*statics_snapshot_guard, &*heaps_snapshot_guard],
                pointer_scan_session_id,
                &pointer_scan_parameters,
                modules,
                with_logging,
            )
        };

        if with_logging {
            let pointer_scan_summary = pointer_scan_session.summarize();

            log::info!(
                "Pointer scan complete: roots={}, total_nodes={}, static_nodes={}, heap_nodes={}",
                pointer_scan_summary.get_root_node_count(),
                pointer_scan_summary.get_total_node_count(),
                pointer_scan_summary.get_total_static_node_count(),
                pointer_scan_summary.get_total_heap_node_count(),
            );
        }

        pointer_scan_session
    }

    fn collect_pointer_scan_values(
        process_info: OpenedProcessInfo,
        statics_snapshot: Arc<RwLock<Snapshot>>,
        heaps_snapshot: Arc<RwLock<Snapshot>>,
        with_logging: bool,
        scan_execution_context: &ScanExecutionContext,
    ) {
        if Arc::ptr_eq(&statics_snapshot, &heaps_snapshot) {
            ValueCollector::collect_values(process_info, statics_snapshot, with_logging, scan_execution_context);
        } else {
            ValueCollector::collect_values(process_info.clone(), statics_snapshot, with_logging, scan_execution_context);
            ValueCollector::collect_values(process_info, heaps_snapshot, with_logging, scan_execution_context);
        }
    }

    fn build_pointer_scan_session(
        snapshots: Vec<&Snapshot>,
        pointer_scan_session_id: u64,
        pointer_scan_parameters: &PointerScanParameters,
        modules: &[NormalizedModule],
        with_logging: bool,
    ) -> PointerScanSession {
        let discovered_pointer_nodes_by_level = Self::discover_pointer_nodes_by_level(&snapshots, pointer_scan_parameters, modules, with_logging);

        if discovered_pointer_nodes_by_level.is_empty() {
            if with_logging {
                log::info!("Pointer scan found no reachable pointer nodes.");
            }

            return Self::create_empty_session(pointer_scan_session_id, pointer_scan_parameters);
        }

        let root_oriented_pointer_nodes_by_level = discovered_pointer_nodes_by_level
            .into_iter()
            .rev()
            .collect::<Vec<_>>();
        let discovery_level_count = root_oriented_pointer_nodes_by_level.len();
        let mut pointer_scan_nodes = Vec::new();
        let mut next_node_id = 1_u64;
        let mut total_static_node_count = 0_u64;
        let mut total_heap_node_count = 0_u64;

        for (pointer_level_index, discovered_pointer_nodes) in root_oriented_pointer_nodes_by_level.iter().enumerate() {
            let depth = pointer_level_index as u64 + 1;

            for discovered_pointer_node in discovered_pointer_nodes {
                let node_id = next_node_id;
                next_node_id = next_node_id.saturating_add(1);

                match discovered_pointer_node.pointer_scan_node_type {
                    PointerScanNodeType::Static => {
                        total_static_node_count = total_static_node_count.saturating_add(1);
                    }
                    PointerScanNodeType::Heap => {
                        total_heap_node_count = total_heap_node_count.saturating_add(1);
                    }
                }

                let mut pointer_scan_node = PointerScanNode::new(
                    node_id,
                    None,
                    discovered_pointer_node.pointer_scan_node_type,
                    depth,
                    discovered_pointer_node.pointer_address,
                    discovered_pointer_node.pointer_value,
                    discovered_pointer_node.resolved_target_address,
                    discovered_pointer_node.pointer_offset,
                    discovered_pointer_node.module_name.clone(),
                    discovered_pointer_node.module_offset,
                    Vec::new(),
                );
                pointer_scan_node.set_discovery_depth(discovery_level_count.saturating_sub(pointer_level_index) as u64);
                pointer_scan_nodes.push(pointer_scan_node);
            }
        }

        Self::attach_graph_child_node_ids(&mut pointer_scan_nodes);
        let (root_node_ids, pointer_scan_levels) = Self::build_root_oriented_levels(&mut pointer_scan_nodes);

        if with_logging {
            for pointer_scan_level in &pointer_scan_levels {
                log::info!(
                    "Pointer scan level {} retained {} unique nodes (static {} / heap {}).",
                    pointer_scan_level.get_depth(),
                    pointer_scan_level.get_node_count(),
                    pointer_scan_level.get_static_node_count(),
                    pointer_scan_level.get_heap_node_count(),
                );
            }
        }

        PointerScanSession::new(
            pointer_scan_session_id,
            pointer_scan_parameters.get_target_address(),
            pointer_scan_parameters.get_pointer_size(),
            pointer_scan_parameters.get_max_depth(),
            pointer_scan_parameters.get_offset_radius(),
            root_node_ids,
            pointer_scan_levels,
            pointer_scan_nodes,
            total_static_node_count,
            total_heap_node_count,
        )
    }

    fn discover_pointer_nodes_by_level(
        snapshots: &[&Snapshot],
        pointer_scan_parameters: &PointerScanParameters,
        modules: &[NormalizedModule],
        with_logging: bool,
    ) -> Vec<Vec<DiscoveredPointerNode>> {
        let max_depth = pointer_scan_parameters.get_max_depth();

        if max_depth == 0 {
            return Vec::new();
        }

        let target_address = pointer_scan_parameters.get_target_address();
        let mut frontier_target_addresses = vec![target_address];
        let mut discovered_pointer_nodes_by_level = Vec::new();
        let total_snapshot_region_count = snapshots
            .iter()
            .map(|snapshot| snapshot.get_snapshot_regions().len())
            .sum::<usize>();

        for pointer_chain_depth in 0..max_depth {
            let level_number = pointer_chain_depth.saturating_add(1);
            frontier_target_addresses.sort_unstable();
            frontier_target_addresses.dedup();

            if frontier_target_addresses.is_empty() {
                if with_logging {
                    log::info!(
                        "Pointer scan stopped after level {} because no frontier targets remained.",
                        level_number.saturating_sub(1)
                    );
                }

                break;
            }

            if with_logging {
                log::info!(
                    "Pointer scan level {}/{}: scanning {} snapshot regions for {} frontier targets.",
                    level_number,
                    max_depth,
                    total_snapshot_region_count,
                    frontier_target_addresses.len(),
                );
            }

            let pointer_matches_by_target = Self::scan_snapshots_for_pointer_targets(
                snapshots,
                &frontier_target_addresses,
                pointer_scan_parameters.get_pointer_size(),
                pointer_scan_parameters.get_offset_radius(),
                modules,
            );
            let mut discovered_pointer_nodes = pointer_matches_by_target
                .values()
                .flat_map(|pointer_matches| pointer_matches.iter().cloned())
                .collect::<Vec<_>>();
            discovered_pointer_nodes.sort_by(Self::compare_discovered_pointer_nodes);
            discovered_pointer_nodes.dedup();

            if with_logging {
                let next_frontier_target_count = discovered_pointer_nodes
                    .iter()
                    .map(|discovered_pointer_node| discovered_pointer_node.pointer_address)
                    .collect::<Vec<_>>()
                    .len();

                log::info!(
                    "Pointer scan level {}/{}: matched {} frontier targets, retained {} unique pointer nodes, and produced {} next frontier targets before deduplication.",
                    level_number,
                    max_depth,
                    pointer_matches_by_target.len(),
                    discovered_pointer_nodes.len(),
                    next_frontier_target_count,
                );
            }

            if discovered_pointer_nodes.is_empty() {
                if with_logging {
                    log::info!(
                        "Pointer scan stopped after level {} because no deeper pointer candidates were found.",
                        level_number
                    );
                }

                break;
            }

            frontier_target_addresses = discovered_pointer_nodes
                .iter()
                .map(|discovered_pointer_node| discovered_pointer_node.pointer_address)
                .collect();
            discovered_pointer_nodes_by_level.push(discovered_pointer_nodes);
        }

        if with_logging {
            let discovered_pointer_node_count = discovered_pointer_nodes_by_level
                .iter()
                .map(Vec::len)
                .sum::<usize>();

            log::info!(
                "Pointer scan discovered {} unique reachable pointer nodes across {} levels before any tree expansion.",
                discovered_pointer_node_count,
                discovered_pointer_nodes_by_level.len(),
            );
        }

        discovered_pointer_nodes_by_level
    }

    fn scan_snapshots_for_pointer_targets(
        snapshots: &[&Snapshot],
        frontier_target_addresses: &[u64],
        pointer_size: PointerScanPointerSize,
        offset_radius: u64,
        modules: &[NormalizedModule],
    ) -> HashMap<u64, Vec<DiscoveredPointerNode>> {
        let mut pointer_matches_by_target: HashMap<u64, Vec<DiscoveredPointerNode>> = HashMap::new();

        if frontier_target_addresses.is_empty() {
            return pointer_matches_by_target;
        }

        for snapshot in snapshots {
            for snapshot_region in snapshot.get_snapshot_regions() {
                Self::scan_snapshot_region_for_pointer_targets(
                    snapshot_region,
                    frontier_target_addresses,
                    pointer_size,
                    offset_radius,
                    modules,
                    &mut pointer_matches_by_target,
                );
            }
        }

        for pointer_matches in pointer_matches_by_target.values_mut() {
            pointer_matches.sort_by(Self::compare_discovered_pointer_nodes);
            pointer_matches.dedup();
        }

        pointer_matches_by_target
    }

    fn scan_snapshot_region_for_pointer_targets(
        snapshot_region: &SnapshotRegion,
        frontier_target_addresses: &[u64],
        pointer_size: PointerScanPointerSize,
        offset_radius: u64,
        modules: &[NormalizedModule],
        pointer_matches_by_target: &mut HashMap<u64, Vec<DiscoveredPointerNode>>,
    ) {
        let pointer_size_in_bytes = pointer_size.get_size_in_bytes() as usize;
        let current_values = snapshot_region.get_current_values();

        if current_values.len() < pointer_size_in_bytes {
            return;
        }

        let base_address = snapshot_region.get_base_address();
        let pointer_alignment = pointer_size_in_bytes as u64;
        let alignment_remainder = base_address % pointer_alignment;
        let start_offset = if alignment_remainder == 0 {
            0_usize
        } else {
            (pointer_alignment.saturating_sub(alignment_remainder)) as usize
        };

        if start_offset.saturating_add(pointer_size_in_bytes) > current_values.len() {
            return;
        }

        let mut pointer_value_offset = start_offset;

        while pointer_value_offset.saturating_add(pointer_size_in_bytes) <= current_values.len() {
            let value_slice = &current_values[pointer_value_offset..pointer_value_offset + pointer_size_in_bytes];
            let Some(pointer_value) = Self::read_pointer_value(value_slice, pointer_size) else {
                pointer_value_offset = pointer_value_offset.saturating_add(pointer_size_in_bytes);
                continue;
            };

            let lower_target_bound = pointer_value.saturating_sub(offset_radius);
            let upper_target_bound = pointer_value.saturating_add(offset_radius);
            let matching_target_start_index = frontier_target_addresses.partition_point(|target_address| *target_address < lower_target_bound);
            let matching_target_end_index = frontier_target_addresses.partition_point(|target_address| *target_address <= upper_target_bound);

            if matching_target_start_index < matching_target_end_index {
                let pointer_address = base_address.saturating_add(pointer_value_offset as u64);
                let (pointer_scan_node_type, module_name, module_offset) = Self::classify_pointer_address(pointer_address, modules);

                for target_address in &frontier_target_addresses[matching_target_start_index..matching_target_end_index] {
                    let Some(pointer_offset) = Self::calculate_pointer_offset(*target_address, pointer_value) else {
                        continue;
                    };

                    pointer_matches_by_target
                        .entry(*target_address)
                        .or_default()
                        .push(DiscoveredPointerNode {
                            pointer_scan_node_type,
                            pointer_address,
                            pointer_value,
                            resolved_target_address: *target_address,
                            pointer_offset,
                            module_name: module_name.clone(),
                            module_offset,
                        });
                }
            }

            pointer_value_offset = pointer_value_offset.saturating_add(pointer_size_in_bytes);
        }
    }

    fn classify_pointer_address(
        pointer_address: u64,
        modules: &[NormalizedModule],
    ) -> (PointerScanNodeType, String, u64) {
        if let Some(module) = modules
            .iter()
            .find(|module| module.contains_address(pointer_address))
        {
            (
                PointerScanNodeType::Static,
                module.get_module_name().to_string(),
                pointer_address.saturating_sub(module.get_base_address()),
            )
        } else {
            (PointerScanNodeType::Heap, String::new(), 0)
        }
    }

    fn calculate_pointer_offset(
        target_address: u64,
        pointer_value: u64,
    ) -> Option<i64> {
        let pointer_offset = target_address as i128 - pointer_value as i128;

        i64::try_from(pointer_offset).ok()
    }

    fn read_pointer_value(
        pointer_bytes: &[u8],
        pointer_size: PointerScanPointerSize,
    ) -> Option<u64> {
        match pointer_size {
            PointerScanPointerSize::Pointer32 => {
                let pointer_bytes: [u8; 4] = pointer_bytes.try_into().ok()?;

                Some(u32::from_le_bytes(pointer_bytes) as u64)
            }
            PointerScanPointerSize::Pointer64 => {
                let pointer_bytes: [u8; 8] = pointer_bytes.try_into().ok()?;

                Some(u64::from_le_bytes(pointer_bytes))
            }
        }
    }

    fn attach_graph_child_node_ids(pointer_scan_nodes: &mut [PointerScanNode]) {
        let mut graph_child_node_ids_by_depth_and_pointer_address: HashMap<(u64, u64), Vec<u64>> = HashMap::new();

        for pointer_scan_node in pointer_scan_nodes.iter() {
            graph_child_node_ids_by_depth_and_pointer_address
                .entry((pointer_scan_node.get_depth(), pointer_scan_node.get_pointer_address()))
                .or_default()
                .push(pointer_scan_node.get_node_id());
        }

        for pointer_scan_node in pointer_scan_nodes.iter_mut() {
            let child_node_ids = graph_child_node_ids_by_depth_and_pointer_address
                .get(&(pointer_scan_node.get_depth().saturating_add(1), pointer_scan_node.get_resolved_target_address()))
                .cloned()
                .unwrap_or_default();

            pointer_scan_node.set_child_node_ids(child_node_ids);
        }
    }

    fn build_root_oriented_levels(pointer_scan_nodes: &mut [PointerScanNode]) -> (Vec<u64>, Vec<PointerScanLevel>) {
        let mut parent_count_by_node_id: HashMap<u64, usize> = HashMap::new();

        for pointer_scan_node in pointer_scan_nodes.iter() {
            for child_node_id in pointer_scan_node.get_child_node_ids() {
                *parent_count_by_node_id.entry(*child_node_id).or_default() += 1;
            }
        }

        let root_node_ids = pointer_scan_nodes
            .iter()
            .filter(|pointer_scan_node| !parent_count_by_node_id.contains_key(&pointer_scan_node.get_node_id()))
            .map(PointerScanNode::get_node_id)
            .collect::<Vec<_>>();
        let mut assigned_depth_by_node_id = HashMap::new();
        let mut breadth_first_queue = VecDeque::new();

        for root_node_id in &root_node_ids {
            assigned_depth_by_node_id.insert(*root_node_id, 1_u64);
            breadth_first_queue.push_back(*root_node_id);
        }

        while let Some(parent_node_id) = breadth_first_queue.pop_front() {
            let Some(parent_depth) = assigned_depth_by_node_id.get(&parent_node_id).copied() else {
                continue;
            };
            let Some(parent_pointer_scan_node) = pointer_scan_nodes
                .iter()
                .find(|pointer_scan_node| pointer_scan_node.get_node_id() == parent_node_id)
            else {
                continue;
            };

            for child_node_id in parent_pointer_scan_node.get_child_node_ids() {
                let next_depth = parent_depth.saturating_add(1);

                if assigned_depth_by_node_id
                    .get(child_node_id)
                    .map(|existing_depth| *existing_depth <= next_depth)
                    .unwrap_or(false)
                {
                    continue;
                }

                assigned_depth_by_node_id.insert(*child_node_id, next_depth);
                breadth_first_queue.push_back(*child_node_id);
            }
        }

        let mut level_accumulators = Vec::new();

        for pointer_scan_node in pointer_scan_nodes.iter_mut() {
            let assigned_depth = assigned_depth_by_node_id
                .get(&pointer_scan_node.get_node_id())
                .copied()
                .unwrap_or(1);

            pointer_scan_node.set_depth(assigned_depth);

            while level_accumulators.len() < assigned_depth as usize {
                level_accumulators.push(PointerScanLevelAccumulator::default());
            }

            level_accumulators[assigned_depth.saturating_sub(1) as usize]
                .track_node(pointer_scan_node.get_node_id(), pointer_scan_node.get_pointer_scan_node_type());
        }

        let pointer_scan_levels = level_accumulators
            .into_iter()
            .enumerate()
            .map(|(pointer_level_index, level_accumulator)| {
                PointerScanLevel::new(
                    pointer_level_index as u64 + 1,
                    level_accumulator.node_ids,
                    level_accumulator.static_node_count,
                    level_accumulator.heap_node_count,
                )
            })
            .collect::<Vec<_>>();

        (root_node_ids, pointer_scan_levels)
    }

    fn compare_discovered_pointer_nodes(
        left_pointer_node: &DiscoveredPointerNode,
        right_pointer_node: &DiscoveredPointerNode,
    ) -> Ordering {
        left_pointer_node
            .pointer_address
            .cmp(&right_pointer_node.pointer_address)
            .then_with(|| {
                left_pointer_node
                    .pointer_value
                    .cmp(&right_pointer_node.pointer_value)
            })
            .then_with(|| {
                left_pointer_node
                    .resolved_target_address
                    .cmp(&right_pointer_node.resolved_target_address)
            })
            .then_with(|| {
                left_pointer_node
                    .pointer_offset
                    .cmp(&right_pointer_node.pointer_offset)
            })
            .then_with(|| {
                left_pointer_node
                    .module_name
                    .cmp(&right_pointer_node.module_name)
            })
            .then_with(|| {
                left_pointer_node
                    .module_offset
                    .cmp(&right_pointer_node.module_offset)
            })
            .then_with(|| {
                let left_node_class = match left_pointer_node.pointer_scan_node_type {
                    PointerScanNodeType::Heap => 0_u8,
                    PointerScanNodeType::Static => 1_u8,
                };
                let right_node_class = match right_pointer_node.pointer_scan_node_type {
                    PointerScanNodeType::Heap => 0_u8,
                    PointerScanNodeType::Static => 1_u8,
                };

                left_node_class.cmp(&right_node_class)
            })
    }

    fn create_empty_session(
        pointer_scan_session_id: u64,
        pointer_scan_parameters: &PointerScanParameters,
    ) -> PointerScanSession {
        PointerScanSession::new(
            pointer_scan_session_id,
            pointer_scan_parameters.get_target_address(),
            pointer_scan_parameters.get_pointer_size(),
            pointer_scan_parameters.get_max_depth(),
            pointer_scan_parameters.get_offset_radius(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            0,
            0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::PointerScanExecutor;
    use crate::scanners::scan_execution_context::ScanExecutionContext;
    use squalr_engine_api::structures::memory::bitness::Bitness;
    use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
    use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
    use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
    use squalr_engine_api::structures::scanning::plans::pointer_scan::pointer_scan_parameters::PointerScanParameters;
    use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
    use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    #[test]
    fn execute_scan_builds_pointer_chains_and_classifies_static_nodes() {
        let memory_map = Arc::new(build_pointer_scan_memory_map());
        let scan_execution_context = ScanExecutionContext::new(
            None,
            None,
            Some(Arc::new({
                let memory_map = memory_map.clone();

                move |_opened_process_info, address, values| read_memory_from_map(&memory_map, address, values)
            })),
        );
        let snapshot = Arc::new(RwLock::new(build_pointer_scan_snapshot()));
        let pointer_scan_parameters = PointerScanParameters::new(0x3010, PointerScanPointerSize::Pointer64, 0x20, 3, true, false);
        let mut pointer_scan_session = PointerScanExecutor::execute_scan(
            OpenedProcessInfo::new(7, "pointer-test".to_string(), 0, Bitness::Bit64, None),
            snapshot.clone(),
            snapshot,
            41,
            pointer_scan_parameters,
            &[NormalizedModule::new("game.exe", 0x1000, 0x100)],
            false,
            &scan_execution_context,
        );

        assert_eq!(pointer_scan_session.get_session_id(), 41);
        assert_eq!(pointer_scan_session.get_root_node_ids().len(), 2);
        assert_eq!(pointer_scan_session.get_total_node_count(), 3);
        assert_eq!(pointer_scan_session.get_total_static_node_count(), 2);
        assert_eq!(pointer_scan_session.get_total_heap_node_count(), 1);

        let pointer_scan_levels = pointer_scan_session.get_pointer_scan_levels();
        assert_eq!(pointer_scan_levels.len(), 2);
        assert_eq!(pointer_scan_levels[0].get_depth(), 1);
        assert_eq!(pointer_scan_levels[0].get_node_count(), 2);
        assert_eq!(pointer_scan_levels[0].get_static_node_count(), 2);
        assert_eq!(pointer_scan_levels[0].get_heap_node_count(), 0);
        assert_eq!(pointer_scan_levels[1].get_depth(), 2);
        assert_eq!(pointer_scan_levels[1].get_node_count(), 1);
        assert_eq!(pointer_scan_levels[1].get_static_node_count(), 0);
        assert_eq!(pointer_scan_levels[1].get_heap_node_count(), 1);

        let root_nodes = pointer_scan_session.get_expanded_nodes(None);
        assert_eq!(root_nodes.len(), 2);

        let static_chain_root = root_nodes
            .iter()
            .find(|pointer_scan_node| pointer_scan_node.get_pointer_address() == 0x1010)
            .expect("Expected the rooted static pointer chain.");
        let direct_static_root = root_nodes
            .iter()
            .find(|pointer_scan_node| pointer_scan_node.get_pointer_address() == 0x1030)
            .expect("Expected the direct static pointer chain.");

        assert_eq!(static_chain_root.get_pointer_scan_node_type(), PointerScanNodeType::Static);
        assert_eq!(static_chain_root.get_depth(), 1);
        assert_eq!(static_chain_root.get_module_name(), "game.exe");
        assert_eq!(static_chain_root.get_module_offset(), 0x10);
        assert_eq!(static_chain_root.get_resolved_target_address(), 0x2000);
        assert_eq!(static_chain_root.get_pointer_offset(), 0x10);
        assert!(static_chain_root.has_children());

        let child_nodes = pointer_scan_session.get_expanded_nodes(Some(static_chain_root.get_node_id()));
        assert_eq!(child_nodes.len(), 1);
        assert_eq!(child_nodes[0].get_pointer_address(), 0x2000);
        assert_eq!(child_nodes[0].get_pointer_scan_node_type(), PointerScanNodeType::Heap);
        assert_eq!(child_nodes[0].get_depth(), 2);
        assert_eq!(child_nodes[0].get_resolved_target_address(), 0x3010);
        assert_eq!(child_nodes[0].get_pointer_offset(), 0x10);
        assert!(!child_nodes[0].has_children());

        assert_eq!(direct_static_root.get_pointer_scan_node_type(), PointerScanNodeType::Static);
        assert_eq!(direct_static_root.get_depth(), 1);
        assert_eq!(direct_static_root.get_module_name(), "game.exe");
        assert_eq!(direct_static_root.get_module_offset(), 0x30);
        assert_eq!(direct_static_root.get_resolved_target_address(), 0x3010);
        assert_eq!(direct_static_root.get_pointer_offset(), -0x10);
        assert!(!direct_static_root.has_children());
    }

    fn build_pointer_scan_snapshot() -> Snapshot {
        let mut snapshot = Snapshot::new();

        snapshot.set_snapshot_regions(vec![
            SnapshotRegion::new(NormalizedRegion::new(0x1000, 0x40), Vec::new()),
            SnapshotRegion::new(NormalizedRegion::new(0x2000, 0x40), Vec::new()),
            SnapshotRegion::new(NormalizedRegion::new(0x3000, 0x40), Vec::new()),
        ]);

        snapshot
    }

    fn build_pointer_scan_memory_map() -> HashMap<u64, u8> {
        let mut memory_map = HashMap::new();

        write_pointer_bytes(&mut memory_map, 0x1010, 0x1FF0_u64);
        write_pointer_bytes(&mut memory_map, 0x1030, 0x3020_u64);
        write_pointer_bytes(&mut memory_map, 0x2000, 0x3000_u64);

        memory_map
    }

    fn write_pointer_bytes(
        memory_map: &mut HashMap<u64, u8>,
        address: u64,
        value: u64,
    ) {
        for (byte_index, byte_value) in value.to_le_bytes().iter().enumerate() {
            memory_map.insert(address.saturating_add(byte_index as u64), *byte_value);
        }
    }

    fn read_memory_from_map(
        memory_map: &HashMap<u64, u8>,
        address: u64,
        values: &mut [u8],
    ) -> bool {
        for (byte_index, value) in values.iter_mut().enumerate() {
            *value = *memory_map
                .get(&address.saturating_add(byte_index as u64))
                .unwrap_or(&0);
        }

        true
    }
}
