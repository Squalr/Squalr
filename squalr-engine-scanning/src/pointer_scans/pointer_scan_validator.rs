use crate::pointer_scans::search_kernels::PointerScanRangeSearchKernel;
use crate::pointer_scans::structures::pointer_scan_region_match::PointerScanRegionMatch;
use crate::pointer_scans::structures::pointer_scan_target_ranges::PointerScanTargetRangeSet;
use crate::pointer_scans::structures::pointer_validation_level_log_context::PointerValidationLevelLogContext;
use crate::pointer_scans::structures::rebuilt_pointer_candidate::RebuiltPointerCandidate;
use crate::pointer_scans::structures::rebuilt_pointer_level::RebuiltPointerLevel;
use crate::pointer_scans::structures::snapshot_region_scan_task::SnapshotRegionScanTask;
use crate::scanners::scan_execution_context::ScanExecutionContext;
use rayon::prelude::*;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;
use squalr_engine_api::structures::pointer_scans::pointer_scan_level::PointerScanLevel;
use squalr_engine_api::structures::pointer_scans::pointer_scan_level_candidates::PointerScanLevelCandidates;
use squalr_engine_api::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
use squalr_engine_api::structures::pointer_scans::pointer_scan_session::PointerScanSession;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use std::collections::HashMap;
use std::time::Instant;

const POINTER_VALIDATION_MIN_SNAPSHOT_TASK_BYTE_SIZE: usize = 1024 * 1024;
const POINTER_VALIDATION_TARGET_TASKS_PER_WORKER: usize = 4;

pub struct PointerScanValidator;

impl PointerScanValidator {
    pub fn validate_scan(
        _process_info: OpenedProcessInfo,
        pointer_scan_session: &PointerScanSession,
        validation_target_address: u64,
        validation_snapshot: &Snapshot,
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

        let mut required_target_ranges =
            PointerScanTargetRangeSet::from_target_addresses(&[validation_target_address], pointer_scan_session.get_offset_radius());
        let mut rebuilt_pointer_levels = Vec::new();
        let level_count = pointer_scan_session.get_pointer_scan_level_candidates().len();
        let (snapshot_region_scan_tasks, total_snapshot_region_count, snapshot_task_byte_size) =
            Self::build_snapshot_region_scan_tasks(validation_snapshot, pointer_scan_session.get_pointer_size());

        for level_index in 0..level_count {
            let level_number = level_index + 1;
            let is_terminal_level = level_number >= level_count;
            if scan_execution_context.should_cancel() {
                break;
            }

            if required_target_ranges.is_empty() {
                if with_logging {
                    log::info!(
                        "Pointer scan validation stopped after level {} because no frontier targets remained.",
                        level_index
                    );
                }

                break;
            }

            let range_search_kernel = PointerScanRangeSearchKernel::new(&required_target_ranges, pointer_scan_session.get_pointer_size());
            let validation_level_log_context = PointerValidationLevelLogContext { level_number, level_count };
            let level_start_time = Instant::now();
            let empty_static_pointer_scan_candidates: &[PointerScanCandidate] = &[];
            let static_pointer_scan_candidates = pointer_scan_session
                .get_pointer_scan_level_candidates()
                .get(level_index)
                .map(PointerScanLevelCandidates::get_static_candidates)
                .map_or(empty_static_pointer_scan_candidates, Vec::as_slice);

            if with_logging {
                log::info!(
                    "Pointer scan validation level {}/{}: checking {} static nodes and scanning {} snapshot regions across {} scan tasks ({} bytes/task) for {} frontier targets merged into {} ranges with {} kernel.",
                    validation_level_log_context.level_number,
                    validation_level_log_context.level_count,
                    static_pointer_scan_candidates.len(),
                    total_snapshot_region_count,
                    snapshot_region_scan_tasks.len(),
                    snapshot_task_byte_size,
                    required_target_ranges.get_source_target_count(),
                    required_target_ranges.get_range_count(),
                    range_search_kernel.get_name(),
                );
            }

            let rebuilt_pointer_level = Self::scan_snapshot_regions_for_pointer_targets(
                &snapshot_region_scan_tasks,
                pointer_scan_session,
                static_pointer_scan_candidates,
                &range_search_kernel,
                modules,
                !is_terminal_level,
            );
            let rebuilt_static_candidates = rebuilt_pointer_level.static_candidates;
            let rebuilt_heap_candidates = rebuilt_pointer_level.heap_candidates;

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

            if !is_terminal_level {
                required_target_ranges = PointerScanTargetRangeSet::from_target_addresses_iter(
                    rebuilt_heap_candidates
                        .iter()
                        .map(|rebuilt_pointer_candidate| rebuilt_pointer_candidate.pointer_address),
                    pointer_scan_session.get_offset_radius(),
                );
            }
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

    fn scan_snapshot_regions_for_pointer_targets(
        snapshot_region_scan_tasks: &[SnapshotRegionScanTask<'_>],
        original_pointer_scan_session: &PointerScanSession,
        static_pointer_scan_candidates: &[PointerScanCandidate],
        range_search_kernel: &PointerScanRangeSearchKernel<'_>,
        modules: &[NormalizedModule],
        retain_heap_candidates: bool,
    ) -> RebuiltPointerLevel {
        let sorted_original_static_membership = Self::build_sorted_original_static_membership(original_pointer_scan_session, static_pointer_scan_candidates);

        snapshot_region_scan_tasks
            .par_iter()
            .fold(
                || RebuiltPointerLevel {
                    static_candidates: Vec::new(),
                    heap_candidates: Vec::new(),
                },
                |mut worker_rebuilt_pointer_level, snapshot_region_scan_task| {
                    Self::scan_snapshot_region_for_pointer_targets(
                        snapshot_region_scan_task,
                        range_search_kernel,
                        modules,
                        &sorted_original_static_membership,
                        retain_heap_candidates,
                        &mut worker_rebuilt_pointer_level,
                    );

                    worker_rebuilt_pointer_level
                },
            )
            .reduce(
                || RebuiltPointerLevel {
                    static_candidates: Vec::new(),
                    heap_candidates: Vec::new(),
                },
                |mut left_rebuilt_pointer_level, mut right_rebuilt_pointer_level| {
                    left_rebuilt_pointer_level
                        .static_candidates
                        .append(&mut right_rebuilt_pointer_level.static_candidates);
                    left_rebuilt_pointer_level
                        .heap_candidates
                        .append(&mut right_rebuilt_pointer_level.heap_candidates);

                    left_rebuilt_pointer_level
                },
            )
    }

    fn scan_snapshot_region_for_pointer_targets(
        snapshot_region_scan_task: &SnapshotRegionScanTask<'_>,
        range_search_kernel: &PointerScanRangeSearchKernel<'_>,
        modules: &[NormalizedModule],
        sorted_original_static_membership: &[(&str, u64)],
        retain_heap_candidates: bool,
        rebuilt_pointer_level: &mut RebuiltPointerLevel,
    ) {
        range_search_kernel.scan_region_with_visitor(
            snapshot_region_scan_task.base_address,
            snapshot_region_scan_task.current_values,
            |pointer_match: PointerScanRegionMatch| {
                let pointer_address = pointer_match.get_pointer_address();
                let pointer_value = pointer_match.get_pointer_value();

                if let Some((module_name, module_offset)) = Self::classify_static_pointer_address(pointer_address, modules) {
                    if Self::contains_original_static_membership(sorted_original_static_membership, module_name, module_offset) {
                        rebuilt_pointer_level
                            .static_candidates
                            .push(RebuiltPointerCandidate {
                                pointer_scan_node_type: PointerScanNodeType::Static,
                                pointer_address,
                                pointer_value,
                                module_name: module_name.to_string(),
                                module_offset,
                            });
                    }
                } else if retain_heap_candidates {
                    rebuilt_pointer_level
                        .heap_candidates
                        .push(RebuiltPointerCandidate {
                            pointer_scan_node_type: PointerScanNodeType::Heap,
                            pointer_address,
                            pointer_value,
                            module_name: String::new(),
                            module_offset: 0,
                        });
                }
            },
        );
    }

    fn build_snapshot_region_scan_tasks<'a>(
        validation_snapshot: &'a Snapshot,
        pointer_size: squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize,
    ) -> (Vec<SnapshotRegionScanTask<'a>>, usize, usize) {
        let pointer_size_in_bytes = pointer_size.get_size_in_bytes() as usize;
        let total_snapshot_region_count = validation_snapshot.get_snapshot_regions().len();
        let total_snapshot_byte_count = validation_snapshot
            .get_snapshot_regions()
            .iter()
            .map(|snapshot_region| snapshot_region.get_current_values().len())
            .sum::<usize>();
        let task_byte_size = Self::calculate_snapshot_task_byte_size(total_snapshot_byte_count, total_snapshot_region_count, pointer_size_in_bytes);
        let estimated_task_count = validation_snapshot
            .get_snapshot_regions()
            .iter()
            .map(|snapshot_region| {
                let current_value_byte_count = snapshot_region.get_current_values().len();

                if current_value_byte_count == 0 {
                    0
                } else {
                    current_value_byte_count
                        .saturating_add(task_byte_size.saturating_sub(1))
                        .checked_div(task_byte_size)
                        .unwrap_or(0)
                }
            })
            .sum();
        let mut snapshot_region_scan_tasks = Vec::with_capacity(estimated_task_count);

        for snapshot_region in validation_snapshot.get_snapshot_regions() {
            let current_values = snapshot_region.get_current_values().as_slice();
            let mut task_start_offset = 0_usize;

            while task_start_offset < current_values.len() {
                let remaining_byte_count = current_values.len().saturating_sub(task_start_offset);
                let task_byte_count = remaining_byte_count.min(task_byte_size.max(pointer_size_in_bytes));
                let task_end_offset = task_start_offset.saturating_add(task_byte_count);

                snapshot_region_scan_tasks.push(SnapshotRegionScanTask {
                    base_address: snapshot_region
                        .get_base_address()
                        .saturating_add(task_start_offset as u64),
                    current_values: &current_values[task_start_offset..task_end_offset],
                });

                task_start_offset = task_end_offset;
            }
        }

        (snapshot_region_scan_tasks, total_snapshot_region_count, task_byte_size)
    }

    fn calculate_snapshot_task_byte_size(
        total_snapshot_byte_count: usize,
        total_snapshot_region_count: usize,
        pointer_size_in_bytes: usize,
    ) -> usize {
        let pointer_alignment = pointer_size_in_bytes.max(1);
        let minimum_task_byte_size = POINTER_VALIDATION_MIN_SNAPSHOT_TASK_BYTE_SIZE.max(pointer_alignment);
        let target_task_count = total_snapshot_region_count
            .max(
                rayon::current_num_threads()
                    .max(1)
                    .saturating_mul(POINTER_VALIDATION_TARGET_TASKS_PER_WORKER),
            )
            .max(1);
        let target_task_byte_size = total_snapshot_byte_count
            .saturating_add(target_task_count.saturating_sub(1))
            .checked_div(target_task_count)
            .unwrap_or(0)
            .max(minimum_task_byte_size);

        target_task_byte_size
            .max(pointer_alignment)
            .saturating_sub(target_task_byte_size.max(pointer_alignment) % pointer_alignment)
    }

    fn classify_static_pointer_address<'a>(
        pointer_address: u64,
        modules: &'a [NormalizedModule],
    ) -> Option<(&'a str, u64)> {
        modules.iter().find_map(|module| {
            module
                .contains_address(pointer_address)
                .then_some((module.get_module_name(), pointer_address.saturating_sub(module.get_base_address())))
        })
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
        let mut module_names = Vec::new();
        let mut module_indices_by_name = HashMap::new();
        for (level_index, rebuilt_pointer_level) in rebuilt_pointer_levels.iter().enumerate() {
            let discovery_depth = level_index as u64 + 1;
            let mut static_candidates = Vec::with_capacity(rebuilt_pointer_level.static_candidates.len());

            for rebuilt_pointer_candidate in &rebuilt_pointer_level.static_candidates {
                let module_index = if let Some(module_index) = module_indices_by_name.get(rebuilt_pointer_candidate.module_name.as_str()) {
                    *module_index
                } else {
                    let next_module_index = module_names.len();
                    module_names.push(rebuilt_pointer_candidate.module_name.clone());
                    module_indices_by_name.insert(rebuilt_pointer_candidate.module_name.clone(), next_module_index);
                    next_module_index
                };
                static_candidates.push(PointerScanCandidate::new(
                    next_candidate_id,
                    discovery_depth,
                    PointerScanNodeType::Static,
                    rebuilt_pointer_candidate.pointer_address,
                    rebuilt_pointer_candidate.pointer_value,
                    module_index,
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
                    0,
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
            pointer_scan_level_candidates.push(level_candidates);
        }

        let root_node_count = total_static_node_count;

        PointerScanSession::new(
            original_pointer_scan_session.get_session_id(),
            validation_target_address,
            original_pointer_scan_session.get_pointer_size(),
            original_pointer_scan_session.get_max_depth(),
            original_pointer_scan_session.get_offset_radius(),
            module_names,
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
            Vec::new(),
            0,
            0,
            0,
        )
    }

    fn build_sorted_original_static_membership<'a>(
        original_pointer_scan_session: &'a PointerScanSession,
        static_pointer_scan_candidates: &'a [PointerScanCandidate],
    ) -> Vec<(&'a str, u64)> {
        let mut sorted_original_static_membership = static_pointer_scan_candidates
            .iter()
            .filter_map(|static_pointer_scan_candidate| {
                original_pointer_scan_session
                    .get_module_name(static_pointer_scan_candidate.get_module_index())
                    .map(|module_name| (module_name, static_pointer_scan_candidate.get_module_offset()))
            })
            .collect::<Vec<_>>();

        sorted_original_static_membership.sort_unstable();
        sorted_original_static_membership.dedup();

        sorted_original_static_membership
    }

    fn find_original_static_membership_for_module<'a>(
        sorted_original_static_membership: &'a [(&'a str, u64)],
        module_name: &str,
    ) -> &'a [(&'a str, u64)] {
        let membership_start_index =
            sorted_original_static_membership.partition_point(|(candidate_module_name, _candidate_module_offset)| *candidate_module_name < module_name);
        let membership_end_index =
            sorted_original_static_membership.partition_point(|(candidate_module_name, _candidate_module_offset)| *candidate_module_name <= module_name);

        &sorted_original_static_membership[membership_start_index..membership_end_index]
    }

    fn contains_original_static_membership(
        sorted_original_static_membership: &[(&str, u64)],
        module_name: &str,
        module_offset: u64,
    ) -> bool {
        let original_module_membership = Self::find_original_static_membership_for_module(sorted_original_static_membership, module_name);

        original_module_membership
            .binary_search_by_key(&module_offset, |(_module_name, candidate_module_offset)| *candidate_module_offset)
            .is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::PointerScanValidator;
    use crate::pointer_scans::pointer_scan_executor_task::PointerScanExecutor;
    use crate::pointer_scans::search_kernels::PointerScanRangeSearchKernel;
    use crate::pointer_scans::structures::pointer_scan_target_ranges::PointerScanTargetRangeSet;
    use crate::scanners::scan_execution_context::ScanExecutionContext;
    use squalr_engine_api::structures::memory::bitness::Bitness;
    use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
    use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;
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
            &build_snapshot_from_memory_map(&build_validation_memory_regions(), &validation_memory_map),
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
            &build_snapshot_from_memory_map(&build_rebased_validation_memory_regions(), &rebased_validation_memory_map),
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
    fn validate_scan_intersects_live_static_matches_with_original_static_candidates() {
        let original_pointer_scan_session = build_original_pointer_scan_session();
        let extra_static_validation_memory_map = Arc::new(build_extra_static_validation_memory_map());
        let scan_execution_context = ScanExecutionContext::new(
            None,
            None,
            Some(Arc::new({
                let extra_static_validation_memory_map = extra_static_validation_memory_map.clone();

                move |_opened_process_info, address, values| read_memory_from_map(&extra_static_validation_memory_map, address, values)
            })),
        );
        let mut validated_pointer_scan_session = PointerScanValidator::validate_scan(
            OpenedProcessInfo::new(7, "pointer-test".to_string(), 0, Bitness::Bit64, None),
            &original_pointer_scan_session,
            0x3010,
            &build_snapshot_from_memory_map(&build_extra_static_validation_memory_regions(), &extra_static_validation_memory_map),
            &[NormalizedModule::new("game.exe", 0x1000, 0x100)],
            &scan_execution_context,
            false,
        );

        let root_nodes = validated_pointer_scan_session.get_expanded_nodes(None);

        assert!(
            root_nodes
                .iter()
                .any(|pointer_scan_node| pointer_scan_node.get_pointer_address() == 0x1010)
        );
        assert!(
            root_nodes
                .iter()
                .any(|pointer_scan_node| pointer_scan_node.get_pointer_address() == 0x1030)
        );
        assert!(
            root_nodes
                .iter()
                .all(|pointer_scan_node| pointer_scan_node.get_pointer_address() != 0x1020)
        );
    }

    #[test]
    fn scan_snapshot_regions_for_pointer_targets_matches_multiple_targets_in_one_pass() {
        let multi_target_validation_memory_map = Arc::new(build_multi_target_validation_memory_map());
        let required_target_ranges = PointerScanTargetRangeSet::from_target_addresses(&[0x4010, 0x5010], 0x10);
        let range_search_kernel = PointerScanRangeSearchKernel::new(&required_target_ranges, PointerScanPointerSize::Pointer64);
        let validation_snapshot = build_snapshot_from_memory_map(&[NormalizedRegion::new(0x3000, 0x40)], &multi_target_validation_memory_map);
        let static_pointer_scan_candidates: &[PointerScanCandidate] = &[];
        let rebuilt_pointer_level = PointerScanValidator::scan_snapshot_regions_for_pointer_targets(
            &PointerScanValidator::build_snapshot_region_scan_tasks(&validation_snapshot, PointerScanPointerSize::Pointer64).0,
            &build_original_pointer_scan_session(),
            static_pointer_scan_candidates,
            &range_search_kernel,
            &[],
            true,
        );

        assert_eq!(rebuilt_pointer_level.static_candidates.len(), 0);
        assert_eq!(rebuilt_pointer_level.heap_candidates.len(), 2);
        assert!(
            rebuilt_pointer_level
                .heap_candidates
                .iter()
                .any(|rebuilt_pointer_candidate| { rebuilt_pointer_candidate.pointer_address == 0x3000 && rebuilt_pointer_candidate.pointer_value == 0x4000 })
        );
        assert!(
            rebuilt_pointer_level
                .heap_candidates
                .iter()
                .any(|rebuilt_pointer_candidate| { rebuilt_pointer_candidate.pointer_address == 0x3008 && rebuilt_pointer_candidate.pointer_value == 0x5000 })
        );
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

    fn build_snapshot_from_memory_map(
        memory_regions: &[NormalizedRegion],
        memory_map: &HashMap<u64, u8>,
    ) -> Snapshot {
        let mut snapshot = Snapshot::new();
        let mut snapshot_regions = Vec::with_capacity(memory_regions.len());

        for memory_region in memory_regions {
            let mut snapshot_region = SnapshotRegion::new(memory_region.clone(), Vec::new());
            snapshot_region.current_values = (0..memory_region.get_region_size())
                .map(|byte_offset| {
                    *memory_map
                        .get(&memory_region.get_base_address().saturating_add(byte_offset))
                        .unwrap_or(&0)
                })
                .collect();
            snapshot_regions.push(snapshot_region);
        }

        snapshot.set_snapshot_regions(snapshot_regions);

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

    fn build_extra_static_validation_memory_map() -> HashMap<u64, u8> {
        let mut memory_map = HashMap::new();

        write_pointer_bytes(&mut memory_map, 0x1010, 0x1FF0_u64);
        write_pointer_bytes(&mut memory_map, 0x1020, 0x3000_u64);
        write_pointer_bytes(&mut memory_map, 0x1030, 0x3020_u64);
        write_pointer_bytes(&mut memory_map, 0x2000, 0x3000_u64);

        memory_map
    }

    fn build_extra_static_validation_memory_regions() -> Vec<NormalizedRegion> {
        vec![
            NormalizedRegion::new(0x1000, 0x40),
            NormalizedRegion::new(0x2000, 0x40),
        ]
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
