use crate::pointer_scans::pointer_scan_range_pass::PointerScanRangePass;
use crate::pointer_scans::pointer_scan_task_builder::PointerScanTaskBuilder;
use crate::pointer_scans::structures::pointer_scan_collected_level::PointerScanCollectedLevel;
use crate::pointer_scans::structures::pointer_scan_target_ranges::PointerScanTargetRangeSet;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::scanning::plans::pointer_scan::pointer_scan_parameters::PointerScanParameters;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use std::time::Instant;

pub(crate) struct PointerScanLevelCollector;

impl PointerScanLevelCollector {
    pub(crate) fn discover_pointer_levels(
        snapshots: &[&Snapshot],
        target_addresses: &[u64],
        pointer_scan_parameters: &PointerScanParameters,
        modules: &[NormalizedModule],
        with_logging: bool,
    ) -> Vec<PointerScanCollectedLevel> {
        let discovery_start_time = Instant::now();
        let max_depth = pointer_scan_parameters.get_max_depth();

        if max_depth == 0 {
            return Vec::new();
        }

        let mut frontier_target_ranges = PointerScanTargetRangeSet::from_target_addresses(target_addresses, pointer_scan_parameters.get_offset_radius());
        let mut discovered_pointer_levels = Vec::new();
        let (snapshot_region_scan_tasks, total_snapshot_region_count) =
            PointerScanTaskBuilder::build_snapshot_region_scan_tasks(snapshots, modules, pointer_scan_parameters.get_pointer_size());
        let total_snapshot_region_scan_task_count = snapshot_region_scan_tasks.len();

        for pointer_chain_depth in 0..max_depth {
            let level_number = pointer_chain_depth.saturating_add(1);
            let is_terminal_level = level_number >= max_depth;

            if frontier_target_ranges.is_empty() {
                if with_logging {
                    log::info!(
                        "Pointer scan stopped after level {} because no frontier targets remained.",
                        level_number.saturating_sub(1)
                    );
                }

                break;
            }

            let pointer_scan_execution_plan =
                PointerScanRangePass::build_execution_plan(&frontier_target_ranges, pointer_scan_parameters.get_pointer_size(), &snapshot_region_scan_tasks);
            let level_start_time = Instant::now();

            if with_logging {
                log::info!(
                    "Pointer scan level {}/{}: scanning {} snapshot regions across {} natural scan tasks for {} frontier targets merged into {} ranges with {} kernel.",
                    level_number,
                    max_depth,
                    total_snapshot_region_count,
                    total_snapshot_region_scan_task_count,
                    frontier_target_ranges.get_source_target_count(),
                    frontier_target_ranges.get_range_count(),
                    pointer_scan_execution_plan
                        .get_planned_kernel_kind()
                        .get_display_name(),
                );
            }

            let discovered_pointer_level = PointerScanRangePass::collect_candidates(
                &snapshot_region_scan_tasks,
                &frontier_target_ranges,
                &pointer_scan_execution_plan,
                true,
                !is_terminal_level,
            );
            let level_duration = level_start_time.elapsed();

            if with_logging {
                log::info!(
                    "Pointer scan level {}/{} complete in {:?}: retained {} static nodes and {} heap nodes, and produced {} next frontier targets.",
                    level_number,
                    max_depth,
                    level_duration,
                    discovered_pointer_level.static_candidates.len(),
                    discovered_pointer_level.heap_candidates.len(),
                    if is_terminal_level {
                        0
                    } else {
                        discovered_pointer_level.heap_candidates.len()
                    },
                );
            }

            if discovered_pointer_level.static_candidates.is_empty() && discovered_pointer_level.heap_candidates.is_empty() {
                if with_logging {
                    log::info!(
                        "Pointer scan stopped after level {} because no deeper pointer candidates were found.",
                        level_number
                    );
                }

                break;
            }

            if !is_terminal_level {
                frontier_target_ranges = PointerScanTargetRangeSet::from_sorted_target_addresses_iter(
                    discovered_pointer_level
                        .heap_candidates
                        .iter()
                        .map(|discovered_pointer_candidate| discovered_pointer_candidate.pointer_address),
                    pointer_scan_parameters.get_offset_radius(),
                );
            }
            discovered_pointer_levels.push(discovered_pointer_level);
        }

        if with_logging {
            let discovered_pointer_node_count = discovered_pointer_levels
                .iter()
                .map(|discovered_pointer_level| discovered_pointer_level.static_candidates.len() + discovered_pointer_level.heap_candidates.len())
                .sum::<usize>();

            log::info!(
                "Pointer scan discovered {} unique reachable pointer nodes across {} levels before any tree expansion.",
                discovered_pointer_node_count,
                discovered_pointer_levels.len(),
            );
            log::info!("Pointer scan reachability levels built in: {:?}", discovery_start_time.elapsed());
        }

        discovered_pointer_levels
    }
}
