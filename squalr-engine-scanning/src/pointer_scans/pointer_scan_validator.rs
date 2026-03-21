use crate::pointer_scans::search_kernels::pointer_scan_pointer_value_reader::read_pointer_value_unchecked;
use crate::pointer_scans::structures::pointer_validation_level_log_context::PointerValidationLevelLogContext;
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
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use std::time::Instant;

const POINTER_VALIDATION_ROOT_CHUNK_SIZE: usize = 64;

struct PointerValidationContext<'a> {
    validation_snapshot_regions: &'a [SnapshotRegion],
    current_modules_by_name: Vec<(&'a str, u64)>,
    validation_target_address: u64,
    offset_radius: u64,
    pointer_size: PointerScanPointerSize,
    scan_execution_context: &'a ScanExecutionContext,
}

impl<'a> PointerValidationContext<'a> {
    fn new(
        original_pointer_scan_session: &'a PointerScanSession,
        validation_target_address: u64,
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
            validation_target_address,
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

        let validation_context = PointerValidationContext::new(
            pointer_scan_session,
            validation_target_address,
            validation_snapshot,
            modules,
            scan_execution_context,
        );
        let level_count = pointer_scan_session.get_pointer_scan_level_candidates().len();
        let mut validated_pointer_levels = vec![ValidatedPointerLevel::default(); level_count];

        for (level_index, pointer_scan_level_candidates) in pointer_scan_session
            .get_pointer_scan_level_candidates()
            .iter()
            .enumerate()
        {
            if scan_execution_context.should_cancel() {
                break;
            }

            let validation_level_log_context = PointerValidationLevelLogContext {
                level_number: level_index + 1,
                level_count,
            };
            let level_start_time = Instant::now();

            if with_logging {
                log::info!(
                    "Pointer scan validation level {}/{}: validating {} stored static roots against the fresh snapshot.",
                    validation_level_log_context.level_number,
                    validation_level_log_context.level_count,
                    pointer_scan_level_candidates.get_static_node_count(),
                );
            }

            let validated_pointer_level_chunks = pointer_scan_level_candidates
                .get_static_candidates()
                .par_chunks(POINTER_VALIDATION_ROOT_CHUNK_SIZE)
                .map(|static_pointer_scan_candidate_chunk| {
                    let mut worker_validated_pointer_levels = vec![ValidatedPointerLevel::default(); level_count];

                    for static_pointer_scan_candidate in static_pointer_scan_candidate_chunk {
                        if validation_context.scan_execution_context.should_cancel() {
                            break;
                        }

                        Self::validate_static_candidate(
                            pointer_scan_session,
                            static_pointer_scan_candidate,
                            level_index,
                            &validation_context,
                            &mut worker_validated_pointer_levels,
                        );
                    }

                    Self::normalize_worker_validated_pointer_levels(&mut worker_validated_pointer_levels);

                    worker_validated_pointer_levels
                })
                .collect::<Vec<_>>();
            let mut retained_static_root_count = 0_u64;

            for mut worker_validated_pointer_levels in validated_pointer_level_chunks {
                retained_static_root_count = retained_static_root_count.saturating_add(
                    worker_validated_pointer_levels[level_index]
                        .static_candidates
                        .len() as u64,
                );

                Self::append_validated_pointer_level(&mut validated_pointer_levels[level_index], &mut worker_validated_pointer_levels[level_index]);

                for child_level_index in 0..level_count {
                    if child_level_index == level_index {
                        continue;
                    }

                    Self::append_validated_pointer_level(
                        &mut validated_pointer_levels[child_level_index],
                        &mut worker_validated_pointer_levels[child_level_index],
                    );
                }
            }

            if with_logging {
                log::info!(
                    "Pointer scan validation level {}/{} complete in {:?}: retained {} static roots.",
                    validation_level_log_context.level_number,
                    validation_level_log_context.level_count,
                    level_start_time.elapsed(),
                    retained_static_root_count,
                );
            }
        }

        Self::truncate_empty_trailing_levels(&mut validated_pointer_levels);

        let validated_pointer_scan_session = if validated_pointer_levels.is_empty() {
            Self::create_empty_session(pointer_scan_session, validation_target_address)
        } else {
            Self::build_pointer_scan_session(pointer_scan_session, validation_target_address, validated_pointer_levels)
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

    fn validate_static_candidate(
        pointer_scan_session: &PointerScanSession,
        static_pointer_scan_candidate: &PointerScanCandidate,
        level_index: usize,
        validation_context: &PointerValidationContext<'_>,
        validated_pointer_levels: &mut [ValidatedPointerLevel],
    ) -> bool {
        let Some(current_pointer_address) =
            Self::resolve_current_static_pointer_address(pointer_scan_session, static_pointer_scan_candidate, validation_context)
        else {
            return false;
        };
        let Some(current_pointer_value) = Self::read_pointer_value_at_address(
            validation_context.validation_snapshot_regions,
            current_pointer_address,
            validation_context.pointer_size,
        ) else {
            return false;
        };

        if !Self::validate_candidate_children(
            pointer_scan_session,
            static_pointer_scan_candidate,
            level_index,
            current_pointer_value,
            validation_context,
            validated_pointer_levels,
        ) {
            return false;
        }

        validated_pointer_levels[level_index]
            .static_candidates
            .push(ValidatedPointerCandidate {
                pointer_address: current_pointer_address,
                pointer_value: current_pointer_value,
                module_index: static_pointer_scan_candidate.get_module_index(),
                module_offset: static_pointer_scan_candidate.get_module_offset(),
            });

        true
    }

    fn validate_heap_candidate(
        pointer_scan_session: &PointerScanSession,
        heap_pointer_scan_candidate: &PointerScanCandidate,
        level_index: usize,
        current_pointer_address: u64,
        current_pointer_value: u64,
        validation_context: &PointerValidationContext<'_>,
        validated_pointer_levels: &mut [ValidatedPointerLevel],
    ) -> bool {
        if !Self::validate_candidate_children(
            pointer_scan_session,
            heap_pointer_scan_candidate,
            level_index,
            current_pointer_value,
            validation_context,
            validated_pointer_levels,
        ) {
            return false;
        }

        validated_pointer_levels[level_index]
            .heap_candidates
            .push(ValidatedPointerCandidate {
                pointer_address: current_pointer_address,
                pointer_value: current_pointer_value,
                module_index: 0,
                module_offset: 0,
            });

        true
    }

    fn validate_candidate_children(
        pointer_scan_session: &PointerScanSession,
        original_pointer_scan_candidate: &PointerScanCandidate,
        level_index: usize,
        current_pointer_value: u64,
        validation_context: &PointerValidationContext<'_>,
        validated_pointer_levels: &mut [ValidatedPointerLevel],
    ) -> bool {
        if level_index == 0 {
            return Self::pointer_value_reaches_target(
                current_pointer_value,
                validation_context.validation_target_address,
                validation_context.offset_radius,
            );
        }

        let child_level_index = level_index.saturating_sub(1);
        let lower_bound = original_pointer_scan_candidate
            .get_pointer_value()
            .saturating_sub(validation_context.offset_radius);
        let upper_bound = original_pointer_scan_candidate
            .get_pointer_value()
            .saturating_add(validation_context.offset_radius);
        let Some(child_level_candidates) = pointer_scan_session
            .get_pointer_scan_level_candidates()
            .get(child_level_index)
        else {
            return false;
        };
        let original_child_heap_candidates =
            Self::find_original_heap_candidates_in_range(child_level_candidates.get_heap_candidates(), lower_bound, upper_bound);
        let mut snapshot_region_index_hint = None;

        for child_heap_pointer_scan_candidate in original_child_heap_candidates {
            if validation_context.scan_execution_context.should_cancel() {
                return false;
            }

            let Some(rebased_child_pointer_address) = Self::rebase_child_pointer_address(
                current_pointer_value,
                original_pointer_scan_candidate.get_pointer_value(),
                child_heap_pointer_scan_candidate.get_pointer_address(),
            ) else {
                continue;
            };

            let Some(current_child_pointer_value) = Self::read_pointer_value_at_address_in_order(
                validation_context.validation_snapshot_regions,
                rebased_child_pointer_address,
                validation_context.pointer_size,
                &mut snapshot_region_index_hint,
            ) else {
                continue;
            };

            if Self::validate_heap_candidate(
                pointer_scan_session,
                child_heap_pointer_scan_candidate,
                child_level_index,
                rebased_child_pointer_address,
                current_child_pointer_value,
                validation_context,
                validated_pointer_levels,
            ) {
                return true;
            }
        }

        false
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

    fn rebase_child_pointer_address(
        current_parent_pointer_value: u64,
        original_parent_pointer_value: u64,
        original_child_pointer_address: u64,
    ) -> Option<u64> {
        let child_pointer_relative_offset = original_child_pointer_address as i128 - original_parent_pointer_value as i128;
        let rebased_child_pointer_address = current_parent_pointer_value as i128 + child_pointer_relative_offset;

        u64::try_from(rebased_child_pointer_address).ok()
    }

    fn pointer_value_reaches_target(
        pointer_value: u64,
        target_address: u64,
        offset_radius: u64,
    ) -> bool {
        pointer_value >= target_address.saturating_sub(offset_radius) && pointer_value <= target_address.saturating_add(offset_radius)
    }

    fn find_original_heap_candidates_in_range<'a>(
        original_heap_candidates: &'a [PointerScanCandidate],
        lower_bound: u64,
        upper_bound: u64,
    ) -> &'a [PointerScanCandidate] {
        let start_index = Self::find_first_heap_candidate_index_at_or_above(original_heap_candidates, lower_bound);
        let end_index = Self::find_first_heap_candidate_index_above(original_heap_candidates, upper_bound);

        &original_heap_candidates[start_index..end_index]
    }

    fn find_first_heap_candidate_index_at_or_above(
        original_heap_candidates: &[PointerScanCandidate],
        lower_bound: u64,
    ) -> usize {
        let mut lower_index = 0_usize;
        let mut upper_index = original_heap_candidates.len();

        while lower_index < upper_index {
            let middle_index = lower_index.saturating_add(upper_index.saturating_sub(lower_index) / 2);

            if original_heap_candidates[middle_index].get_pointer_address() < lower_bound {
                lower_index = middle_index.saturating_add(1);
            } else {
                upper_index = middle_index;
            }
        }

        lower_index
    }

    fn find_first_heap_candidate_index_above(
        original_heap_candidates: &[PointerScanCandidate],
        upper_bound: u64,
    ) -> usize {
        let mut lower_index = 0_usize;
        let mut upper_index = original_heap_candidates.len();

        while lower_index < upper_index {
            let middle_index = lower_index.saturating_add(upper_index.saturating_sub(lower_index) / 2);

            if original_heap_candidates[middle_index].get_pointer_address() <= upper_bound {
                lower_index = middle_index.saturating_add(1);
            } else {
                upper_index = middle_index;
            }
        }

        lower_index
    }

    fn append_validated_pointer_level(
        accumulated_validated_pointer_level: &mut ValidatedPointerLevel,
        worker_validated_pointer_level: &mut ValidatedPointerLevel,
    ) {
        accumulated_validated_pointer_level
            .static_candidates
            .append(&mut worker_validated_pointer_level.static_candidates);
        accumulated_validated_pointer_level.heap_candidates = Self::merge_sorted_heap_candidates(
            std::mem::take(&mut accumulated_validated_pointer_level.heap_candidates),
            std::mem::take(&mut worker_validated_pointer_level.heap_candidates),
        );
    }

    fn normalize_worker_validated_pointer_levels(validated_pointer_levels: &mut [ValidatedPointerLevel]) {
        for validated_pointer_level in validated_pointer_levels {
            validated_pointer_level
                .heap_candidates
                .sort_unstable_by_key(|validated_pointer_candidate| validated_pointer_candidate.pointer_address);
            validated_pointer_level
                .heap_candidates
                .dedup_by_key(|validated_pointer_candidate| validated_pointer_candidate.pointer_address);
        }
    }

    fn merge_sorted_heap_candidates(
        left_heap_candidates: Vec<ValidatedPointerCandidate>,
        right_heap_candidates: Vec<ValidatedPointerCandidate>,
    ) -> Vec<ValidatedPointerCandidate> {
        if left_heap_candidates.is_empty() {
            return right_heap_candidates;
        }

        if right_heap_candidates.is_empty() {
            return left_heap_candidates;
        }

        let mut left_heap_candidates = left_heap_candidates.into_iter().peekable();
        let mut right_heap_candidates = right_heap_candidates.into_iter().peekable();
        let mut merged_heap_candidates = Vec::with_capacity(
            left_heap_candidates
                .size_hint()
                .0
                .saturating_add(right_heap_candidates.size_hint().0),
        );
        let mut last_pointer_address = None;

        while let (Some(left_heap_candidate), Some(right_heap_candidate)) = (left_heap_candidates.peek(), right_heap_candidates.peek()) {
            let next_heap_candidate = if left_heap_candidate.pointer_address <= right_heap_candidate.pointer_address {
                left_heap_candidates.next()
            } else {
                right_heap_candidates.next()
            };
            let Some(next_heap_candidate) = next_heap_candidate else {
                continue;
            };

            if last_pointer_address != Some(next_heap_candidate.pointer_address) {
                last_pointer_address = Some(next_heap_candidate.pointer_address);
                merged_heap_candidates.push(next_heap_candidate);
            }
        }

        for remaining_heap_candidate in left_heap_candidates.chain(right_heap_candidates) {
            if last_pointer_address != Some(remaining_heap_candidate.pointer_address) {
                last_pointer_address = Some(remaining_heap_candidate.pointer_address);
                merged_heap_candidates.push(remaining_heap_candidate);
            }
        }

        merged_heap_candidates
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

    fn read_pointer_value_at_address_in_order(
        snapshot_regions: &[SnapshotRegion],
        pointer_address: u64,
        pointer_size: PointerScanPointerSize,
        snapshot_region_index_hint: &mut Option<usize>,
    ) -> Option<u64> {
        let pointer_size_in_bytes = pointer_size.get_size_in_bytes() as usize;

        if snapshot_region_index_hint.is_none() {
            *snapshot_region_index_hint = Self::find_snapshot_region_index_for_pointer_address(snapshot_regions, pointer_address, pointer_size_in_bytes);
        }

        let snapshot_region_index_hint = snapshot_region_index_hint.as_mut()?;

        Self::read_pointer_value_from_snapshot_region(snapshot_regions, pointer_address, pointer_size, snapshot_region_index_hint)
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
        validation_target_address: u64,
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

        let root_node_count = total_static_node_count;

        PointerScanSession::new(
            original_pointer_scan_session.get_session_id(),
            validation_target_address,
            original_pointer_scan_session.get_pointer_size(),
            original_pointer_scan_session.get_max_depth(),
            original_pointer_scan_session.get_offset_radius(),
            original_pointer_scan_session.get_module_names().clone(),
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
    fn validate_scan_does_not_rediscover_unstored_heap_candidates() {
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
            0x4010,
            &build_snapshot_from_memory_map(&build_validation_memory_regions_with_extra_heap_match(), &validation_memory_map),
            &[NormalizedModule::new("game.exe", 0x1000, 0x100)],
            &scan_execution_context,
            false,
        );

        assert_eq!(validated_pointer_scan_session.get_total_heap_node_count(), 1);

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
            0x4010,
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

    fn build_shared_child_original_pointer_scan_session() -> PointerScanSession {
        PointerScanSession::new(
            43,
            0x4010,
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
