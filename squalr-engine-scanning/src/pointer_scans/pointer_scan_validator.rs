use crate::pointer_scans::pointer_scan_range_search_kernel::PointerScanRangeSearchKernel;
use crate::pointer_scans::pointer_scan_root_tracker::PointerScanRootTracker;
use crate::pointer_scans::pointer_scan_target_ranges::PointerScanTargetRangeSet;
use crate::pointer_scans::structures::pointer_validation_level_log_context::PointerValidationLevelLogContext;
use crate::pointer_scans::structures::rebuilt_pointer_candidate::RebuiltPointerCandidate;
use crate::pointer_scans::structures::rebuilt_pointer_level::RebuiltPointerLevel;
use crate::scanners::scan_execution_context::ScanExecutionContext;
use rayon::prelude::*;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
use squalr_engine_api::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;
use squalr_engine_api::structures::pointer_scans::pointer_scan_level::PointerScanLevel;
use squalr_engine_api::structures::pointer_scans::pointer_scan_level_candidates::PointerScanLevelCandidates;
use squalr_engine_api::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::pointer_scans::pointer_scan_session::PointerScanSession;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::mem::size_of;
use std::time::Instant;

const VALIDATION_SCAN_CHUNK_SIZE: usize = 64 * 1024;

pub struct PointerScanValidator;

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
        let total_start_time = Instant::now();

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
        let mut rebuilt_pointer_levels = Vec::new();
        let level_count = pointer_scan_session.get_pointer_scan_level_candidates().len();
        let heap_memory_regions = Self::build_heap_memory_regions(memory_regions, modules);

        for level_index in 0..level_count {
            let level_number = level_index + 1;
            let is_terminal_level = level_number >= level_count;
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

            let required_target_ranges =
                PointerScanTargetRangeSet::from_sorted_target_addresses(&required_target_addresses, pointer_scan_session.get_offset_radius());
            let range_search_kernel = PointerScanRangeSearchKernel::new(&required_target_ranges, pointer_scan_session.get_pointer_size());
            let validation_level_log_context = PointerValidationLevelLogContext { level_number, level_count };
            let level_start_time = Instant::now();
            let static_pointer_scan_candidates = pointer_scan_session
                .get_pointer_scan_level_candidates()
                .get(level_index)
                .map(PointerScanLevelCandidates::get_static_candidates)
                .cloned()
                .unwrap_or_default();

            if with_logging {
                log::info!(
                    "Pointer scan validation level {}/{}: checking {} static nodes and scanning {} heap memory regions for {} frontier targets merged into {} ranges with {} kernel.",
                    validation_level_log_context.level_number,
                    validation_level_log_context.level_count,
                    static_pointer_scan_candidates.len(),
                    heap_memory_regions.len(),
                    required_target_addresses.len(),
                    required_target_ranges.get_range_count(),
                    range_search_kernel.get_name(),
                );
            }

            let rebuilt_static_candidates = Self::validate_static_pointer_candidates_for_targets(
                &process_info,
                &static_pointer_scan_candidates,
                &range_search_kernel,
                modules,
                scan_execution_context,
            );
            let rebuilt_heap_candidates = if is_terminal_level {
                Vec::new()
            } else {
                Self::scan_memory_regions_for_heap_pointer_candidates_by_target(
                    &process_info,
                    &range_search_kernel,
                    &heap_memory_regions,
                    scan_execution_context,
                    with_logging,
                    &validation_level_log_context,
                )
            };

            if rebuilt_static_candidates.is_empty() && rebuilt_heap_candidates.is_empty() {
                if with_logging {
                    log::info!(
                        "Pointer scan validation stopped after level {} because no validated nodes remained.",
                        validation_level_log_context.level_number
                    );
                }

                break;
            }

            if with_logging {
                log::info!(
                    "Pointer scan validation level {}/{} complete in {:?}: retained {} static nodes, rebuilt {} heap nodes, and produced {} next frontier targets.",
                    validation_level_log_context.level_number,
                    validation_level_log_context.level_count,
                    level_start_time.elapsed(),
                    rebuilt_static_candidates.len(),
                    rebuilt_heap_candidates.len(),
                    if is_terminal_level { 0 } else { rebuilt_heap_candidates.len() },
                );
            }

            required_target_addresses = rebuilt_heap_candidates
                .iter()
                .map(|rebuilt_pointer_candidate| rebuilt_pointer_candidate.pointer_address)
                .collect();
            rebuilt_pointer_levels.push(RebuiltPointerLevel {
                static_candidates: rebuilt_static_candidates,
                heap_candidates: rebuilt_heap_candidates,
            });
        }

        let validated_pointer_scan_session = if rebuilt_pointer_levels.is_empty() {
            Self::create_empty_session(pointer_scan_session, validation_target_address)
        } else {
            Self::build_pointer_scan_session(pointer_scan_session, validation_target_address, rebuilt_pointer_levels)
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
            log::info!("Total pointer scan validation time: {:?}", total_start_time.elapsed());
        }

        validated_pointer_scan_session
    }

    fn validate_static_pointer_candidates_for_targets(
        process_info: &OpenedProcessInfo,
        static_pointer_scan_candidates: &[PointerScanCandidate],
        range_search_kernel: &PointerScanRangeSearchKernel<'_>,
        modules: &[NormalizedModule],
        scan_execution_context: &ScanExecutionContext,
    ) -> Vec<RebuiltPointerCandidate> {
        let mut rebuilt_pointer_candidates = Vec::new();
        let modules_by_name = Self::group_modules_by_name(modules);

        for static_pointer_scan_candidate in static_pointer_scan_candidates {
            let Some(candidate_modules) = modules_by_name.get(static_pointer_scan_candidate.get_module_name()) else {
                continue;
            };

            for module in candidate_modules {
                let pointer_address = module
                    .get_base_address()
                    .saturating_add(static_pointer_scan_candidate.get_module_offset());

                let Some(pointer_value) =
                    Self::read_pointer_value_at_address(process_info, scan_execution_context, pointer_address, range_search_kernel.get_pointer_size())
                else {
                    continue;
                };

                if range_search_kernel.contains_pointer_value(pointer_value) {
                    rebuilt_pointer_candidates.push(RebuiltPointerCandidate {
                        pointer_scan_node_type: PointerScanNodeType::Static,
                        pointer_address,
                        pointer_value,
                        module_name: module.get_module_name().to_string(),
                        module_offset: static_pointer_scan_candidate.get_module_offset(),
                    });
                }
            }
        }

        Self::sort_and_deduplicate_rebuilt_pointer_candidates(&mut rebuilt_pointer_candidates);

        rebuilt_pointer_candidates
    }

    fn scan_memory_regions_for_heap_pointer_candidates_by_target(
        process_info: &OpenedProcessInfo,
        range_search_kernel: &PointerScanRangeSearchKernel<'_>,
        memory_regions: &[NormalizedRegion],
        scan_execution_context: &ScanExecutionContext,
        with_logging: bool,
        validation_level_log_context: &PointerValidationLevelLogContext,
    ) -> Vec<RebuiltPointerCandidate> {
        let mut rebuilt_pointer_candidates = memory_regions
            .par_iter()
            .enumerate()
            .fold(
                || (Vec::new(), vec![0_u8; VALIDATION_SCAN_CHUNK_SIZE]),
                |(mut worker_rebuilt_pointer_candidates, mut worker_scan_buffer), (memory_region_index, memory_region)| {
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

                    Self::scan_memory_region_for_heap_pointer_candidates_by_target(
                        process_info,
                        memory_region,
                        range_search_kernel,
                        scan_execution_context,
                        &mut worker_scan_buffer,
                        &mut worker_rebuilt_pointer_candidates,
                    );

                    (worker_rebuilt_pointer_candidates, worker_scan_buffer)
                },
            )
            .map(|(worker_rebuilt_pointer_candidates, _worker_scan_buffer)| worker_rebuilt_pointer_candidates)
            .reduce(Vec::new, |mut left_candidates, mut right_candidates| {
                left_candidates.append(&mut right_candidates);
                left_candidates
            });

        Self::sort_and_deduplicate_rebuilt_pointer_candidates(&mut rebuilt_pointer_candidates);

        rebuilt_pointer_candidates
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

    fn scan_memory_region_for_heap_pointer_candidates_by_target(
        process_info: &OpenedProcessInfo,
        memory_region: &NormalizedRegion,
        range_search_kernel: &PointerScanRangeSearchKernel<'_>,
        scan_execution_context: &ScanExecutionContext,
        scan_buffer: &mut Vec<u8>,
        rebuilt_pointer_candidates: &mut Vec<RebuiltPointerCandidate>,
    ) {
        let pointer_size_in_bytes = range_search_kernel.get_pointer_size().get_size_in_bytes() as usize;
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

            let current_values = &mut scan_buffer[..scan_chunk_size];
            let read_succeeded = scan_execution_context.read_bytes(process_info, scan_address, current_values);

            if read_succeeded {
                range_search_kernel.scan_region_with_visitor(scan_address, current_values, |pointer_match| {
                    rebuilt_pointer_candidates.push(RebuiltPointerCandidate {
                        pointer_scan_node_type: PointerScanNodeType::Heap,
                        pointer_address: pointer_match.get_pointer_address(),
                        pointer_value: pointer_match.get_pointer_value(),
                        module_name: String::new(),
                        module_offset: 0,
                    });
                });
            }

            if scan_execution_context.should_cancel() {
                break;
            }

            scan_address = scan_address.saturating_add(scan_chunk_size as u64);
        }
    }

    fn sort_and_deduplicate_rebuilt_pointer_candidates(rebuilt_pointer_candidates: &mut Vec<RebuiltPointerCandidate>) {
        rebuilt_pointer_candidates.par_sort_unstable_by(Self::compare_rebuilt_pointer_candidates);
        rebuilt_pointer_candidates.dedup();
    }

    fn read_pointer_value_at_address(
        process_info: &OpenedProcessInfo,
        scan_execution_context: &ScanExecutionContext,
        pointer_address: u64,
        pointer_size: PointerScanPointerSize,
    ) -> Option<u64> {
        let pointer_byte_count = pointer_size.get_size_in_bytes() as usize;
        let mut pointer_bytes = [0_u8; size_of::<u64>()];

        if !scan_execution_context.read_bytes(process_info, pointer_address, &mut pointer_bytes[..pointer_byte_count]) {
            return None;
        }

        match pointer_size {
            PointerScanPointerSize::Pointer32 => Some(u32::from_le_bytes(pointer_bytes[..size_of::<u32>()].try_into().ok()?) as u64),
            PointerScanPointerSize::Pointer64 => Some(u64::from_le_bytes(pointer_bytes)),
        }
    }

    fn build_pointer_scan_session(
        original_pointer_scan_session: &PointerScanSession,
        validation_target_address: u64,
        rebuilt_pointer_levels: Vec<RebuiltPointerLevel>,
    ) -> PointerScanSession {
        let mut pointer_scan_levels = Vec::new();
        let mut pointer_scan_level_candidates = Vec::new();
        let mut next_candidate_id = 1_u64;
        let mut total_static_node_count = 0_u64;
        let mut total_heap_node_count = 0_u64;
        let mut root_tracker = PointerScanRootTracker::new(original_pointer_scan_session.get_offset_radius());

        for (level_index, rebuilt_pointer_level) in rebuilt_pointer_levels.iter().enumerate() {
            let discovery_depth = level_index as u64 + 1;
            let mut static_candidates = Vec::with_capacity(rebuilt_pointer_level.static_candidates.len());

            for rebuilt_pointer_candidate in &rebuilt_pointer_level.static_candidates {
                root_tracker.record_static_candidate(discovery_depth, rebuilt_pointer_candidate.pointer_value);
                static_candidates.push(PointerScanCandidate::new(
                    next_candidate_id,
                    discovery_depth,
                    PointerScanNodeType::Static,
                    rebuilt_pointer_candidate.pointer_address,
                    rebuilt_pointer_candidate.pointer_value,
                    rebuilt_pointer_candidate.module_name.clone(),
                    rebuilt_pointer_candidate.module_offset,
                ));
                next_candidate_id = next_candidate_id.saturating_add(1);
            }

            let mut heap_candidates = Vec::with_capacity(rebuilt_pointer_level.heap_candidates.len());

            for rebuilt_pointer_candidate in &rebuilt_pointer_level.heap_candidates {
                heap_candidates.push(PointerScanCandidate::new(
                    next_candidate_id,
                    discovery_depth,
                    PointerScanNodeType::Heap,
                    rebuilt_pointer_candidate.pointer_address,
                    rebuilt_pointer_candidate.pointer_value,
                    String::new(),
                    0,
                ));
                next_candidate_id = next_candidate_id.saturating_add(1);
            }

            let level_candidates = PointerScanLevelCandidates::new(discovery_depth, static_candidates, heap_candidates);

            total_static_node_count = total_static_node_count.saturating_add(level_candidates.get_static_node_count());
            total_heap_node_count = total_heap_node_count.saturating_add(level_candidates.get_heap_node_count());
            pointer_scan_levels.push(PointerScanLevel::new(
                discovery_depth,
                level_candidates.get_node_count(),
                level_candidates.get_static_node_count(),
                level_candidates.get_heap_node_count(),
            ));
            root_tracker.advance_to_next_level(level_candidates.get_heap_candidates());
            pointer_scan_level_candidates.push(level_candidates);
        }

        let root_node_count = root_tracker.get_root_node_count();

        PointerScanSession::new(
            original_pointer_scan_session.get_session_id(),
            validation_target_address,
            original_pointer_scan_session.get_pointer_size(),
            original_pointer_scan_session.get_max_depth(),
            original_pointer_scan_session.get_offset_radius(),
            pointer_scan_levels,
            pointer_scan_level_candidates,
            root_node_count,
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
            0,
            0,
            0,
        )
    }

    fn build_heap_memory_regions(
        memory_regions: &[NormalizedRegion],
        modules: &[NormalizedModule],
    ) -> Vec<NormalizedRegion> {
        if modules.is_empty() {
            return memory_regions.to_vec();
        }

        let mut module_regions = modules
            .iter()
            .map(NormalizedModule::get_base_region)
            .cloned()
            .collect::<Vec<_>>();
        module_regions.sort_by_key(NormalizedRegion::get_base_address);

        let mut heap_memory_regions = Vec::new();

        for memory_region in memory_regions {
            let mut next_heap_region_base_address = memory_region.get_base_address();
            let memory_region_end_address = memory_region.get_end_address();

            for module_region in &module_regions {
                let module_base_address = module_region.get_base_address();
                let module_end_address = module_region.get_end_address();

                if module_end_address <= next_heap_region_base_address {
                    continue;
                }

                if module_base_address >= memory_region_end_address {
                    break;
                }

                if next_heap_region_base_address < module_base_address {
                    heap_memory_regions.push(NormalizedRegion::new(
                        next_heap_region_base_address,
                        module_base_address.saturating_sub(next_heap_region_base_address),
                    ));
                }

                next_heap_region_base_address = next_heap_region_base_address.max(module_end_address);

                if next_heap_region_base_address >= memory_region_end_address {
                    break;
                }
            }

            if next_heap_region_base_address < memory_region_end_address {
                heap_memory_regions.push(NormalizedRegion::new(
                    next_heap_region_base_address,
                    memory_region_end_address.saturating_sub(next_heap_region_base_address),
                ));
            }
        }

        heap_memory_regions
    }

    fn group_modules_by_name<'a>(modules: &'a [NormalizedModule]) -> HashMap<&'a str, Vec<&'a NormalizedModule>> {
        let mut modules_by_name = HashMap::new();

        for module in modules {
            modules_by_name
                .entry(module.get_module_name())
                .or_insert_with(Vec::new)
                .push(module);
        }

        modules_by_name
    }

    fn compare_rebuilt_pointer_candidates(
        left_pointer_node: &RebuiltPointerCandidate,
        right_pointer_node: &RebuiltPointerCandidate,
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
    use crate::pointer_scans::pointer_scan_range_search_kernel::PointerScanRangeSearchKernel;
    use crate::pointer_scans::pointer_scan_target_ranges::PointerScanTargetRangeSet;
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
        assert_eq!(validated_pointer_scan_session.get_root_node_count(), 1);
        assert_eq!(validated_pointer_scan_session.get_total_node_count(), 2);
        assert_eq!(validated_pointer_scan_session.get_total_static_node_count(), 1);
        assert_eq!(validated_pointer_scan_session.get_total_heap_node_count(), 1);

        let root_nodes = validated_pointer_scan_session.get_expanded_nodes(None);
        assert_eq!(root_nodes.len(), 1);
        assert_eq!(root_nodes[0].get_pointer_address(), 0x1010);
        assert_eq!(root_nodes[0].get_pointer_scan_node_type(), PointerScanNodeType::Static);
        assert_eq!(root_nodes[0].get_resolved_target_address(), 0x2FF0);
        assert_eq!(root_nodes[0].get_pointer_offset(), 0);

        let child_nodes = validated_pointer_scan_session.get_expanded_nodes(Some(root_nodes[0].get_node_id()));
        assert_eq!(child_nodes.len(), 1);
        assert_eq!(child_nodes[0].get_pointer_address(), 0x1010);
        assert_eq!(child_nodes[0].get_pointer_scan_node_type(), PointerScanNodeType::Static);
        assert_eq!(child_nodes[0].get_resolved_target_address(), 0x3000);
        assert_eq!(child_nodes[0].get_pointer_offset(), 0x10);

        let grandchild_nodes = validated_pointer_scan_session.get_expanded_nodes(Some(child_nodes[0].get_node_id()));
        assert_eq!(grandchild_nodes.len(), 1);
        assert_eq!(grandchild_nodes[0].get_pointer_address(), 0x3000);
        assert_eq!(grandchild_nodes[0].get_pointer_scan_node_type(), PointerScanNodeType::Heap);
        assert_eq!(grandchild_nodes[0].get_resolved_target_address(), 0x4010);
        assert_eq!(grandchild_nodes[0].get_pointer_offset(), 0x10);
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

        assert_eq!(validated_pointer_scan_session.get_root_node_count(), 1);
        assert_eq!(validated_pointer_scan_session.get_total_node_count(), 2);

        let root_nodes = validated_pointer_scan_session.get_expanded_nodes(None);
        assert_eq!(root_nodes.len(), 1);
        assert_eq!(root_nodes[0].get_pointer_address(), 0x5010);
        assert_eq!(root_nodes[0].get_module_name(), "game.exe");
        assert_eq!(root_nodes[0].get_module_offset(), 0x10);

        let child_nodes = validated_pointer_scan_session.get_expanded_nodes(Some(root_nodes[0].get_node_id()));
        assert_eq!(child_nodes.len(), 1);
        assert_eq!(child_nodes[0].get_pointer_address(), 0x5010);
        assert_eq!(child_nodes[0].get_pointer_scan_node_type(), PointerScanNodeType::Static);

        let grandchild_nodes = validated_pointer_scan_session.get_expanded_nodes(Some(child_nodes[0].get_node_id()));
        assert_eq!(grandchild_nodes.len(), 1);
        assert_eq!(grandchild_nodes[0].get_pointer_address(), 0x7000);
        assert_eq!(grandchild_nodes[0].get_pointer_scan_node_type(), PointerScanNodeType::Heap);
    }

    #[test]
    fn scan_memory_regions_for_heap_pointer_candidates_by_target_matches_multiple_targets_in_one_pass() {
        let multi_target_validation_memory_map = Arc::new(build_multi_target_validation_memory_map());
        let scan_execution_context = ScanExecutionContext::new(
            None,
            None,
            Some(Arc::new({
                let multi_target_validation_memory_map = multi_target_validation_memory_map.clone();

                move |_opened_process_info, address, values| read_memory_from_map(&multi_target_validation_memory_map, address, values)
            })),
        );
        let required_target_ranges = PointerScanTargetRangeSet::from_target_addresses(&[0x4010, 0x5010], 0x10);
        let range_search_kernel = PointerScanRangeSearchKernel::new(&required_target_ranges, PointerScanPointerSize::Pointer64);
        let rebuilt_pointer_candidates = PointerScanValidator::scan_memory_regions_for_heap_pointer_candidates_by_target(
            &OpenedProcessInfo::new(7, "pointer-test".to_string(), 0, Bitness::Bit64, None),
            &range_search_kernel,
            &[NormalizedRegion::new(0x3000, 0x40)],
            &scan_execution_context,
            false,
            &PointerValidationLevelLogContext {
                level_number: 1,
                level_count: 1,
            },
        );

        assert_eq!(rebuilt_pointer_candidates.len(), 2);
        assert!(
            rebuilt_pointer_candidates
                .iter()
                .any(|rebuilt_pointer_candidate| { rebuilt_pointer_candidate.pointer_address == 0x3000 && rebuilt_pointer_candidate.pointer_value == 0x4000 })
        );
        assert!(
            rebuilt_pointer_candidates
                .iter()
                .any(|rebuilt_pointer_candidate| { rebuilt_pointer_candidate.pointer_address == 0x3008 && rebuilt_pointer_candidate.pointer_value == 0x5000 })
        );
    }

    #[test]
    fn build_heap_memory_regions_excludes_module_ranges() {
        let heap_memory_regions = PointerScanValidator::build_heap_memory_regions(
            &[
                NormalizedRegion::new(0x1000, 0x500),
                NormalizedRegion::new(0x2000, 0x100),
            ],
            &[
                NormalizedModule::new("game.exe", 0x1100, 0x100),
                NormalizedModule::new("engine.dll", 0x1300, 0x80),
            ],
        );

        assert_eq!(heap_memory_regions.len(), 4);
        assert_eq!(heap_memory_regions[0].get_base_address(), 0x1000);
        assert_eq!(heap_memory_regions[0].get_region_size(), 0x100);
        assert_eq!(heap_memory_regions[1].get_base_address(), 0x1200);
        assert_eq!(heap_memory_regions[1].get_region_size(), 0x100);
        assert_eq!(heap_memory_regions[2].get_base_address(), 0x1380);
        assert_eq!(heap_memory_regions[2].get_region_size(), 0x180);
        assert_eq!(heap_memory_regions[3].get_base_address(), 0x2000);
        assert_eq!(heap_memory_regions[3].get_region_size(), 0x100);
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
