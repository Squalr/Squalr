use crate::pointer_scans::search_kernels::PointerScanRangeSearchKernel;
use crate::pointer_scans::search_kernels::pointer_scan_pointer_value_reader::read_pointer_value_unchecked;
use crate::pointer_scans::structures::pointer_scan_target_ranges::PointerScanTargetRangeSet;
use crate::pointer_scans::structures::pointer_validation_level_log_context::PointerValidationLevelLogContext;
use crate::pointer_scans::structures::snapshot_region_scan_task::SnapshotRegionScanTask;
use crate::pointer_scans::structures::snapshot_region_scan_task_kind::SnapshotRegionScanTaskKind;
use crate::pointer_scans::structures::validated_pointer_candidate::ValidatedPointerCandidate;
use crate::pointer_scans::structures::validated_pointer_level::ValidatedPointerLevel;
use crate::scanners::scan_execution_context::ScanExecutionContext;
use rayon::prelude::*;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;
use squalr_engine_api::structures::pointer_scans::pointer_scan_level::PointerScanLevel;
use squalr_engine_api::structures::pointer_scans::pointer_scan_level_candidates::PointerScanLevelCandidates;
use squalr_engine_api::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::pointer_scans::pointer_scan_session::PointerScanSession;
use squalr_engine_api::structures::pointer_scans::pointer_scan_target_descriptor::PointerScanTargetDescriptor;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use std::time::Instant;

const POINTER_VALIDATION_ROOT_CHUNK_SIZE: usize = 64;
const POINTER_VALIDATION_MIN_SNAPSHOT_TASK_BYTE_SIZE: usize = 1024 * 1024;
const POINTER_VALIDATION_TARGET_TASKS_PER_WORKER: usize = 4;

struct PointerValidationContext<'a> {
    validation_snapshot_regions: &'a [SnapshotRegion],
    current_modules_by_name: Vec<(&'a str, u64)>,
    validation_target_addresses: &'a [u64],
    offset_radius: u64,
    pointer_size: PointerScanPointerSize,
    scan_execution_context: &'a ScanExecutionContext,
}

impl<'a> PointerValidationContext<'a> {
    fn new(
        original_pointer_scan_session: &'a PointerScanSession,
        validation_target_addresses: &'a [u64],
        validation_snapshot: &'a Snapshot,
        modules: &'a [NormalizedModule],
        scan_execution_context: &'a ScanExecutionContext,
    ) -> Self {
        let mut current_modules_by_name = modules
            .iter()
            .map(|module| (module.get_module_name(), module.get_base_address()))
            .collect::<Vec<_>>();
        current_modules_by_name.sort_unstable_by(|left_module, right_module| left_module.0.cmp(right_module.0));

        Self {
            validation_snapshot_regions: validation_snapshot.get_snapshot_regions().as_slice(),
            current_modules_by_name,
            validation_target_addresses,
            offset_radius: original_pointer_scan_session.get_offset_radius(),
            pointer_size: original_pointer_scan_session.get_pointer_size(),
            scan_execution_context,
        }
    }
}

pub struct PointerScanValidator;

impl PointerScanValidator {
    pub fn validate_scan(
        _process_info: OpenedProcessInfo,
        pointer_scan_session: &PointerScanSession,
        validation_target_descriptor: PointerScanTargetDescriptor,
        validation_target_addresses: Vec<u64>,
        validation_snapshot: &Snapshot,
        modules: &[NormalizedModule],
        scan_execution_context: &ScanExecutionContext,
        with_logging: bool,
    ) -> PointerScanSession {
        let total_start_time = Instant::now();

        if with_logging {
            log::info!(
                "Validating pointer scan session {} against target {}.",
                pointer_scan_session.get_session_id(),
                validation_target_descriptor,
            );
        }

        if pointer_scan_session.get_pointer_scan_levels().is_empty() {
            return Self::create_empty_session(pointer_scan_session, validation_target_descriptor, validation_target_addresses);
        }

        let validation_context = PointerValidationContext::new(
            pointer_scan_session,
            validation_target_addresses.as_slice(),
            validation_snapshot,
            modules,
            scan_execution_context,
        );
        let validation_heap_scan_tasks =
            Self::build_validation_heap_scan_tasks(validation_context.validation_snapshot_regions, modules, validation_context.pointer_size);
        let mut validated_pointer_levels = Self::validate_pointer_levels(pointer_scan_session, &validation_context, &validation_heap_scan_tasks, with_logging);

        Self::truncate_empty_trailing_levels(&mut validated_pointer_levels);

        let validated_pointer_scan_session = if validated_pointer_levels.is_empty() {
            Self::create_empty_session(pointer_scan_session, validation_target_descriptor, validation_target_addresses)
        } else {
            Self::build_pointer_scan_session(
                pointer_scan_session,
                validation_target_descriptor,
                validation_target_addresses,
                validated_pointer_levels,
            )
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

    fn validate_pointer_levels(
        pointer_scan_session: &PointerScanSession,
        validation_context: &PointerValidationContext<'_>,
        validation_heap_scan_tasks: &[SnapshotRegionScanTask<'_>],
        with_logging: bool,
    ) -> Vec<ValidatedPointerLevel> {
        let level_count = pointer_scan_session.get_pointer_scan_level_candidates().len();
        let mut validated_pointer_levels = Vec::with_capacity(level_count);

        // Validation mirrors the old rebase flow: frontier -> rebuild heaps -> prune stored statics -> next frontier.
        let mut frontier_target_ranges =
            PointerScanTargetRangeSet::from_target_addresses(validation_context.validation_target_addresses, validation_context.offset_radius);

        for (level_index, pointer_scan_level_candidates) in pointer_scan_session
            .get_pointer_scan_level_candidates()
            .iter()
            .enumerate()
        {
            if validation_context.scan_execution_context.should_cancel() || frontier_target_ranges.is_empty() {
                break;
            }

            let validation_level_log_context = PointerValidationLevelLogContext {
                level_number: level_index + 1,
                level_count,
            };
            let level_start_time = Instant::now();
            let retain_heap_candidates = level_index.saturating_add(1) < level_count;

            if with_logging {
                log::info!(
                    "Pointer scan validation level {}/{}: rebuilding heaps from {} frontier targets and pruning {} stored static roots.",
                    validation_level_log_context.level_number,
                    validation_level_log_context.level_count,
                    frontier_target_ranges.get_source_target_count(),
                    pointer_scan_level_candidates.get_static_node_count(),
                );
            }

            let validated_heap_candidates = if retain_heap_candidates {
                Self::collect_validated_heap_candidates(validation_heap_scan_tasks, &frontier_target_ranges, validation_context.pointer_size)
            } else {
                Vec::new()
            };
            let validated_static_candidates = Self::collect_validated_static_candidates(
                pointer_scan_session,
                pointer_scan_level_candidates.get_static_candidates(),
                validation_context,
                &frontier_target_ranges,
            );

            if with_logging {
                log::info!(
                    "Pointer scan validation level {}/{} complete in {:?}: retained {} static roots and rebuilt {} heap nodes.",
                    validation_level_log_context.level_number,
                    validation_level_log_context.level_count,
                    level_start_time.elapsed(),
                    validated_static_candidates.len(),
                    validated_heap_candidates.len(),
                );
            }

            validated_pointer_levels.push(ValidatedPointerLevel {
                static_candidates: validated_static_candidates,
                heap_candidates: validated_heap_candidates,
            });

            if let Some(validated_pointer_level) = validated_pointer_levels.last() {
                frontier_target_ranges = PointerScanTargetRangeSet::from_sorted_target_addresses_iter(
                    validated_pointer_level
                        .heap_candidates
                        .iter()
                        .map(|validated_pointer_candidate| validated_pointer_candidate.pointer_address),
                    validation_context.offset_radius,
                );
            }
        }

        validated_pointer_levels
    }

    fn collect_validated_heap_candidates(
        validation_heap_scan_tasks: &[SnapshotRegionScanTask<'_>],
        frontier_target_ranges: &PointerScanTargetRangeSet,
        pointer_size: PointerScanPointerSize,
    ) -> Vec<ValidatedPointerCandidate> {
        if frontier_target_ranges.is_empty() {
            return Vec::new();
        }

        let range_search_kernel = PointerScanRangeSearchKernel::new(frontier_target_ranges, pointer_size);
        let validated_heap_candidates_by_task = validation_heap_scan_tasks
            .par_iter()
            .map(|validation_heap_scan_task| {
                let mut validated_heap_candidates = Vec::new();

                range_search_kernel.scan_region_with_visitor(
                    validation_heap_scan_task.scan_base_address,
                    validation_heap_scan_task.current_values,
                    |pointer_match| {
                        if pointer_match.get_pointer_address() >= validation_heap_scan_task.scan_end_address {
                            return;
                        }

                        validated_heap_candidates.push(ValidatedPointerCandidate {
                            pointer_address: pointer_match.get_pointer_address(),
                            pointer_value: pointer_match.get_pointer_value(),
                            module_index: 0,
                            module_offset: 0,
                        });
                    },
                );

                validated_heap_candidates
            })
            .collect::<Vec<_>>();
        let total_heap_candidate_count = validated_heap_candidates_by_task.iter().map(Vec::len).sum();
        let mut validated_heap_candidates = Vec::with_capacity(total_heap_candidate_count);

        for mut worker_validated_heap_candidates in validated_heap_candidates_by_task {
            validated_heap_candidates.append(&mut worker_validated_heap_candidates);
        }

        validated_heap_candidates
    }

    fn collect_validated_static_candidates(
        pointer_scan_session: &PointerScanSession,
        static_pointer_scan_candidates: &[PointerScanCandidate],
        validation_context: &PointerValidationContext<'_>,
        frontier_target_ranges: &PointerScanTargetRangeSet,
    ) -> Vec<ValidatedPointerCandidate> {
        let validated_static_candidates_by_chunk = static_pointer_scan_candidates
            .par_chunks(POINTER_VALIDATION_ROOT_CHUNK_SIZE)
            .map(|static_pointer_scan_candidate_chunk| {
                let mut validated_static_candidates = Vec::with_capacity(static_pointer_scan_candidate_chunk.len());

                for static_pointer_scan_candidate in static_pointer_scan_candidate_chunk {
                    if validation_context.scan_execution_context.should_cancel() {
                        break;
                    }

                    let Some(current_pointer_address) =
                        Self::resolve_current_static_pointer_address(pointer_scan_session, static_pointer_scan_candidate, validation_context)
                    else {
                        continue;
                    };
                    let Some(current_pointer_value) = Self::read_pointer_value_at_address(
                        validation_context.validation_snapshot_regions,
                        current_pointer_address,
                        validation_context.pointer_size,
                    ) else {
                        continue;
                    };

                    if !Self::pointer_value_reaches_frontier(frontier_target_ranges, current_pointer_value) {
                        continue;
                    }

                    validated_static_candidates.push(ValidatedPointerCandidate {
                        pointer_address: current_pointer_address,
                        pointer_value: current_pointer_value,
                        module_index: static_pointer_scan_candidate.get_module_index(),
                        module_offset: static_pointer_scan_candidate.get_module_offset(),
                    });
                }

                validated_static_candidates
            })
            .collect::<Vec<_>>();
        let total_static_candidate_count = validated_static_candidates_by_chunk.iter().map(Vec::len).sum();
        let mut validated_static_candidates = Vec::with_capacity(total_static_candidate_count);

        for mut worker_validated_static_candidates in validated_static_candidates_by_chunk {
            validated_static_candidates.append(&mut worker_validated_static_candidates);
        }

        validated_static_candidates
    }

    fn pointer_value_reaches_frontier(
        frontier_target_ranges: &PointerScanTargetRangeSet,
        pointer_value: u64,
    ) -> bool {
        if frontier_target_ranges.get_range_count() <= 8 {
            frontier_target_ranges.contains_value_linear(pointer_value)
        } else {
            frontier_target_ranges.contains_value_binary(pointer_value)
        }
    }

    fn resolve_current_static_pointer_address(
        pointer_scan_session: &PointerScanSession,
        static_pointer_scan_candidate: &PointerScanCandidate,
        validation_context: &PointerValidationContext<'_>,
    ) -> Option<u64> {
        let module_name = pointer_scan_session.get_module_name(static_pointer_scan_candidate.get_module_index())?;
        let current_module_base_address = Self::find_current_module_base_address(&validation_context.current_modules_by_name, module_name)?;

        Some(current_module_base_address.saturating_add(static_pointer_scan_candidate.get_module_offset()))
    }

    fn find_current_module_base_address(
        current_modules_by_name: &[(&str, u64)],
        module_name: &str,
    ) -> Option<u64> {
        current_modules_by_name
            .binary_search_by(|(candidate_module_name, _candidate_module_base_address)| candidate_module_name.cmp(&module_name))
            .ok()
            .and_then(|module_index| current_modules_by_name.get(module_index))
            .map(|(_module_name, module_base_address)| *module_base_address)
    }

    fn build_validation_heap_scan_tasks<'a>(
        validation_snapshot_regions: &'a [SnapshotRegion],
        modules: &[NormalizedModule],
        pointer_size: PointerScanPointerSize,
    ) -> Vec<SnapshotRegionScanTask<'a>> {
        // Validation only rebuilds heap levels from live heap memory. Static bases are reread separately from the stored module offsets.
        let mut sorted_modules = modules.iter().collect::<Vec<_>>();
        sorted_modules.sort_unstable_by_key(|module| module.get_base_address());
        let pointer_size_in_bytes = pointer_size.get_size_in_bytes() as usize;
        let total_snapshot_byte_count = validation_snapshot_regions
            .iter()
            .map(|snapshot_region| snapshot_region.get_current_values().len())
            .sum::<usize>();
        let task_byte_size =
            Self::calculate_validation_snapshot_task_byte_size(total_snapshot_byte_count, validation_snapshot_regions.len(), pointer_size_in_bytes);
        let mut validation_heap_scan_tasks = Vec::new();

        for validation_snapshot_region in validation_snapshot_regions {
            if validation_snapshot_region.get_current_values().is_empty() {
                continue;
            }

            let mut uncovered_range_base_address = validation_snapshot_region.get_base_address();
            let validation_snapshot_region_end_address = validation_snapshot_region.get_end_address();

            for module in &sorted_modules {
                let module_base_address = module.get_base_address();
                let module_end_address = module
                    .get_base_address()
                    .saturating_add(module.get_region_size());

                if module_end_address <= uncovered_range_base_address {
                    continue;
                }

                if module_base_address >= validation_snapshot_region_end_address {
                    break;
                }

                if uncovered_range_base_address < module_base_address {
                    Self::append_validation_heap_scan_tasks_for_range(
                        validation_snapshot_region,
                        uncovered_range_base_address,
                        module_base_address.min(validation_snapshot_region_end_address),
                        task_byte_size,
                        pointer_size_in_bytes,
                        &mut validation_heap_scan_tasks,
                    );
                }

                uncovered_range_base_address = uncovered_range_base_address.max(module_end_address);

                if uncovered_range_base_address >= validation_snapshot_region_end_address {
                    break;
                }
            }

            if uncovered_range_base_address < validation_snapshot_region_end_address {
                Self::append_validation_heap_scan_tasks_for_range(
                    validation_snapshot_region,
                    uncovered_range_base_address,
                    validation_snapshot_region_end_address,
                    task_byte_size,
                    pointer_size_in_bytes,
                    &mut validation_heap_scan_tasks,
                );
            }
        }

        validation_heap_scan_tasks
    }

    fn append_validation_heap_scan_tasks_for_range<'a>(
        validation_snapshot_region: &'a SnapshotRegion,
        range_base_address: u64,
        range_end_address: u64,
        task_byte_size: usize,
        pointer_size_in_bytes: usize,
        validation_heap_scan_tasks: &mut Vec<SnapshotRegionScanTask<'a>>,
    ) {
        if range_end_address <= range_base_address {
            return;
        }

        let range_start_offset = range_base_address.saturating_sub(validation_snapshot_region.get_base_address()) as usize;
        let range_end_offset = range_end_address.saturating_sub(validation_snapshot_region.get_base_address()) as usize;
        let current_values = validation_snapshot_region.get_current_values().as_slice();
        let mut task_start_offset = range_start_offset;

        while task_start_offset < range_end_offset {
            let remaining_byte_count = range_end_offset.saturating_sub(task_start_offset);
            let task_byte_count = remaining_byte_count.min(task_byte_size);
            let task_end_offset = task_start_offset.saturating_add(task_byte_count);
            let task_read_end_offset = task_end_offset
                .saturating_add(pointer_size_in_bytes.saturating_sub(1))
                .min(current_values.len());

            validation_heap_scan_tasks.push(SnapshotRegionScanTask {
                scan_base_address: validation_snapshot_region
                    .get_base_address()
                    .saturating_add(task_start_offset as u64),
                scan_end_address: validation_snapshot_region
                    .get_base_address()
                    .saturating_add(task_end_offset as u64),
                current_values: &current_values[task_start_offset..task_read_end_offset],
                task_kind: SnapshotRegionScanTaskKind::Heap,
            });

            task_start_offset = task_end_offset;
        }
    }

    fn calculate_validation_snapshot_task_byte_size(
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

    fn read_pointer_value_at_address(
        snapshot_regions: &[SnapshotRegion],
        pointer_address: u64,
        pointer_size: PointerScanPointerSize,
    ) -> Option<u64> {
        let mut snapshot_region_index_hint =
            Self::find_snapshot_region_index_for_pointer_address(snapshot_regions, pointer_address, pointer_size.get_size_in_bytes() as usize)?;

        Self::read_pointer_value_from_snapshot_region(snapshot_regions, pointer_address, pointer_size, &mut snapshot_region_index_hint)
    }

    fn read_pointer_value_from_snapshot_region(
        snapshot_regions: &[SnapshotRegion],
        pointer_address: u64,
        pointer_size: PointerScanPointerSize,
        snapshot_region_index_hint: &mut usize,
    ) -> Option<u64> {
        let pointer_size_in_bytes = pointer_size.get_size_in_bytes() as usize;
        let pointer_end_address = pointer_address.checked_add(pointer_size_in_bytes as u64)?;

        while let Some(snapshot_region) = snapshot_regions.get(*snapshot_region_index_hint) {
            if snapshot_region.get_current_values().is_empty() || pointer_address >= snapshot_region.get_end_address() {
                *snapshot_region_index_hint = (*snapshot_region_index_hint).saturating_add(1);
                continue;
            }

            if pointer_address < snapshot_region.get_base_address() || pointer_end_address > snapshot_region.get_end_address() {
                return None;
            }

            let byte_offset = pointer_address.saturating_sub(snapshot_region.get_base_address()) as usize;
            let pointer_bytes = snapshot_region
                .get_current_values()
                .get(byte_offset..byte_offset.saturating_add(pointer_size_in_bytes))?;

            // Snapshot bytes are already in host memory, so decode them directly without slice copies.
            return Some(unsafe { read_pointer_value_unchecked(pointer_bytes.as_ptr(), pointer_size) });
        }

        None
    }

    fn find_snapshot_region_index_for_pointer_address(
        snapshot_regions: &[SnapshotRegion],
        pointer_address: u64,
        pointer_size_in_bytes: usize,
    ) -> Option<usize> {
        let pointer_end_address = pointer_address.checked_add(pointer_size_in_bytes as u64)?;
        let mut lower_snapshot_region_index = 0_usize;
        let mut upper_snapshot_region_index = snapshot_regions.len();

        while lower_snapshot_region_index < upper_snapshot_region_index {
            let middle_snapshot_region_index =
                lower_snapshot_region_index.saturating_add(upper_snapshot_region_index.saturating_sub(lower_snapshot_region_index) / 2);
            let snapshot_region = snapshot_regions.get(middle_snapshot_region_index)?;

            if snapshot_region.get_end_address() <= pointer_address {
                lower_snapshot_region_index = middle_snapshot_region_index.saturating_add(1);
            } else {
                upper_snapshot_region_index = middle_snapshot_region_index;
            }
        }

        let snapshot_region = snapshot_regions.get(lower_snapshot_region_index)?;
        (pointer_address >= snapshot_region.get_base_address() && pointer_end_address <= snapshot_region.get_end_address())
            .then_some(lower_snapshot_region_index)
    }

    fn truncate_empty_trailing_levels(validated_pointer_levels: &mut Vec<ValidatedPointerLevel>) {
        while validated_pointer_levels
            .last()
            .is_some_and(|validated_pointer_level| validated_pointer_level.static_candidates.is_empty() && validated_pointer_level.heap_candidates.is_empty())
        {
            validated_pointer_levels.pop();
        }
    }

    fn build_pointer_scan_session(
        original_pointer_scan_session: &PointerScanSession,
        validation_target_descriptor: PointerScanTargetDescriptor,
        validation_target_addresses: Vec<u64>,
        validated_pointer_levels: Vec<ValidatedPointerLevel>,
    ) -> PointerScanSession {
        let mut pointer_scan_levels = Vec::new();
        let mut pointer_scan_level_candidates = Vec::new();
        let mut next_candidate_id = 1_u64;
        let mut total_static_node_count = 0_u64;
        let mut total_heap_node_count = 0_u64;

        for (level_index, validated_pointer_level) in validated_pointer_levels.iter().enumerate() {
            let discovery_depth = level_index as u64 + 1;
            let mut static_candidates = Vec::with_capacity(validated_pointer_level.static_candidates.len());

            for validated_pointer_candidate in &validated_pointer_level.static_candidates {
                static_candidates.push(PointerScanCandidate::new(
                    next_candidate_id,
                    discovery_depth,
                    PointerScanNodeType::Static,
                    validated_pointer_candidate.pointer_address,
                    validated_pointer_candidate.pointer_value,
                    validated_pointer_candidate.module_index,
                    validated_pointer_candidate.module_offset,
                ));
                next_candidate_id = next_candidate_id.saturating_add(1);
            }

            let mut heap_candidates = Vec::with_capacity(validated_pointer_level.heap_candidates.len());

            for validated_pointer_candidate in &validated_pointer_level.heap_candidates {
                heap_candidates.push(PointerScanCandidate::new(
                    next_candidate_id,
                    discovery_depth,
                    PointerScanNodeType::Heap,
                    validated_pointer_candidate.pointer_address,
                    validated_pointer_candidate.pointer_value,
                    0,
                    0,
                ));
                next_candidate_id = next_candidate_id.saturating_add(1);
            }

            let level_candidates = PointerScanLevelCandidates::new_presorted(discovery_depth, static_candidates, heap_candidates);

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

        PointerScanSession::new(
            original_pointer_scan_session.get_session_id(),
            validation_target_descriptor,
            validation_target_addresses,
            original_pointer_scan_session.get_pointer_size(),
            original_pointer_scan_session.get_max_depth(),
            original_pointer_scan_session.get_offset_radius(),
            original_pointer_scan_session.get_module_names().clone(),
            pointer_scan_levels,
            pointer_scan_level_candidates,
            total_static_node_count,
            total_heap_node_count,
        )
    }

    fn create_empty_session(
        original_pointer_scan_session: &PointerScanSession,
        validation_target_descriptor: PointerScanTargetDescriptor,
        validation_target_addresses: Vec<u64>,
    ) -> PointerScanSession {
        PointerScanSession::new(
            original_pointer_scan_session.get_session_id(),
            validation_target_descriptor,
            validation_target_addresses,
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
}

#[cfg(test)]
mod tests {
    use super::PointerScanValidator;
    use crate::pointer_scans::pointer_scan_executor_task::PointerScanExecutor;
    use crate::scanners::scan_execution_context::ScanExecutionContext;
    use squalr_engine_api::structures::memory::bitness::Bitness;
    use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
    use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_level::PointerScanLevel;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_level_candidates::PointerScanLevelCandidates;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_node_type::PointerScanNodeType;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_session::PointerScanSession;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_target_descriptor::PointerScanTargetDescriptor;
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
            PointerScanTargetDescriptor::address(0x4010),
            vec![0x4010],
            &build_snapshot_from_memory_map(&build_validation_memory_regions(), &validation_memory_map),
            &[NormalizedModule::new("game.exe", 0x1000, 0x100)],
            &scan_execution_context,
            false,
        );

        assert_eq!(validated_pointer_scan_session.get_session_id(), original_pointer_scan_session.get_session_id());
        assert_eq!(
            validated_pointer_scan_session
                .get_target_descriptor()
                .get_target_address(),
            Some(0x4010)
        );
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
            PointerScanTargetDescriptor::address(0x8010),
            vec![0x8010],
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
    fn validate_scan_rebuilds_live_heap_candidates_from_validation_snapshot() {
        let original_pointer_scan_session = build_original_pointer_scan_session();
        let validation_memory_map = Arc::new(build_validation_memory_map_with_extra_heap_match());
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
            PointerScanTargetDescriptor::address(0x4010),
            vec![0x4010],
            &build_snapshot_from_memory_map(&build_validation_memory_regions_with_extra_heap_match(), &validation_memory_map),
            &[NormalizedModule::new("game.exe", 0x1000, 0x100)],
            &scan_execution_context,
            false,
        );

        assert_eq!(validated_pointer_scan_session.get_total_heap_node_count(), 2);

        let root_nodes = validated_pointer_scan_session.get_expanded_nodes(None);
        let child_nodes = validated_pointer_scan_session.get_expanded_nodes(Some(root_nodes[0].get_node_id()));
        let grandchild_nodes = validated_pointer_scan_session.get_expanded_nodes(Some(child_nodes[0].get_node_id()));

        assert_eq!(grandchild_nodes.len(), 1);
        assert_eq!(grandchild_nodes[0].get_pointer_address(), 0x3000);
    }

    #[test]
    fn validate_scan_deduplicates_shared_live_heap_children() {
        let original_pointer_scan_session = build_shared_child_original_pointer_scan_session();
        let validation_memory_map = Arc::new(build_shared_child_validation_memory_map());
        let scan_execution_context = ScanExecutionContext::new(
            None,
            None,
            Some(Arc::new({
                let validation_memory_map = validation_memory_map.clone();

                move |_opened_process_info, address, values| read_memory_from_map(&validation_memory_map, address, values)
            })),
        );
        let mut validated_pointer_scan_session = PointerScanValidator::validate_scan(
            OpenedProcessInfo::new(9, "pointer-shared-child-test".to_string(), 0, Bitness::Bit64, None),
            &original_pointer_scan_session,
            PointerScanTargetDescriptor::address(0x4010),
            vec![0x4010],
            &build_snapshot_from_memory_map(&build_shared_child_validation_memory_regions(), &validation_memory_map),
            &[NormalizedModule::new("game.exe", 0x1000, 0x100)],
            &scan_execution_context,
            false,
        );

        assert_eq!(validated_pointer_scan_session.get_root_node_count(), 2);
        assert_eq!(validated_pointer_scan_session.get_total_heap_node_count(), 1);

        let root_nodes = validated_pointer_scan_session.get_expanded_nodes(None);
        assert_eq!(root_nodes.len(), 2);

        let first_child_nodes = validated_pointer_scan_session.get_expanded_nodes(Some(root_nodes[0].get_node_id()));
        let second_child_nodes = validated_pointer_scan_session.get_expanded_nodes(Some(root_nodes[1].get_node_id()));

        assert_eq!(first_child_nodes.len(), 1);
        assert_eq!(second_child_nodes.len(), 1);

        let first_grandchild_nodes = validated_pointer_scan_session.get_expanded_nodes(Some(first_child_nodes[0].get_node_id()));
        let second_grandchild_nodes = validated_pointer_scan_session.get_expanded_nodes(Some(second_child_nodes[0].get_node_id()));

        assert_eq!(first_grandchild_nodes.len(), 1);
        assert_eq!(second_grandchild_nodes.len(), 1);
        assert_eq!(first_grandchild_nodes[0].get_pointer_address(), 0x3000);
        assert_eq!(second_grandchild_nodes[0].get_pointer_address(), 0x3000);
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
        let pointer_scan_parameters = PointerScanParameters::new(PointerScanPointerSize::Pointer64, 0x20, 3, true, false);

        PointerScanExecutor::execute_scan(
            OpenedProcessInfo::new(7, "pointer-test".to_string(), 0, Bitness::Bit64, None),
            snapshot.clone(),
            snapshot,
            41,
            pointer_scan_parameters,
            PointerScanTargetDescriptor::address(0x3010),
            vec![0x3010],
            &[NormalizedModule::new("game.exe", 0x1000, 0x100)],
            false,
            &scan_execution_context,
        )
    }

    fn build_shared_child_original_pointer_scan_session() -> PointerScanSession {
        PointerScanSession::new(
            43,
            PointerScanTargetDescriptor::address(0x4010),
            vec![0x4010],
            PointerScanPointerSize::Pointer64,
            2,
            0x20,
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
                        1,
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
                        PointerScanCandidate::new(2, 2, PointerScanNodeType::Static, 0x1010, 0x3000, 0, 0x10),
                        PointerScanCandidate::new(3, 2, PointerScanNodeType::Static, 0x1020, 0x3000, 0, 0x20),
                    ],
                    Vec::new(),
                ),
            ],
            2,
            1,
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

    fn build_validation_memory_map_with_extra_heap_match() -> HashMap<u64, u8> {
        let mut memory_map = build_validation_memory_map();

        write_pointer_bytes(&mut memory_map, 0x3500, 0x4000_u64);

        memory_map
    }

    fn build_validation_memory_regions() -> Vec<NormalizedRegion> {
        vec![
            NormalizedRegion::new(0x1000, 0x40),
            NormalizedRegion::new(0x3000, 0x40),
        ]
    }

    fn build_validation_memory_regions_with_extra_heap_match() -> Vec<NormalizedRegion> {
        vec![
            NormalizedRegion::new(0x1000, 0x40),
            NormalizedRegion::new(0x3000, 0x600),
        ]
    }

    fn build_shared_child_validation_memory_map() -> HashMap<u64, u8> {
        let mut memory_map = HashMap::new();

        write_pointer_bytes(&mut memory_map, 0x1010, 0x3000_u64);
        write_pointer_bytes(&mut memory_map, 0x1020, 0x3000_u64);
        write_pointer_bytes(&mut memory_map, 0x3000, 0x4000_u64);

        memory_map
    }

    fn build_shared_child_validation_memory_regions() -> Vec<NormalizedRegion> {
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
