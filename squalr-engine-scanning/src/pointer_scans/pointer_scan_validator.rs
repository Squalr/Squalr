use crate::pointer_scans::pointer_scan_range_pass::PointerScanRangePass;
use crate::pointer_scans::pointer_scan_session_builder::PointerScanSessionBuilder;
use crate::pointer_scans::pointer_scan_task_builder::PointerScanTaskBuilder;
use crate::pointer_scans::search_kernels::pointer_scan_pointer_value_reader::read_pointer_value_unchecked;
use crate::pointer_scans::structures::pointer_scan_collected_candidate::PointerScanCollectedCandidate;
use crate::pointer_scans::structures::pointer_scan_collected_level::PointerScanCollectedLevel;
use crate::pointer_scans::structures::pointer_scan_target_ranges::PointerScanTargetRangeSet;
use crate::pointer_scans::structures::snapshot_region_scan_task::SnapshotRegionScanTask;
use crate::scanners::scan_execution_context::ScanExecutionContext;
use rayon::prelude::*;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::pointer_scans::pointer_scan_candidate::PointerScanCandidate;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::pointer_scans::pointer_scan_session::PointerScanSession;
use squalr_engine_api::structures::pointer_scans::pointer_scan_target_descriptor::PointerScanTargetDescriptor;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use std::time::Instant;

const POINTER_VALIDATION_ROOT_CHUNK_SIZE: usize = 64;

#[derive(Clone, Copy, Debug)]
struct PointerValidationProgress {
    current_level_number: usize,
    total_level_count: usize,
}

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
            PointerScanTaskBuilder::build_heap_scan_tasks(validation_context.validation_snapshot_regions, modules, validation_context.pointer_size);
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
    ) -> Vec<PointerScanCollectedLevel> {
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

            let validation_progress = PointerValidationProgress {
                current_level_number: level_index + 1,
                total_level_count: level_count,
            };
            let level_start_time = Instant::now();
            let retain_heap_candidates = level_index.saturating_add(1) < level_count;

            if with_logging {
                log::info!(
                    "Pointer scan validation level {}/{}: rebuilding heaps from {} frontier targets and pruning {} stored static roots.",
                    validation_progress.current_level_number,
                    validation_progress.total_level_count,
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
                    validation_progress.current_level_number,
                    validation_progress.total_level_count,
                    level_start_time.elapsed(),
                    validated_static_candidates.len(),
                    validated_heap_candidates.len(),
                );
            }

            validated_pointer_levels.push(PointerScanCollectedLevel {
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
    ) -> Vec<PointerScanCollectedCandidate> {
        if frontier_target_ranges.is_empty() {
            return Vec::new();
        }

        let pointer_scan_execution_plan = PointerScanRangePass::build_execution_plan(frontier_target_ranges, pointer_size, validation_heap_scan_tasks);

        PointerScanRangePass::collect_candidates(validation_heap_scan_tasks, frontier_target_ranges, &pointer_scan_execution_plan, false, true).heap_candidates
    }

    fn collect_validated_static_candidates(
        pointer_scan_session: &PointerScanSession,
        static_pointer_scan_candidates: &[PointerScanCandidate],
        validation_context: &PointerValidationContext<'_>,
        frontier_target_ranges: &PointerScanTargetRangeSet,
    ) -> Vec<PointerScanCollectedCandidate> {
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

                    validated_static_candidates.push(PointerScanCollectedCandidate {
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

    fn truncate_empty_trailing_levels(validated_pointer_levels: &mut Vec<PointerScanCollectedLevel>) {
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
        validated_pointer_levels: Vec<PointerScanCollectedLevel>,
    ) -> PointerScanSession {
        PointerScanSessionBuilder::build_session_with_module_names(
            original_pointer_scan_session.get_session_id(),
            &squalr_engine_api::structures::scanning::plans::pointer_scan::pointer_scan_parameters::PointerScanParameters::new(
                original_pointer_scan_session.get_pointer_size(),
                original_pointer_scan_session.get_offset_radius(),
                original_pointer_scan_session.get_max_depth(),
                false,
                false,
            ),
            validation_target_descriptor,
            validation_target_addresses,
            original_pointer_scan_session.get_address_space(),
            original_pointer_scan_session.get_module_names().clone(),
            &validated_pointer_levels,
            false,
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
            original_pointer_scan_session.get_address_space(),
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
    use squalr_engine_api::structures::pointer_scans::pointer_scan_address_space::PointerScanAddressSpace;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_browser::PointerScanBrowser;
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

        let mut pointer_scan_browser = PointerScanBrowser::new();
        let root_nodes = pointer_scan_browser.get_expanded_nodes(&mut validated_pointer_scan_session, None);
        assert_eq!(root_nodes.len(), 1);
        assert_eq!(root_nodes[0].get_pointer_address(), 0x1010);
        assert_eq!(root_nodes[0].get_pointer_scan_node_type(), PointerScanNodeType::Static);
        assert_eq!(root_nodes[0].get_resolved_target_address(), 0x2FF0);
        assert_eq!(root_nodes[0].get_pointer_offset(), 0);

        let child_nodes = pointer_scan_browser.get_expanded_nodes(&mut validated_pointer_scan_session, Some(root_nodes[0].get_node_id()));
        assert_eq!(child_nodes.len(), 1);
        assert_eq!(child_nodes[0].get_pointer_address(), 0x1010);
        assert_eq!(child_nodes[0].get_pointer_scan_node_type(), PointerScanNodeType::Static);
        assert_eq!(child_nodes[0].get_resolved_target_address(), 0x3000);
        assert_eq!(child_nodes[0].get_pointer_offset(), 0x10);

        let grandchild_nodes = pointer_scan_browser.get_expanded_nodes(&mut validated_pointer_scan_session, Some(child_nodes[0].get_node_id()));
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

        let mut pointer_scan_browser = PointerScanBrowser::new();
        let root_nodes = pointer_scan_browser.get_expanded_nodes(&mut validated_pointer_scan_session, None);
        assert_eq!(root_nodes.len(), 1);
        assert_eq!(root_nodes[0].get_pointer_address(), 0x5010);
        assert_eq!(root_nodes[0].get_module_name(), "game.exe");
        assert_eq!(root_nodes[0].get_module_offset(), 0x10);

        let child_nodes = pointer_scan_browser.get_expanded_nodes(&mut validated_pointer_scan_session, Some(root_nodes[0].get_node_id()));
        assert_eq!(child_nodes.len(), 1);
        assert_eq!(child_nodes[0].get_pointer_address(), 0x5010);
        assert_eq!(child_nodes[0].get_pointer_scan_node_type(), PointerScanNodeType::Static);

        let grandchild_nodes = pointer_scan_browser.get_expanded_nodes(&mut validated_pointer_scan_session, Some(child_nodes[0].get_node_id()));
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

        let mut pointer_scan_browser = PointerScanBrowser::new();
        let root_nodes = pointer_scan_browser.get_expanded_nodes(&mut validated_pointer_scan_session, None);
        let child_nodes = pointer_scan_browser.get_expanded_nodes(&mut validated_pointer_scan_session, Some(root_nodes[0].get_node_id()));
        let grandchild_nodes = pointer_scan_browser.get_expanded_nodes(&mut validated_pointer_scan_session, Some(child_nodes[0].get_node_id()));

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

        let mut pointer_scan_browser = PointerScanBrowser::new();
        let root_nodes = pointer_scan_browser.get_expanded_nodes(&mut validated_pointer_scan_session, None);
        assert_eq!(root_nodes.len(), 2);

        let first_child_nodes = pointer_scan_browser.get_expanded_nodes(&mut validated_pointer_scan_session, Some(root_nodes[0].get_node_id()));
        let second_child_nodes = pointer_scan_browser.get_expanded_nodes(&mut validated_pointer_scan_session, Some(root_nodes[1].get_node_id()));

        assert_eq!(first_child_nodes.len(), 1);
        assert_eq!(second_child_nodes.len(), 1);

        let first_grandchild_nodes = pointer_scan_browser.get_expanded_nodes(&mut validated_pointer_scan_session, Some(first_child_nodes[0].get_node_id()));
        let second_grandchild_nodes = pointer_scan_browser.get_expanded_nodes(&mut validated_pointer_scan_session, Some(second_child_nodes[0].get_node_id()));

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
            PointerScanAddressSpace::EmulatorMemory,
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
            PointerScanAddressSpace::EmulatorMemory,
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
