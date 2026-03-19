use crate::scanners::scan_execution_context::ScanExecutionContext;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
use squalr_engine_api::structures::pointer_scans::pointer_scan_level::PointerScanLevel;
use squalr_engine_api::structures::pointer_scans::pointer_scan_node::PointerScanNode;
use squalr_engine_api::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::pointer_scans::pointer_scan_session::PointerScanSession;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use std::cmp::Ordering;
use std::collections::{HashMap, VecDeque};

const VALIDATION_SCAN_CHUNK_SIZE: usize = 64 * 1024;

pub struct PointerScanValidator;

#[derive(Clone, Debug, Eq, PartialEq)]
struct RebuiltPointerNode {
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

#[derive(Clone, Copy, Debug)]
struct PointerValidationLevelLogContext {
    level_number: usize,
    level_count: usize,
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

impl PointerScanValidator {
    pub fn validate_scan(
        process_info: OpenedProcessInfo,
        pointer_scan_session: &PointerScanSession,
        validation_target_address: u64,
        memory_regions: &[NormalizedRegion],
        modules: &[NormalizedModule],
        scan_execution_context: &ScanExecutionContext,
        with_logging: bool,
    ) -> PointerScanSession {
        if with_logging {
            log::info!(
                "Validating pointer scan session {} against target 0x{:X}.",
                pointer_scan_session.get_session_id(),
                validation_target_address,
            );
        }

        if pointer_scan_session.get_pointer_scan_levels().is_empty() {
            return Self::create_empty_session(pointer_scan_session, validation_target_address);
        }

        let mut required_target_addresses = vec![validation_target_address];
        let mut rebuilt_pointer_nodes_by_level = Vec::new();
        let level_count = pointer_scan_session
            .get_pointer_scan_nodes()
            .iter()
            .map(PointerScanNode::get_discovery_depth)
            .max()
            .unwrap_or(0) as usize;

        for level_index in 0..level_count {
            if scan_execution_context.should_cancel() {
                break;
            }

            required_target_addresses.sort_unstable();
            required_target_addresses.dedup();

            if required_target_addresses.is_empty() {
                if with_logging {
                    log::info!(
                        "Pointer scan validation stopped after level {} because no frontier targets remained.",
                        level_index
                    );
                }

                break;
            }

            let validation_level_log_context = PointerValidationLevelLogContext {
                level_number: level_index + 1,
                level_count,
            };
            let discovery_depth = level_index as u64 + 1;
            let static_pointer_scan_nodes = pointer_scan_session
                .get_pointer_scan_nodes()
                .iter()
                .filter(|pointer_scan_node| pointer_scan_node.get_discovery_depth() == discovery_depth)
                .filter(|pointer_scan_node| pointer_scan_node.get_pointer_scan_node_type() == PointerScanNodeType::Static)
                .collect::<Vec<_>>();

            if with_logging {
                log::info!(
                    "Pointer scan validation level {}/{}: checking {} static nodes and scanning {} memory regions for {} frontier targets.",
                    validation_level_log_context.level_number,
                    validation_level_log_context.level_count,
                    static_pointer_scan_nodes.len(),
                    memory_regions.len(),
                    required_target_addresses.len(),
                );
            }

            let mut rebuilt_pointer_nodes = Self::validate_static_pointer_nodes_for_targets(
                &process_info,
                &static_pointer_scan_nodes,
                &required_target_addresses,
                pointer_scan_session.get_pointer_size(),
                pointer_scan_session.get_offset_radius(),
                modules,
                scan_execution_context,
            );
            let rebuilt_static_node_count = rebuilt_pointer_nodes.len();
            let rebuilt_heap_pointer_nodes = Self::scan_memory_regions_for_heap_pointer_nodes_by_target(
                &process_info,
                &required_target_addresses,
                pointer_scan_session.get_pointer_size(),
                pointer_scan_session.get_offset_radius(),
                memory_regions,
                modules,
                scan_execution_context,
                with_logging,
                &validation_level_log_context,
            );
            let rebuilt_heap_node_count = rebuilt_heap_pointer_nodes.len();

            rebuilt_pointer_nodes.extend(rebuilt_heap_pointer_nodes);
            Self::sort_and_deduplicate_rebuilt_pointer_nodes(&mut rebuilt_pointer_nodes);

            if rebuilt_pointer_nodes.is_empty() {
                if with_logging {
                    log::info!(
                        "Pointer scan validation stopped after level {} because no validated nodes remained.",
                        validation_level_log_context.level_number
                    );
                }

                break;
            }

            if with_logging {
                let next_frontier_target_count = rebuilt_pointer_nodes
                    .iter()
                    .map(|rebuilt_pointer_node| rebuilt_pointer_node.pointer_address)
                    .collect::<Vec<_>>()
                    .len();

                log::info!(
                    "Pointer scan validation level {}/{}: retained {} static nodes, rebuilt {} heap nodes, and produced {} next frontier targets before deduplication.",
                    validation_level_log_context.level_number,
                    validation_level_log_context.level_count,
                    rebuilt_static_node_count,
                    rebuilt_heap_node_count,
                    next_frontier_target_count,
                );
            }

            required_target_addresses = rebuilt_pointer_nodes
                .iter()
                .map(|rebuilt_pointer_node| rebuilt_pointer_node.pointer_address)
                .collect();
            rebuilt_pointer_nodes_by_level.push(rebuilt_pointer_nodes);
        }

        let validated_pointer_scan_session = if rebuilt_pointer_nodes_by_level.is_empty() {
            Self::create_empty_session(pointer_scan_session, validation_target_address)
        } else {
            Self::build_pointer_scan_session(pointer_scan_session, validation_target_address, rebuilt_pointer_nodes_by_level)
        };

        if with_logging {
            let pointer_scan_summary = validated_pointer_scan_session.summarize();

            log::info!(
                "Pointer scan validation complete: roots={}, total_nodes={}, static_nodes={}, heap_nodes={}.",
                pointer_scan_summary.get_root_node_count(),
                pointer_scan_summary.get_total_node_count(),
                pointer_scan_summary.get_total_static_node_count(),
                pointer_scan_summary.get_total_heap_node_count(),
            );
        }

        validated_pointer_scan_session
    }

    fn validate_static_pointer_nodes_for_targets(
        process_info: &OpenedProcessInfo,
        static_pointer_scan_nodes: &[&PointerScanNode],
        required_target_addresses: &[u64],
        pointer_size: PointerScanPointerSize,
        offset_radius: u64,
        modules: &[NormalizedModule],
        scan_execution_context: &ScanExecutionContext,
    ) -> Vec<RebuiltPointerNode> {
        let mut rebuilt_pointer_nodes = Vec::new();

        for static_pointer_scan_node in static_pointer_scan_nodes {
            for module in modules
                .iter()
                .filter(|module| module.get_module_name() == static_pointer_scan_node.get_module_name())
            {
                let pointer_address = module
                    .get_base_address()
                    .saturating_add(static_pointer_scan_node.get_module_offset());

                let Some(pointer_value) = Self::read_pointer_value_at_address(process_info, scan_execution_context, pointer_address, pointer_size) else {
                    continue;
                };

                let lower_target_bound = pointer_value.saturating_sub(offset_radius);
                let upper_target_bound = pointer_value.saturating_add(offset_radius);
                let matching_target_start_index = required_target_addresses.partition_point(|target_address| *target_address < lower_target_bound);
                let matching_target_end_index = required_target_addresses.partition_point(|target_address| *target_address <= upper_target_bound);

                for required_target_address in &required_target_addresses[matching_target_start_index..matching_target_end_index] {
                    let Some(pointer_offset) = Self::calculate_pointer_offset(*required_target_address, pointer_value) else {
                        continue;
                    };

                    rebuilt_pointer_nodes.push(RebuiltPointerNode {
                        pointer_scan_node_type: PointerScanNodeType::Static,
                        pointer_address,
                        pointer_value,
                        resolved_target_address: *required_target_address,
                        pointer_offset,
                        module_name: module.get_module_name().to_string(),
                        module_offset: static_pointer_scan_node.get_module_offset(),
                    });
                }
            }
        }

        Self::sort_and_deduplicate_rebuilt_pointer_nodes(&mut rebuilt_pointer_nodes);

        rebuilt_pointer_nodes
    }

    fn scan_memory_regions_for_heap_pointer_nodes_by_target(
        process_info: &OpenedProcessInfo,
        required_target_addresses: &[u64],
        pointer_size: PointerScanPointerSize,
        offset_radius: u64,
        memory_regions: &[NormalizedRegion],
        modules: &[NormalizedModule],
        scan_execution_context: &ScanExecutionContext,
        with_logging: bool,
        validation_level_log_context: &PointerValidationLevelLogContext,
    ) -> Vec<RebuiltPointerNode> {
        let mut rebuilt_pointer_nodes = Vec::new();

        for (memory_region_index, memory_region) in memory_regions.iter().enumerate() {
            if with_logging && Self::should_log_memory_region_progress(memory_region_index, memory_regions.len()) {
                log::info!(
                    "Pointer scan validation level {}/{} (heap): region {}/{} at 0x{:X}.",
                    validation_level_log_context.level_number,
                    validation_level_log_context.level_count,
                    memory_region_index + 1,
                    memory_regions.len(),
                    memory_region.get_base_address(),
                );
            }

            Self::scan_memory_region_for_heap_pointer_nodes_by_target(
                process_info,
                memory_region,
                required_target_addresses,
                pointer_size,
                offset_radius,
                modules,
                scan_execution_context,
                &mut rebuilt_pointer_nodes,
            );

            if scan_execution_context.should_cancel() {
                break;
            }
        }

        Self::sort_and_deduplicate_rebuilt_pointer_nodes(&mut rebuilt_pointer_nodes);

        rebuilt_pointer_nodes
    }

    fn should_log_memory_region_progress(
        memory_region_index: usize,
        memory_region_count: usize,
    ) -> bool {
        if memory_region_count == 0 {
            return false;
        }

        memory_region_count <= 8 || memory_region_index == 0 || memory_region_index + 1 == memory_region_count || (memory_region_index + 1) % 128 == 0
    }

    fn scan_memory_region_for_heap_pointer_nodes_by_target(
        process_info: &OpenedProcessInfo,
        memory_region: &NormalizedRegion,
        required_target_addresses: &[u64],
        pointer_size: PointerScanPointerSize,
        offset_radius: u64,
        modules: &[NormalizedModule],
        scan_execution_context: &ScanExecutionContext,
        rebuilt_pointer_nodes: &mut Vec<RebuiltPointerNode>,
    ) {
        let pointer_size_in_bytes = pointer_size.get_size_in_bytes() as usize;
        let pointer_alignment = pointer_size_in_bytes as u64;
        let region_base_address = memory_region.get_base_address();
        let region_end_address = memory_region.get_end_address();
        let alignment_remainder = region_base_address % pointer_alignment;
        let mut scan_address = if alignment_remainder == 0 {
            region_base_address
        } else {
            region_base_address.saturating_add(pointer_alignment.saturating_sub(alignment_remainder))
        };

        if scan_address.saturating_add(pointer_size_in_bytes as u64) > region_end_address {
            return;
        }

        while scan_address.saturating_add(pointer_size_in_bytes as u64) <= region_end_address {
            let remaining_region_bytes = region_end_address.saturating_sub(scan_address) as usize;
            let mut scan_chunk_size = remaining_region_bytes.min(VALIDATION_SCAN_CHUNK_SIZE);
            scan_chunk_size -= scan_chunk_size % pointer_size_in_bytes;

            if scan_chunk_size < pointer_size_in_bytes {
                scan_chunk_size = pointer_size_in_bytes;
            }

            if scan_address.saturating_add(scan_chunk_size as u64) > region_end_address {
                scan_chunk_size = region_end_address.saturating_sub(scan_address) as usize;
                scan_chunk_size -= scan_chunk_size % pointer_size_in_bytes;
            }

            if scan_chunk_size < pointer_size_in_bytes {
                break;
            }

            let mut current_values = vec![0_u8; scan_chunk_size];
            let read_succeeded = scan_execution_context.read_bytes(process_info, scan_address, &mut current_values);

            if read_succeeded {
                let mut pointer_value_offset = 0_usize;

                while pointer_value_offset.saturating_add(pointer_size_in_bytes) <= current_values.len() {
                    let value_slice = &current_values[pointer_value_offset..pointer_value_offset + pointer_size_in_bytes];
                    let Some(pointer_value) = Self::read_pointer_value(value_slice, pointer_size) else {
                        pointer_value_offset = pointer_value_offset.saturating_add(pointer_size_in_bytes);
                        continue;
                    };

                    let lower_target_bound = pointer_value.saturating_sub(offset_radius);
                    let upper_target_bound = pointer_value.saturating_add(offset_radius);
                    let matching_target_start_index = required_target_addresses.partition_point(|target_address| *target_address < lower_target_bound);
                    let matching_target_end_index = required_target_addresses.partition_point(|target_address| *target_address <= upper_target_bound);

                    if matching_target_start_index < matching_target_end_index {
                        let pointer_address = scan_address.saturating_add(pointer_value_offset as u64);
                        let (pointer_scan_node_type, _module_name, _module_offset) = Self::classify_pointer_address(pointer_address, modules);

                        if pointer_scan_node_type == PointerScanNodeType::Heap {
                            for required_target_address in &required_target_addresses[matching_target_start_index..matching_target_end_index] {
                                let Some(pointer_offset) = Self::calculate_pointer_offset(*required_target_address, pointer_value) else {
                                    continue;
                                };

                                rebuilt_pointer_nodes.push(RebuiltPointerNode {
                                    pointer_scan_node_type: PointerScanNodeType::Heap,
                                    pointer_address,
                                    pointer_value,
                                    resolved_target_address: *required_target_address,
                                    pointer_offset,
                                    module_name: String::new(),
                                    module_offset: 0,
                                });
                            }
                        }
                    }

                    pointer_value_offset = pointer_value_offset.saturating_add(pointer_size_in_bytes);
                }
            }

            scan_address = scan_address.saturating_add(scan_chunk_size as u64);
        }
    }

    fn sort_and_deduplicate_rebuilt_pointer_nodes(rebuilt_pointer_nodes: &mut Vec<RebuiltPointerNode>) {
        rebuilt_pointer_nodes.sort_by(Self::compare_rebuilt_pointer_nodes);
        rebuilt_pointer_nodes.dedup();
    }

    fn read_pointer_value_at_address(
        process_info: &OpenedProcessInfo,
        scan_execution_context: &ScanExecutionContext,
        pointer_address: u64,
        pointer_size: PointerScanPointerSize,
    ) -> Option<u64> {
        let mut pointer_bytes = vec![0_u8; pointer_size.get_size_in_bytes() as usize];

        if !scan_execution_context.read_bytes(process_info, pointer_address, &mut pointer_bytes) {
            return None;
        }

        Self::read_pointer_value(&pointer_bytes, pointer_size)
    }

    fn build_pointer_scan_session(
        original_pointer_scan_session: &PointerScanSession,
        validation_target_address: u64,
        rebuilt_pointer_nodes_by_level: Vec<Vec<RebuiltPointerNode>>,
    ) -> PointerScanSession {
        let root_oriented_pointer_nodes_by_level = rebuilt_pointer_nodes_by_level
            .into_iter()
            .rev()
            .collect::<Vec<_>>();
        let discovery_level_count = root_oriented_pointer_nodes_by_level.len();
        let mut pointer_scan_nodes = Vec::new();
        let mut next_node_id = 1_u64;
        let mut total_static_node_count = 0_u64;
        let mut total_heap_node_count = 0_u64;

        for (pointer_level_index, rebuilt_pointer_nodes) in root_oriented_pointer_nodes_by_level.iter().enumerate() {
            let depth = pointer_level_index as u64 + 1;

            for rebuilt_pointer_node in rebuilt_pointer_nodes {
                let node_id = next_node_id;
                next_node_id = next_node_id.saturating_add(1);

                match rebuilt_pointer_node.pointer_scan_node_type {
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
                    rebuilt_pointer_node.pointer_scan_node_type,
                    depth,
                    rebuilt_pointer_node.pointer_address,
                    rebuilt_pointer_node.pointer_value,
                    rebuilt_pointer_node.resolved_target_address,
                    rebuilt_pointer_node.pointer_offset,
                    rebuilt_pointer_node.module_name.clone(),
                    rebuilt_pointer_node.module_offset,
                    Vec::new(),
                );
                pointer_scan_node.set_discovery_depth(discovery_level_count.saturating_sub(pointer_level_index) as u64);
                pointer_scan_nodes.push(pointer_scan_node);
            }
        }

        Self::attach_graph_child_node_ids(&mut pointer_scan_nodes);
        let (root_node_ids, pointer_scan_levels) = Self::build_root_oriented_levels(&mut pointer_scan_nodes);

        PointerScanSession::new(
            original_pointer_scan_session.get_session_id(),
            validation_target_address,
            original_pointer_scan_session.get_pointer_size(),
            original_pointer_scan_session.get_max_depth(),
            original_pointer_scan_session.get_offset_radius(),
            root_node_ids,
            pointer_scan_levels,
            pointer_scan_nodes,
            total_static_node_count,
            total_heap_node_count,
        )
    }

    fn attach_graph_child_node_ids(pointer_scan_nodes: &mut [PointerScanNode]) {
        let mut graph_child_node_ids_by_depth_and_pointer_address = HashMap::new();

        for pointer_scan_node in pointer_scan_nodes.iter() {
            graph_child_node_ids_by_depth_and_pointer_address
                .entry((pointer_scan_node.get_depth(), pointer_scan_node.get_pointer_address()))
                .or_insert_with(Vec::new)
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

    fn create_empty_session(
        original_pointer_scan_session: &PointerScanSession,
        validation_target_address: u64,
    ) -> PointerScanSession {
        PointerScanSession::new(
            original_pointer_scan_session.get_session_id(),
            validation_target_address,
            original_pointer_scan_session.get_pointer_size(),
            original_pointer_scan_session.get_max_depth(),
            original_pointer_scan_session.get_offset_radius(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            0,
            0,
        )
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

    fn compare_rebuilt_pointer_nodes(
        left_pointer_node: &RebuiltPointerNode,
        right_pointer_node: &RebuiltPointerNode,
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
}

#[cfg(test)]
mod tests {
    use super::{PointerScanValidator, PointerValidationLevelLogContext};
    use crate::pointer_scans::pointer_scan_executor_task::PointerScanExecutor;
    use crate::scanners::scan_execution_context::ScanExecutionContext;
    use squalr_engine_api::structures::memory::bitness::Bitness;
    use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
    use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_session::PointerScanSession;
    use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
    use squalr_engine_api::structures::scanning::plans::pointer_scan::pointer_scan_parameters::PointerScanParameters;
    use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
    use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    #[test]
    fn validate_scan_rebuilds_live_heap_nodes_and_prunes_invalid_static_roots() {
        let original_pointer_scan_session = build_original_pointer_scan_session();
        let validation_memory_map = Arc::new(build_validation_memory_map());
        let scan_execution_context = ScanExecutionContext::new(
            None,
            None,
            Some(Arc::new({
                let validation_memory_map = validation_memory_map.clone();

                move |_opened_process_info, address, values| read_memory_from_map(&validation_memory_map, address, values)
            })),
        );
        let mut validated_pointer_scan_session = PointerScanValidator::validate_scan(
            OpenedProcessInfo::new(7, "pointer-test".to_string(), 0, Bitness::Bit64, None),
            &original_pointer_scan_session,
            0x4010,
            &build_validation_memory_regions(),
            &[NormalizedModule::new("game.exe", 0x1000, 0x100)],
            &scan_execution_context,
            false,
        );

        assert_eq!(validated_pointer_scan_session.get_session_id(), original_pointer_scan_session.get_session_id());
        assert_eq!(validated_pointer_scan_session.get_target_address(), 0x4010);
        assert_eq!(validated_pointer_scan_session.get_root_node_ids().len(), 1);
        assert_eq!(validated_pointer_scan_session.get_total_node_count(), 2);
        assert_eq!(validated_pointer_scan_session.get_total_static_node_count(), 1);
        assert_eq!(validated_pointer_scan_session.get_total_heap_node_count(), 1);

        let root_nodes = validated_pointer_scan_session.get_expanded_nodes(None);
        assert_eq!(root_nodes.len(), 1);
        assert_eq!(root_nodes[0].get_pointer_address(), 0x1010);
        assert_eq!(root_nodes[0].get_pointer_scan_node_type(), PointerScanNodeType::Static);
        assert_eq!(root_nodes[0].get_resolved_target_address(), 0x3000);
        assert_eq!(root_nodes[0].get_pointer_offset(), 0x10);

        let child_nodes = validated_pointer_scan_session.get_expanded_nodes(Some(root_nodes[0].get_node_id()));
        assert_eq!(child_nodes.len(), 1);
        assert_eq!(child_nodes[0].get_pointer_address(), 0x3000);
        assert_eq!(child_nodes[0].get_pointer_scan_node_type(), PointerScanNodeType::Heap);
        assert_eq!(child_nodes[0].get_resolved_target_address(), 0x4010);
        assert_eq!(child_nodes[0].get_pointer_offset(), 0x10);
    }

    #[test]
    fn validate_scan_rebases_static_module_addresses_before_pruning() {
        let original_pointer_scan_session = build_original_pointer_scan_session();
        let rebased_validation_memory_map = Arc::new(build_rebased_validation_memory_map());
        let scan_execution_context = ScanExecutionContext::new(
            None,
            None,
            Some(Arc::new({
                let rebased_validation_memory_map = rebased_validation_memory_map.clone();

                move |_opened_process_info, address, values| read_memory_from_map(&rebased_validation_memory_map, address, values)
            })),
        );
        let mut validated_pointer_scan_session = PointerScanValidator::validate_scan(
            OpenedProcessInfo::new(7, "pointer-test".to_string(), 0, Bitness::Bit64, None),
            &original_pointer_scan_session,
            0x8010,
            &build_rebased_validation_memory_regions(),
            &[NormalizedModule::new("game.exe", 0x5000, 0x100)],
            &scan_execution_context,
            false,
        );

        assert_eq!(validated_pointer_scan_session.get_root_node_ids().len(), 1);
        assert_eq!(validated_pointer_scan_session.get_total_node_count(), 2);

        let root_nodes = validated_pointer_scan_session.get_expanded_nodes(None);
        assert_eq!(root_nodes.len(), 1);
        assert_eq!(root_nodes[0].get_pointer_address(), 0x5010);
        assert_eq!(root_nodes[0].get_module_name(), "game.exe");
        assert_eq!(root_nodes[0].get_module_offset(), 0x10);

        let child_nodes = validated_pointer_scan_session.get_expanded_nodes(Some(root_nodes[0].get_node_id()));
        assert_eq!(child_nodes.len(), 1);
        assert_eq!(child_nodes[0].get_pointer_address(), 0x7000);
        assert_eq!(child_nodes[0].get_pointer_scan_node_type(), PointerScanNodeType::Heap);
    }

    #[test]
    fn scan_memory_regions_for_heap_pointer_nodes_by_target_matches_multiple_targets_in_one_pass() {
        let multi_target_validation_memory_map = Arc::new(build_multi_target_validation_memory_map());
        let scan_execution_context = ScanExecutionContext::new(
            None,
            None,
            Some(Arc::new({
                let multi_target_validation_memory_map = multi_target_validation_memory_map.clone();

                move |_opened_process_info, address, values| read_memory_from_map(&multi_target_validation_memory_map, address, values)
            })),
        );
        let rebuilt_pointer_nodes = PointerScanValidator::scan_memory_regions_for_heap_pointer_nodes_by_target(
            &OpenedProcessInfo::new(7, "pointer-test".to_string(), 0, Bitness::Bit64, None),
            &[0x4010, 0x5010],
            PointerScanPointerSize::Pointer64,
            0x10,
            &[NormalizedRegion::new(0x3000, 0x40)],
            &[],
            &scan_execution_context,
            false,
            &PointerValidationLevelLogContext {
                level_number: 1,
                level_count: 1,
            },
        );

        assert_eq!(rebuilt_pointer_nodes.len(), 2);
        assert!(rebuilt_pointer_nodes.iter().any(|rebuilt_pointer_node| {
            rebuilt_pointer_node.pointer_address == 0x3000
                && rebuilt_pointer_node.resolved_target_address == 0x4010
                && rebuilt_pointer_node.pointer_offset == 0x10
        }));
        assert!(rebuilt_pointer_nodes.iter().any(|rebuilt_pointer_node| {
            rebuilt_pointer_node.pointer_address == 0x3008
                && rebuilt_pointer_node.resolved_target_address == 0x5010
                && rebuilt_pointer_node.pointer_offset == 0x10
        }));
    }

    fn build_original_pointer_scan_session() -> PointerScanSession {
        let original_memory_map = Arc::new(build_original_pointer_scan_memory_map());
        let scan_execution_context = ScanExecutionContext::new(
            None,
            None,
            Some(Arc::new({
                let original_memory_map = original_memory_map.clone();

                move |_opened_process_info, address, values| read_memory_from_map(&original_memory_map, address, values)
            })),
        );
        let snapshot = Arc::new(RwLock::new(build_pointer_scan_snapshot()));
        let pointer_scan_parameters = PointerScanParameters::new(0x3010, PointerScanPointerSize::Pointer64, 0x20, 3, true, false);

        PointerScanExecutor::execute_scan(
            OpenedProcessInfo::new(7, "pointer-test".to_string(), 0, Bitness::Bit64, None),
            snapshot.clone(),
            snapshot,
            41,
            pointer_scan_parameters,
            &[NormalizedModule::new("game.exe", 0x1000, 0x100)],
            false,
            &scan_execution_context,
        )
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

    fn build_original_pointer_scan_memory_map() -> HashMap<u64, u8> {
        let mut memory_map = HashMap::new();

        write_pointer_bytes(&mut memory_map, 0x1010, 0x1FF0_u64);
        write_pointer_bytes(&mut memory_map, 0x1030, 0x3020_u64);
        write_pointer_bytes(&mut memory_map, 0x2000, 0x3000_u64);

        memory_map
    }

    fn build_validation_memory_map() -> HashMap<u64, u8> {
        let mut memory_map = HashMap::new();

        write_pointer_bytes(&mut memory_map, 0x1010, 0x2FF0_u64);
        write_pointer_bytes(&mut memory_map, 0x1030, 0x3020_u64);
        write_pointer_bytes(&mut memory_map, 0x3000, 0x4000_u64);

        memory_map
    }

    fn build_validation_memory_regions() -> Vec<NormalizedRegion> {
        vec![
            NormalizedRegion::new(0x1000, 0x40),
            NormalizedRegion::new(0x3000, 0x40),
        ]
    }

    fn build_rebased_validation_memory_map() -> HashMap<u64, u8> {
        let mut memory_map = HashMap::new();

        write_pointer_bytes(&mut memory_map, 0x5010, 0x6FF0_u64);
        write_pointer_bytes(&mut memory_map, 0x5030, 0x3020_u64);
        write_pointer_bytes(&mut memory_map, 0x7000, 0x8000_u64);

        memory_map
    }

    fn build_rebased_validation_memory_regions() -> Vec<NormalizedRegion> {
        vec![
            NormalizedRegion::new(0x5000, 0x40),
            NormalizedRegion::new(0x7000, 0x40),
        ]
    }

    fn build_multi_target_validation_memory_map() -> HashMap<u64, u8> {
        let mut memory_map = HashMap::new();

        write_pointer_bytes(&mut memory_map, 0x3000, 0x4000_u64);
        write_pointer_bytes(&mut memory_map, 0x3008, 0x5000_u64);

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
