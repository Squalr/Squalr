use crate::pointer_scans::structures::pointer_validation_heap_candidate_index::PointerValidationHeapCandidateIndex;
use crate::pointer_scans::structures::pointer_validation_level_log_context::PointerValidationLevelLogContext;
use crate::pointer_scans::structures::pointer_validation_snapshot_region_index::PointerValidationSnapshotRegionIndex;
use crate::pointer_scans::structures::validated_pointer_candidate::ValidatedPointerCandidate;
use crate::pointer_scans::structures::validated_pointer_candidate_key::ValidatedPointerCandidateKey;
use crate::pointer_scans::structures::validated_pointer_candidate_state::ValidatedPointerCandidateState;
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
use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Mutex;
use std::time::Instant;

const POINTER_VALIDATION_MEMO_SHARDS_PER_WORKER: usize = 8;

type PointerValidationMemo = Vec<Mutex<HashMap<ValidatedPointerCandidateKey, ValidatedPointerCandidateState>>>;

struct PointerValidationContext<'a> {
    original_pointer_scan_session: &'a PointerScanSession,
    validation_snapshot_region_index: PointerValidationSnapshotRegionIndex<'a>,
    current_modules_by_name: Vec<(&'a str, u64)>,
    original_heap_candidate_indices_by_level: Vec<PointerValidationHeapCandidateIndex<'a>>,
    original_candidates_by_id: Vec<Option<&'a PointerScanCandidate>>,
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

        let original_heap_candidate_indices_by_level = original_pointer_scan_session
            .get_pointer_scan_level_candidates()
            .iter()
            .map(|pointer_scan_level_candidates| PointerValidationHeapCandidateIndex::new(pointer_scan_level_candidates.get_heap_candidates()))
            .collect::<Vec<_>>();
        let mut original_candidates_by_id = vec![None; original_pointer_scan_session.get_total_node_count() as usize + 1];

        for pointer_scan_level_candidates in original_pointer_scan_session.get_pointer_scan_level_candidates() {
            for pointer_scan_candidate in pointer_scan_level_candidates
                .get_static_candidates()
                .iter()
                .chain(pointer_scan_level_candidates.get_heap_candidates().iter())
            {
                let candidate_index = pointer_scan_candidate.get_candidate_id() as usize;

                if candidate_index >= original_candidates_by_id.len() {
                    original_candidates_by_id.resize(candidate_index.saturating_add(1), None);
                }

                original_candidates_by_id[candidate_index] = Some(pointer_scan_candidate);
            }
        }

        Self {
            original_pointer_scan_session,
            validation_snapshot_region_index: PointerValidationSnapshotRegionIndex::new(validation_snapshot.get_snapshot_regions().as_slice()),
            current_modules_by_name,
            original_heap_candidate_indices_by_level,
            original_candidates_by_id,
            validation_target_address,
            offset_radius: original_pointer_scan_session.get_offset_radius(),
            pointer_size: original_pointer_scan_session.get_pointer_size(),
            scan_execution_context,
        }
    }

    fn find_original_pointer_scan_candidate(
        &self,
        candidate_id: u64,
    ) -> Option<&'a PointerScanCandidate> {
        self.original_candidates_by_id
            .get(candidate_id as usize)
            .copied()
            .flatten()
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
        let validation_memo = Self::create_validation_memo();
        let level_count = pointer_scan_session.get_pointer_scan_level_candidates().len();

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

            let retained_static_root_count = pointer_scan_level_candidates
                .get_static_candidates()
                .par_iter()
                .filter(|static_pointer_scan_candidate| {
                    let Some(current_pointer_address) = Self::resolve_current_static_pointer_address(static_pointer_scan_candidate, &validation_context) else {
                        return false;
                    };

                    Self::validate_pointer_candidate_context(
                        static_pointer_scan_candidate,
                        level_index,
                        current_pointer_address,
                        &validation_context,
                        &validation_memo,
                    )
                    .is_some()
                })
                .count() as u64;

            if with_logging {
                let memoized_heap_node_count = Self::count_validated_heap_contexts(&validation_context, &validation_memo);

                log::info!(
                    "Pointer scan validation level {}/{} complete in {:?}: retained {} static roots and memoized {} descendant heap contexts so far.",
                    validation_level_log_context.level_number,
                    validation_level_log_context.level_count,
                    level_start_time.elapsed(),
                    retained_static_root_count,
                    memoized_heap_node_count,
                );
            }
        }

        let validated_pointer_levels = Self::collect_validated_pointer_levels(&validation_context, &validation_memo, level_count);

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

    fn validate_pointer_candidate_context(
        pointer_scan_candidate: &PointerScanCandidate,
        level_index: usize,
        current_pointer_address: u64,
        validation_context: &PointerValidationContext<'_>,
        validation_memo: &PointerValidationMemo,
    ) -> Option<u64> {
        if validation_context.scan_execution_context.should_cancel() {
            return None;
        }

        let validated_pointer_candidate_key =
            ValidatedPointerCandidateKey::new(level_index, pointer_scan_candidate.get_candidate_id(), current_pointer_address);

        if let Some(validated_pointer_candidate_state) = Self::get_validation_memo_state(validation_memo, &validated_pointer_candidate_key) {
            return match validated_pointer_candidate_state {
                ValidatedPointerCandidateState::Invalid => None,
                ValidatedPointerCandidateState::Valid { current_pointer_value } => Some(current_pointer_value),
            };
        }

        let Some(current_pointer_value) = validation_context
            .validation_snapshot_region_index
            .read_pointer_value_at_address(current_pointer_address, validation_context.pointer_size)
        else {
            Self::insert_validation_memo_state(validation_memo, validated_pointer_candidate_key, ValidatedPointerCandidateState::Invalid);

            return None;
        };

        let is_valid = if level_index == 0 {
            Self::pointer_value_reaches_target(
                current_pointer_value,
                validation_context.validation_target_address,
                validation_context.offset_radius,
            )
        } else {
            Self::validate_pointer_candidate_children(pointer_scan_candidate, level_index, current_pointer_value, validation_context, validation_memo)
        };

        if !is_valid {
            Self::insert_validation_memo_state(validation_memo, validated_pointer_candidate_key, ValidatedPointerCandidateState::Invalid);

            return None;
        }

        Self::insert_validation_memo_state(
            validation_memo,
            validated_pointer_candidate_key,
            ValidatedPointerCandidateState::Valid { current_pointer_value },
        );

        Some(current_pointer_value)
    }

    fn validate_pointer_candidate_children(
        original_pointer_scan_candidate: &PointerScanCandidate,
        level_index: usize,
        current_pointer_value: u64,
        validation_context: &PointerValidationContext<'_>,
        validation_memo: &PointerValidationMemo,
    ) -> bool {
        let child_level_index = level_index.saturating_sub(1);
        let lower_bound = original_pointer_scan_candidate
            .get_pointer_value()
            .saturating_sub(validation_context.offset_radius);
        let upper_bound = original_pointer_scan_candidate
            .get_pointer_value()
            .saturating_add(validation_context.offset_radius);

        for child_heap_pointer_scan_candidate in
            validation_context.original_heap_candidate_indices_by_level[child_level_index].find_candidates_in_range(lower_bound, upper_bound)
        {
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

            if Self::validate_pointer_candidate_context(
                child_heap_pointer_scan_candidate,
                child_level_index,
                rebased_child_pointer_address,
                validation_context,
                validation_memo,
            )
            .is_some()
            {
                return true;
            }
        }

        false
    }

    fn resolve_current_static_pointer_address(
        static_pointer_scan_candidate: &PointerScanCandidate,
        validation_context: &PointerValidationContext<'_>,
    ) -> Option<u64> {
        let module_name = validation_context
            .original_pointer_scan_session
            .get_module_name(static_pointer_scan_candidate.get_module_index())?;
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

    fn create_validation_memo() -> PointerValidationMemo {
        let validation_memo_shard_count = rayon::current_num_threads()
            .max(1)
            .saturating_mul(POINTER_VALIDATION_MEMO_SHARDS_PER_WORKER);

        (0..validation_memo_shard_count)
            .map(|_| Mutex::new(HashMap::new()))
            .collect()
    }

    fn get_validation_memo_state(
        validation_memo: &PointerValidationMemo,
        validated_pointer_candidate_key: &ValidatedPointerCandidateKey,
    ) -> Option<ValidatedPointerCandidateState> {
        let memo_shard_index = Self::get_validation_memo_shard_index(validation_memo, validated_pointer_candidate_key);
        let validation_memo_shard = validation_memo.get(memo_shard_index)?;
        let validation_memo_shard_guard = match validation_memo_shard.lock() {
            Ok(validation_memo_shard_guard) => validation_memo_shard_guard,
            Err(poisoned_validation_memo_shard_guard) => poisoned_validation_memo_shard_guard.into_inner(),
        };

        validation_memo_shard_guard
            .get(validated_pointer_candidate_key)
            .copied()
    }

    fn insert_validation_memo_state(
        validation_memo: &PointerValidationMemo,
        validated_pointer_candidate_key: ValidatedPointerCandidateKey,
        validated_pointer_candidate_state: ValidatedPointerCandidateState,
    ) {
        let memo_shard_index = Self::get_validation_memo_shard_index(validation_memo, &validated_pointer_candidate_key);
        let Some(validation_memo_shard) = validation_memo.get(memo_shard_index) else {
            return;
        };
        let mut validation_memo_shard_guard = match validation_memo_shard.lock() {
            Ok(validation_memo_shard_guard) => validation_memo_shard_guard,
            Err(poisoned_validation_memo_shard_guard) => poisoned_validation_memo_shard_guard.into_inner(),
        };

        validation_memo_shard_guard
            .entry(validated_pointer_candidate_key)
            .or_insert(validated_pointer_candidate_state);
    }

    fn get_validation_memo_shard_index(
        validation_memo: &PointerValidationMemo,
        validated_pointer_candidate_key: &ValidatedPointerCandidateKey,
    ) -> usize {
        if validation_memo.is_empty() {
            return 0;
        }

        let mut hasher = DefaultHasher::new();
        validated_pointer_candidate_key.hash(&mut hasher);

        (hasher.finish() as usize) % validation_memo.len()
    }

    fn count_validated_heap_contexts(
        validation_context: &PointerValidationContext<'_>,
        validation_memo: &PointerValidationMemo,
    ) -> u64 {
        let mut validated_heap_context_count = 0_u64;

        for validation_memo_shard in validation_memo {
            let validation_memo_shard_guard = match validation_memo_shard.lock() {
                Ok(validation_memo_shard_guard) => validation_memo_shard_guard,
                Err(poisoned_validation_memo_shard_guard) => poisoned_validation_memo_shard_guard.into_inner(),
            };

            for (validated_pointer_candidate_key, validated_pointer_candidate_state) in validation_memo_shard_guard.iter() {
                if !matches!(validated_pointer_candidate_state, ValidatedPointerCandidateState::Valid { .. }) {
                    continue;
                }

                let Some(original_pointer_scan_candidate) =
                    validation_context.find_original_pointer_scan_candidate(validated_pointer_candidate_key.candidate_id)
                else {
                    continue;
                };

                if original_pointer_scan_candidate.get_pointer_scan_node_type() == PointerScanNodeType::Heap {
                    validated_heap_context_count = validated_heap_context_count.saturating_add(1);
                }
            }
        }

        validated_heap_context_count
    }

    fn collect_validated_pointer_levels(
        validation_context: &PointerValidationContext<'_>,
        validation_memo: &PointerValidationMemo,
        level_count: usize,
    ) -> Vec<ValidatedPointerLevel> {
        let mut validated_pointer_levels = vec![ValidatedPointerLevel::default(); level_count];

        for validation_memo_shard in validation_memo {
            let validation_memo_shard_guard = match validation_memo_shard.lock() {
                Ok(validation_memo_shard_guard) => validation_memo_shard_guard,
                Err(poisoned_validation_memo_shard_guard) => poisoned_validation_memo_shard_guard.into_inner(),
            };

            for (validated_pointer_candidate_key, validated_pointer_candidate_state) in validation_memo_shard_guard.iter() {
                let ValidatedPointerCandidateState::Valid { current_pointer_value } = *validated_pointer_candidate_state else {
                    continue;
                };
                let Some(original_pointer_scan_candidate) =
                    validation_context.find_original_pointer_scan_candidate(validated_pointer_candidate_key.candidate_id)
                else {
                    continue;
                };
                let Some(validated_pointer_level) = validated_pointer_levels.get_mut(validated_pointer_candidate_key.level_index) else {
                    continue;
                };
                let validated_pointer_candidate = ValidatedPointerCandidate {
                    pointer_address: validated_pointer_candidate_key.current_pointer_address,
                    pointer_value: current_pointer_value,
                    module_index: original_pointer_scan_candidate.get_module_index(),
                    module_offset: original_pointer_scan_candidate.get_module_offset(),
                };

                match original_pointer_scan_candidate.get_pointer_scan_node_type() {
                    PointerScanNodeType::Static => validated_pointer_level
                        .static_candidates
                        .push(validated_pointer_candidate),
                    PointerScanNodeType::Heap => validated_pointer_level
                        .heap_candidates
                        .push(validated_pointer_candidate),
                }
            }
        }

        Self::sort_and_deduplicate_validated_pointer_levels(&mut validated_pointer_levels);
        Self::truncate_empty_trailing_levels(&mut validated_pointer_levels);

        validated_pointer_levels
    }

    fn sort_and_deduplicate_validated_pointer_levels(validated_pointer_levels: &mut [ValidatedPointerLevel]) {
        for validated_pointer_level in validated_pointer_levels {
            validated_pointer_level
                .static_candidates
                .sort_unstable_by_key(|validated_pointer_candidate| {
                    (
                        validated_pointer_candidate.module_index,
                        validated_pointer_candidate.module_offset,
                        validated_pointer_candidate.pointer_address,
                    )
                });
            validated_pointer_level
                .static_candidates
                .dedup_by_key(|validated_pointer_candidate| {
                    (
                        validated_pointer_candidate.module_index,
                        validated_pointer_candidate.module_offset,
                        validated_pointer_candidate.pointer_address,
                    )
                });
            validated_pointer_level
                .heap_candidates
                .sort_unstable_by_key(|validated_pointer_candidate| validated_pointer_candidate.pointer_address);
            validated_pointer_level
                .heap_candidates
                .dedup_by_key(|validated_pointer_candidate| validated_pointer_candidate.pointer_address);
        }
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
