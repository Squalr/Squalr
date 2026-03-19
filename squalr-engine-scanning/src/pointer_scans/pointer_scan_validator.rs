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
use std::collections::HashMap;

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

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct PointerChainValidationStep {
    node_type_class: u8,
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
struct PointerValidationStepLogContext {
    signature_number: usize,
    signature_count: usize,
    step_number: usize,
    step_count: usize,
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

impl PointerChainValidationStep {
    fn from_node(pointer_scan_node: &PointerScanNode) -> Self {
        Self {
            node_type_class: Self::node_type_to_class(pointer_scan_node.get_pointer_scan_node_type()),
            module_name: pointer_scan_node.get_module_name().to_string(),
            module_offset: pointer_scan_node.get_module_offset(),
        }
    }

    fn to_pointer_scan_node_type(&self) -> PointerScanNodeType {
        match self.node_type_class {
            0 => PointerScanNodeType::Heap,
            _ => PointerScanNodeType::Static,
        }
    }

    fn node_type_to_class(pointer_scan_node_type: PointerScanNodeType) -> u8 {
        match pointer_scan_node_type {
            PointerScanNodeType::Heap => 0,
            PointerScanNodeType::Static => 1,
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

        let pointer_chain_validation_steps = Self::extract_pointer_chain_validation_steps(pointer_scan_session);

        if pointer_chain_validation_steps.is_empty() {
            return Self::create_empty_session(pointer_scan_session, validation_target_address);
        }

        if with_logging {
            log::info!(
                "Pointer scan validation will replay {} distinct pointer-chain signatures.",
                pointer_chain_validation_steps.len()
            );
        }

        let mut rebuilt_pointer_chains = Vec::new();

        for (signature_index, pointer_chain_validation_step) in pointer_chain_validation_steps.iter().enumerate() {
            if with_logging {
                log::info!(
                    "Pointer scan validation signature {}/{}: replaying {} steps.",
                    signature_index + 1,
                    pointer_chain_validation_steps.len(),
                    pointer_chain_validation_step.len(),
                );
            }

            rebuilt_pointer_chains.extend(Self::rebuild_pointer_chains_for_step_sequence(
                process_info.clone(),
                pointer_chain_validation_step,
                validation_target_address,
                pointer_scan_session.get_pointer_size(),
                pointer_scan_session.get_offset_radius(),
                memory_regions,
                modules,
                scan_execution_context,
                with_logging,
                signature_index + 1,
                pointer_chain_validation_steps.len(),
            ));

            if scan_execution_context.should_cancel() {
                break;
            }
        }

        Self::sort_and_deduplicate_pointer_chains(&mut rebuilt_pointer_chains);

        let validated_pointer_scan_session = if rebuilt_pointer_chains.is_empty() {
            Self::create_empty_session(pointer_scan_session, validation_target_address)
        } else {
            Self::build_pointer_scan_session(pointer_scan_session, validation_target_address, rebuilt_pointer_chains)
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

    fn extract_pointer_chain_validation_steps(pointer_scan_session: &PointerScanSession) -> Vec<Vec<PointerChainValidationStep>> {
        let pointer_scan_nodes_by_id = pointer_scan_session
            .get_pointer_scan_nodes()
            .iter()
            .map(|pointer_scan_node| (pointer_scan_node.get_node_id(), pointer_scan_node))
            .collect::<HashMap<_, _>>();
        let mut pointer_chain_validation_steps = Vec::new();
        let mut active_pointer_chain_validation_steps = Vec::new();

        for root_node_id in pointer_scan_session.get_root_node_ids() {
            Self::collect_pointer_chain_validation_steps(
                &pointer_scan_nodes_by_id,
                *root_node_id,
                &mut active_pointer_chain_validation_steps,
                &mut pointer_chain_validation_steps,
            );
        }

        pointer_chain_validation_steps.sort_by(Self::compare_pointer_chain_validation_steps);
        pointer_chain_validation_steps.dedup();
        pointer_chain_validation_steps
    }

    fn collect_pointer_chain_validation_steps(
        pointer_scan_nodes_by_id: &HashMap<u64, &PointerScanNode>,
        node_id: u64,
        active_pointer_chain_validation_steps: &mut Vec<PointerChainValidationStep>,
        pointer_chain_validation_steps: &mut Vec<Vec<PointerChainValidationStep>>,
    ) {
        let Some(pointer_scan_node) = pointer_scan_nodes_by_id.get(&node_id) else {
            return;
        };

        active_pointer_chain_validation_steps.push(PointerChainValidationStep::from_node(pointer_scan_node));

        if pointer_scan_node.get_child_node_ids().is_empty() {
            pointer_chain_validation_steps.push(active_pointer_chain_validation_steps.clone());
        } else {
            for child_node_id in pointer_scan_node.get_child_node_ids() {
                Self::collect_pointer_chain_validation_steps(
                    pointer_scan_nodes_by_id,
                    *child_node_id,
                    active_pointer_chain_validation_steps,
                    pointer_chain_validation_steps,
                );
            }
        }

        active_pointer_chain_validation_steps.pop();
    }

    fn rebuild_pointer_chains_for_step_sequence(
        process_info: OpenedProcessInfo,
        pointer_chain_validation_steps: &[PointerChainValidationStep],
        validation_target_address: u64,
        pointer_size: PointerScanPointerSize,
        offset_radius: u64,
        memory_regions: &[NormalizedRegion],
        modules: &[NormalizedModule],
        scan_execution_context: &ScanExecutionContext,
        with_logging: bool,
        signature_number: usize,
        signature_count: usize,
    ) -> Vec<Vec<RebuiltPointerNode>> {
        let Some(step_matches_by_required_target) = Self::collect_step_matches_for_pointer_chain(
            process_info,
            pointer_chain_validation_steps,
            validation_target_address,
            pointer_size,
            offset_radius,
            memory_regions,
            modules,
            scan_execution_context,
            with_logging,
            signature_number,
            signature_count,
        ) else {
            return Vec::new();
        };

        Self::rebuild_pointer_chains_from_step_matches(&step_matches_by_required_target)
    }

    fn collect_step_matches_for_pointer_chain(
        process_info: OpenedProcessInfo,
        pointer_chain_validation_steps: &[PointerChainValidationStep],
        validation_target_address: u64,
        pointer_size: PointerScanPointerSize,
        offset_radius: u64,
        memory_regions: &[NormalizedRegion],
        modules: &[NormalizedModule],
        scan_execution_context: &ScanExecutionContext,
        with_logging: bool,
        signature_number: usize,
        signature_count: usize,
    ) -> Option<Vec<HashMap<u64, Vec<RebuiltPointerNode>>>> {
        let mut step_matches_by_required_target_reversed = Vec::with_capacity(pointer_chain_validation_steps.len());
        let mut required_target_addresses = vec![validation_target_address];

        for (reverse_step_index, pointer_chain_validation_step) in pointer_chain_validation_steps.iter().rev().enumerate() {
            if scan_execution_context.should_cancel() {
                return None;
            }

            required_target_addresses.sort_unstable();
            required_target_addresses.dedup();

            if required_target_addresses.is_empty() {
                if with_logging {
                    log::info!(
                        "Pointer scan validation signature {}/{} stopped before step replay because no required targets remained.",
                        signature_number,
                        signature_count,
                    );
                }

                return None;
            }

            let validation_step_log_context = PointerValidationStepLogContext {
                signature_number,
                signature_count,
                step_number: reverse_step_index + 1,
                step_count: pointer_chain_validation_steps.len(),
            };
            let pointer_scan_node_type = pointer_chain_validation_step.to_pointer_scan_node_type();

            if with_logging {
                log::info!(
                    "Pointer scan validation signature {}/{} step {}/{} ({}): checking {} required targets.",
                    validation_step_log_context.signature_number,
                    validation_step_log_context.signature_count,
                    validation_step_log_context.step_number,
                    validation_step_log_context.step_count,
                    Self::format_pointer_scan_node_type(pointer_scan_node_type),
                    required_target_addresses.len(),
                );
            }

            let step_matches_by_required_target = match pointer_scan_node_type {
                PointerScanNodeType::Static => Self::validate_static_pointer_nodes_for_targets(
                    process_info.clone(),
                    pointer_chain_validation_step,
                    &required_target_addresses,
                    pointer_size,
                    offset_radius,
                    modules,
                    scan_execution_context,
                ),
                PointerScanNodeType::Heap => Self::scan_memory_regions_for_heap_pointer_nodes_by_target(
                    process_info.clone(),
                    &required_target_addresses,
                    pointer_size,
                    offset_radius,
                    memory_regions,
                    modules,
                    scan_execution_context,
                    with_logging,
                    &validation_step_log_context,
                ),
            };

            if step_matches_by_required_target.is_empty() {
                if with_logging {
                    log::info!(
                        "Pointer scan validation signature {}/{} step {}/{} ({}): found no matches.",
                        validation_step_log_context.signature_number,
                        validation_step_log_context.signature_count,
                        validation_step_log_context.step_number,
                        validation_step_log_context.step_count,
                        Self::format_pointer_scan_node_type(pointer_scan_node_type),
                    );
                }

                return None;
            }

            if with_logging {
                let rebuilt_pointer_node_count = step_matches_by_required_target
                    .values()
                    .map(Vec::len)
                    .sum::<usize>();

                log::info!(
                    "Pointer scan validation signature {}/{} step {}/{} ({}): matched {} targets and rebuilt {} nodes.",
                    validation_step_log_context.signature_number,
                    validation_step_log_context.signature_count,
                    validation_step_log_context.step_number,
                    validation_step_log_context.step_count,
                    Self::format_pointer_scan_node_type(pointer_scan_node_type),
                    step_matches_by_required_target.len(),
                    rebuilt_pointer_node_count,
                );
            }

            required_target_addresses = step_matches_by_required_target
                .values()
                .flat_map(|rebuilt_pointer_nodes| {
                    rebuilt_pointer_nodes
                        .iter()
                        .map(|rebuilt_pointer_node| rebuilt_pointer_node.pointer_address)
                })
                .collect();

            step_matches_by_required_target_reversed.push(step_matches_by_required_target);
        }

        step_matches_by_required_target_reversed.reverse();

        Some(step_matches_by_required_target_reversed)
    }

    fn rebuild_pointer_chains_from_step_matches(step_matches_by_required_target: &[HashMap<u64, Vec<RebuiltPointerNode>>]) -> Vec<Vec<RebuiltPointerNode>> {
        if step_matches_by_required_target.is_empty() {
            return Vec::new();
        }

        let rebuilt_pointer_nodes_by_pointer_address = step_matches_by_required_target
            .iter()
            .map(Self::build_rebuilt_pointer_nodes_by_pointer_address)
            .collect::<Vec<_>>();
        let mut rebuilt_pointer_chains = Vec::new();
        let mut active_rebuilt_pointer_chain = Vec::with_capacity(step_matches_by_required_target.len());

        for root_rebuilt_pointer_nodes in step_matches_by_required_target[0].values() {
            for root_rebuilt_pointer_node in root_rebuilt_pointer_nodes {
                Self::append_rebuilt_pointer_chains_from_node(
                    root_rebuilt_pointer_node.clone(),
                    0,
                    &rebuilt_pointer_nodes_by_pointer_address,
                    &mut active_rebuilt_pointer_chain,
                    &mut rebuilt_pointer_chains,
                );
            }
        }

        Self::sort_and_deduplicate_pointer_chains(&mut rebuilt_pointer_chains);

        rebuilt_pointer_chains
    }

    fn append_rebuilt_pointer_chains_from_node(
        rebuilt_pointer_node: RebuiltPointerNode,
        pointer_chain_step_index: usize,
        rebuilt_pointer_nodes_by_pointer_address: &[HashMap<u64, Vec<RebuiltPointerNode>>],
        active_rebuilt_pointer_chain: &mut Vec<RebuiltPointerNode>,
        rebuilt_pointer_chains: &mut Vec<Vec<RebuiltPointerNode>>,
    ) {
        active_rebuilt_pointer_chain.push(rebuilt_pointer_node.clone());

        if pointer_chain_step_index + 1 >= rebuilt_pointer_nodes_by_pointer_address.len() {
            rebuilt_pointer_chains.push(active_rebuilt_pointer_chain.clone());
            active_rebuilt_pointer_chain.pop();

            return;
        }

        if let Some(child_rebuilt_pointer_nodes) =
            rebuilt_pointer_nodes_by_pointer_address[pointer_chain_step_index + 1].get(&rebuilt_pointer_node.resolved_target_address)
        {
            for child_rebuilt_pointer_node in child_rebuilt_pointer_nodes {
                Self::append_rebuilt_pointer_chains_from_node(
                    child_rebuilt_pointer_node.clone(),
                    pointer_chain_step_index + 1,
                    rebuilt_pointer_nodes_by_pointer_address,
                    active_rebuilt_pointer_chain,
                    rebuilt_pointer_chains,
                );
            }
        }

        active_rebuilt_pointer_chain.pop();
    }

    fn build_rebuilt_pointer_nodes_by_pointer_address(
        step_matches_by_required_target: &HashMap<u64, Vec<RebuiltPointerNode>>
    ) -> HashMap<u64, Vec<RebuiltPointerNode>> {
        let mut rebuilt_pointer_nodes_by_pointer_address = HashMap::new();

        for rebuilt_pointer_nodes in step_matches_by_required_target.values() {
            for rebuilt_pointer_node in rebuilt_pointer_nodes {
                rebuilt_pointer_nodes_by_pointer_address
                    .entry(rebuilt_pointer_node.pointer_address)
                    .or_insert_with(Vec::new)
                    .push(rebuilt_pointer_node.clone());
            }
        }

        Self::sort_and_deduplicate_rebuilt_pointer_node_map(&mut rebuilt_pointer_nodes_by_pointer_address);

        rebuilt_pointer_nodes_by_pointer_address
    }

    fn validate_static_pointer_nodes_for_targets(
        process_info: OpenedProcessInfo,
        pointer_chain_validation_step: &PointerChainValidationStep,
        required_target_addresses: &[u64],
        pointer_size: PointerScanPointerSize,
        offset_radius: u64,
        modules: &[NormalizedModule],
        scan_execution_context: &ScanExecutionContext,
    ) -> HashMap<u64, Vec<RebuiltPointerNode>> {
        let mut rebuilt_pointer_nodes_by_required_target = HashMap::new();

        for module in modules
            .iter()
            .filter(|module| module.get_module_name() == pointer_chain_validation_step.module_name)
        {
            let pointer_address = module
                .get_base_address()
                .saturating_add(pointer_chain_validation_step.module_offset);

            let Some(pointer_value) = Self::read_pointer_value_at_address(&process_info, scan_execution_context, pointer_address, pointer_size) else {
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

                rebuilt_pointer_nodes_by_required_target
                    .entry(*required_target_address)
                    .or_insert_with(Vec::new)
                    .push(RebuiltPointerNode {
                        pointer_scan_node_type: PointerScanNodeType::Static,
                        pointer_address,
                        pointer_value,
                        resolved_target_address: *required_target_address,
                        pointer_offset,
                        module_name: module.get_module_name().to_string(),
                        module_offset: pointer_chain_validation_step.module_offset,
                    });
            }
        }

        Self::sort_and_deduplicate_rebuilt_pointer_node_map(&mut rebuilt_pointer_nodes_by_required_target);

        rebuilt_pointer_nodes_by_required_target
    }

    fn scan_memory_regions_for_heap_pointer_nodes_by_target(
        process_info: OpenedProcessInfo,
        required_target_addresses: &[u64],
        pointer_size: PointerScanPointerSize,
        offset_radius: u64,
        memory_regions: &[NormalizedRegion],
        modules: &[NormalizedModule],
        scan_execution_context: &ScanExecutionContext,
        with_logging: bool,
        validation_step_log_context: &PointerValidationStepLogContext,
    ) -> HashMap<u64, Vec<RebuiltPointerNode>> {
        let mut rebuilt_pointer_nodes_by_required_target = HashMap::new();

        if with_logging {
            log::info!(
                "Pointer scan validation signature {}/{} step {}/{} (heap): scanning {} memory regions.",
                validation_step_log_context.signature_number,
                validation_step_log_context.signature_count,
                validation_step_log_context.step_number,
                validation_step_log_context.step_count,
                memory_regions.len(),
            );
        }

        for (memory_region_index, memory_region) in memory_regions.iter().enumerate() {
            if with_logging && Self::should_log_memory_region_progress(memory_region_index, memory_regions.len()) {
                log::info!(
                    "Pointer scan validation signature {}/{} step {}/{} (heap): region {}/{} at 0x{:X}.",
                    validation_step_log_context.signature_number,
                    validation_step_log_context.signature_count,
                    validation_step_log_context.step_number,
                    validation_step_log_context.step_count,
                    memory_region_index + 1,
                    memory_regions.len(),
                    memory_region.get_base_address(),
                );
            }

            Self::scan_memory_region_for_heap_pointer_nodes_by_target(
                &process_info,
                memory_region,
                required_target_addresses,
                pointer_size,
                offset_radius,
                modules,
                scan_execution_context,
                &mut rebuilt_pointer_nodes_by_required_target,
            );

            if scan_execution_context.should_cancel() {
                break;
            }
        }

        Self::sort_and_deduplicate_rebuilt_pointer_node_map(&mut rebuilt_pointer_nodes_by_required_target);

        rebuilt_pointer_nodes_by_required_target
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
        rebuilt_pointer_nodes_by_required_target: &mut HashMap<u64, Vec<RebuiltPointerNode>>,
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

                                rebuilt_pointer_nodes_by_required_target
                                    .entry(*required_target_address)
                                    .or_insert_with(Vec::new)
                                    .push(RebuiltPointerNode {
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

    fn sort_and_deduplicate_rebuilt_pointer_node_map(rebuilt_pointer_nodes_by_target: &mut HashMap<u64, Vec<RebuiltPointerNode>>) {
        for rebuilt_pointer_nodes in rebuilt_pointer_nodes_by_target.values_mut() {
            rebuilt_pointer_nodes.sort_by(Self::compare_rebuilt_pointer_nodes);
            rebuilt_pointer_nodes.dedup();
        }
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
        rebuilt_pointer_chains: Vec<Vec<RebuiltPointerNode>>,
    ) -> PointerScanSession {
        let mut pointer_scan_nodes = Vec::new();
        let mut root_node_ids = Vec::new();
        let mut level_accumulators = Vec::new();
        let mut next_node_id = 1_u64;
        let mut total_static_node_count = 0_u64;
        let mut total_heap_node_count = 0_u64;

        for rebuilt_pointer_chain in rebuilt_pointer_chains {
            let chain_node_ids = (0..rebuilt_pointer_chain.len())
                .map(|_| {
                    let allocated_node_id = next_node_id;
                    next_node_id = next_node_id.saturating_add(1);

                    allocated_node_id
                })
                .collect::<Vec<_>>();

            if let Some(root_node_id) = chain_node_ids.first().copied() {
                root_node_ids.push(root_node_id);
            }

            for (pointer_chain_index, rebuilt_pointer_node) in rebuilt_pointer_chain.iter().enumerate() {
                while level_accumulators.len() <= pointer_chain_index {
                    level_accumulators.push(PointerScanLevelAccumulator::default());
                }

                let depth = pointer_chain_index as u64 + 1;
                let node_id = chain_node_ids[pointer_chain_index];
                let parent_node_id = if pointer_chain_index == 0 {
                    None
                } else {
                    Some(chain_node_ids[pointer_chain_index - 1])
                };
                let child_node_ids = if pointer_chain_index + 1 < chain_node_ids.len() {
                    vec![chain_node_ids[pointer_chain_index + 1]]
                } else {
                    Vec::new()
                };

                level_accumulators[pointer_chain_index].track_node(node_id, rebuilt_pointer_node.pointer_scan_node_type);

                match rebuilt_pointer_node.pointer_scan_node_type {
                    PointerScanNodeType::Static => {
                        total_static_node_count = total_static_node_count.saturating_add(1);
                    }
                    PointerScanNodeType::Heap => {
                        total_heap_node_count = total_heap_node_count.saturating_add(1);
                    }
                }

                pointer_scan_nodes.push(PointerScanNode::new(
                    node_id,
                    parent_node_id,
                    rebuilt_pointer_node.pointer_scan_node_type,
                    depth,
                    rebuilt_pointer_node.pointer_address,
                    rebuilt_pointer_node.pointer_value,
                    rebuilt_pointer_node.resolved_target_address,
                    rebuilt_pointer_node.pointer_offset,
                    rebuilt_pointer_node.module_name.clone(),
                    rebuilt_pointer_node.module_offset,
                    child_node_ids,
                ));
            }
        }

        let pointer_scan_levels = level_accumulators
            .into_iter()
            .enumerate()
            .map(|(pointer_chain_index, level_accumulator)| {
                PointerScanLevel::new(
                    pointer_chain_index as u64 + 1,
                    level_accumulator.node_ids,
                    level_accumulator.static_node_count,
                    level_accumulator.heap_node_count,
                )
            })
            .collect();

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

    fn format_pointer_scan_node_type(pointer_scan_node_type: PointerScanNodeType) -> &'static str {
        match pointer_scan_node_type {
            PointerScanNodeType::Heap => "heap",
            PointerScanNodeType::Static => "static",
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

    fn sort_and_deduplicate_pointer_chains(pointer_chains: &mut Vec<Vec<RebuiltPointerNode>>) {
        pointer_chains.sort_by(Self::compare_pointer_chains);
        pointer_chains.dedup();
    }

    fn compare_pointer_chains(
        left_pointer_chain: &Vec<RebuiltPointerNode>,
        right_pointer_chain: &Vec<RebuiltPointerNode>,
    ) -> Ordering {
        left_pointer_chain
            .len()
            .cmp(&right_pointer_chain.len())
            .then_with(|| {
                left_pointer_chain
                    .iter()
                    .zip(right_pointer_chain.iter())
                    .map(|(left_pointer_node, right_pointer_node)| Self::compare_rebuilt_pointer_nodes(left_pointer_node, right_pointer_node))
                    .find(|ordering| *ordering != Ordering::Equal)
                    .unwrap_or(Ordering::Equal)
            })
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
                PointerChainValidationStep::node_type_to_class(left_pointer_node.pointer_scan_node_type)
                    .cmp(&PointerChainValidationStep::node_type_to_class(right_pointer_node.pointer_scan_node_type))
            })
    }

    fn compare_pointer_chain_validation_steps(
        left_pointer_chain_validation_steps: &Vec<PointerChainValidationStep>,
        right_pointer_chain_validation_steps: &Vec<PointerChainValidationStep>,
    ) -> Ordering {
        left_pointer_chain_validation_steps
            .len()
            .cmp(&right_pointer_chain_validation_steps.len())
            .then_with(|| {
                left_pointer_chain_validation_steps
                    .iter()
                    .zip(right_pointer_chain_validation_steps.iter())
                    .map(|(left_validation_step, right_validation_step)| {
                        left_validation_step
                            .node_type_class
                            .cmp(&right_validation_step.node_type_class)
                            .then_with(|| {
                                left_validation_step
                                    .module_name
                                    .cmp(&right_validation_step.module_name)
                            })
                            .then_with(|| {
                                left_validation_step
                                    .module_offset
                                    .cmp(&right_validation_step.module_offset)
                            })
                    })
                    .find(|ordering| *ordering != Ordering::Equal)
                    .unwrap_or(Ordering::Equal)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{PointerScanValidator, PointerValidationStepLogContext};
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
        let validated_pointer_scan_session = PointerScanValidator::validate_scan(
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
        let validated_pointer_scan_session = PointerScanValidator::validate_scan(
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
        let rebuilt_pointer_nodes_by_target = PointerScanValidator::scan_memory_regions_for_heap_pointer_nodes_by_target(
            OpenedProcessInfo::new(7, "pointer-test".to_string(), 0, Bitness::Bit64, None),
            &[0x4010, 0x5010],
            PointerScanPointerSize::Pointer64,
            0x10,
            &[NormalizedRegion::new(0x3000, 0x40)],
            &[],
            &scan_execution_context,
            false,
            &PointerValidationStepLogContext {
                signature_number: 1,
                signature_count: 1,
                step_number: 1,
                step_count: 1,
            },
        );

        let first_target_matches = rebuilt_pointer_nodes_by_target
            .get(&0x4010)
            .expect("Expected matches for the first validation target.");
        assert_eq!(first_target_matches.len(), 1);
        assert_eq!(first_target_matches[0].pointer_address, 0x3000);
        assert_eq!(first_target_matches[0].pointer_offset, 0x10);

        let second_target_matches = rebuilt_pointer_nodes_by_target
            .get(&0x5010)
            .expect("Expected matches for the second validation target.");
        assert_eq!(second_target_matches.len(), 1);
        assert_eq!(second_target_matches[0].pointer_address, 0x3008);
        assert_eq!(second_target_matches[0].pointer_offset, 0x10);
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
