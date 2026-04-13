use crate::pointer_scans::pointer_scan_dispatcher::PointerScanDispatcher;
use crate::pointer_scans::search_kernels::PointerScanRangeSearchKernel;
use crate::pointer_scans::structures::discovered_pointer_candidate::DiscoveredPointerCandidate;
use crate::pointer_scans::structures::discovered_pointer_level::DiscoveredPointerLevel;
use crate::pointer_scans::structures::pointer_scan_target_ranges::PointerScanTargetRangeSet;
use crate::pointer_scans::structures::snapshot_region_scan_task::SnapshotRegionScanTask;
use crate::pointer_scans::structures::snapshot_region_scan_task_kind::SnapshotRegionScanTaskKind;
use rayon::prelude::*;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::scanning::plans::pointer_scan::pointer_scan_parameters::PointerScanParameters;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use std::time::Instant;

const POINTER_SCAN_MIN_SNAPSHOT_TASK_BYTE_SIZE: usize = 1024 * 1024;
const POINTER_SCAN_TARGET_TASKS_PER_WORKER: usize = 4;

pub(crate) struct PointerScanLevelCollector;

impl PointerScanLevelCollector {
    pub(crate) fn discover_pointer_levels(
        snapshots: &[&Snapshot],
        target_addresses: &[u64],
        pointer_scan_parameters: &PointerScanParameters,
        modules: &[NormalizedModule],
        with_logging: bool,
    ) -> Vec<DiscoveredPointerLevel> {
        let discovery_start_time = Instant::now();
        let max_depth = pointer_scan_parameters.get_max_depth();

        if max_depth == 0 {
            return Vec::new();
        }

        let mut frontier_target_ranges = PointerScanTargetRangeSet::from_target_addresses(target_addresses, pointer_scan_parameters.get_offset_radius());
        let mut discovered_pointer_levels = Vec::new();
        let (snapshot_region_scan_tasks, total_snapshot_region_count, snapshot_task_byte_size) =
            Self::build_snapshot_region_scan_tasks(snapshots, modules, pointer_scan_parameters.get_pointer_size());
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
                PointerScanDispatcher::build_execution_plan(&frontier_target_ranges, pointer_scan_parameters.get_pointer_size(), snapshot_task_byte_size);
            let range_search_kernel = PointerScanDispatcher::acquire_range_search_kernel(&frontier_target_ranges, &pointer_scan_execution_plan);
            let level_start_time = Instant::now();

            if with_logging {
                log::info!(
                    "Pointer scan level {}/{}: scanning {} snapshot regions across {} scan tasks ({} bytes/task) for {} frontier targets merged into {} ranges with {} kernel.",
                    level_number,
                    max_depth,
                    total_snapshot_region_count,
                    total_snapshot_region_scan_task_count,
                    snapshot_task_byte_size,
                    frontier_target_ranges.get_source_target_count(),
                    frontier_target_ranges.get_range_count(),
                    pointer_scan_execution_plan
                        .get_planned_kernel_kind()
                        .get_display_name(),
                );
            }

            let discovered_pointer_level = Self::collect_level(&snapshot_region_scan_tasks, &range_search_kernel, !is_terminal_level);
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

    fn collect_level(
        snapshot_region_scan_tasks: &[SnapshotRegionScanTask<'_>],
        range_search_kernel: &PointerScanRangeSearchKernel<'_>,
        retain_heap_candidates: bool,
    ) -> DiscoveredPointerLevel {
        if range_search_kernel.is_empty() {
            return DiscoveredPointerLevel::default();
        }

        let discovered_pointer_levels_by_task = snapshot_region_scan_tasks
            .par_iter()
            .map(|snapshot_region_scan_task| {
                let mut discovered_pointer_level = DiscoveredPointerLevel::default();

                Self::collect_task_matches(
                    snapshot_region_scan_task,
                    range_search_kernel,
                    retain_heap_candidates,
                    &mut discovered_pointer_level,
                );

                discovered_pointer_level
            })
            .collect::<Vec<_>>();
        let total_static_candidate_count = discovered_pointer_levels_by_task
            .iter()
            .map(|discovered_pointer_level| discovered_pointer_level.static_candidates.len())
            .sum();
        let total_heap_candidate_count = discovered_pointer_levels_by_task
            .iter()
            .map(|discovered_pointer_level| discovered_pointer_level.heap_candidates.len())
            .sum();
        let mut merged_discovered_pointer_level = DiscoveredPointerLevel {
            static_candidates: Vec::with_capacity(total_static_candidate_count),
            heap_candidates: Vec::with_capacity(total_heap_candidate_count),
        };

        for mut discovered_pointer_level in discovered_pointer_levels_by_task {
            merged_discovered_pointer_level
                .static_candidates
                .append(&mut discovered_pointer_level.static_candidates);
            merged_discovered_pointer_level
                .heap_candidates
                .append(&mut discovered_pointer_level.heap_candidates);
        }

        merged_discovered_pointer_level
    }

    fn collect_task_matches(
        snapshot_region_scan_task: &SnapshotRegionScanTask<'_>,
        range_search_kernel: &PointerScanRangeSearchKernel<'_>,
        retain_heap_candidates: bool,
        discovered_pointer_level: &mut DiscoveredPointerLevel,
    ) {
        range_search_kernel.scan_region_with_visitor(
            snapshot_region_scan_task.scan_base_address,
            snapshot_region_scan_task.current_values,
            |pointer_match| {
                let pointer_address = pointer_match.get_pointer_address();

                if pointer_address >= snapshot_region_scan_task.scan_end_address {
                    return;
                }

                match snapshot_region_scan_task.task_kind {
                    SnapshotRegionScanTaskKind::Static {
                        module_index,
                        module_base_address,
                    } => {
                        discovered_pointer_level
                            .static_candidates
                            .push(DiscoveredPointerCandidate {
                                pointer_address,
                                pointer_value: pointer_match.get_pointer_value(),
                                module_index,
                                module_offset: pointer_address.saturating_sub(module_base_address),
                            });
                    }
                    SnapshotRegionScanTaskKind::Heap if retain_heap_candidates => {
                        discovered_pointer_level
                            .heap_candidates
                            .push(DiscoveredPointerCandidate {
                                pointer_address,
                                pointer_value: pointer_match.get_pointer_value(),
                                module_index: 0,
                                module_offset: 0,
                            });
                    }
                    SnapshotRegionScanTaskKind::Heap => {}
                }
            },
        );
    }

    pub(crate) fn build_snapshot_region_scan_tasks<'a>(
        snapshots: &[&'a Snapshot],
        modules: &[NormalizedModule],
        pointer_size: PointerScanPointerSize,
    ) -> (Vec<SnapshotRegionScanTask<'a>>, usize, usize) {
        let pointer_size_in_bytes = pointer_size.get_size_in_bytes() as usize;
        let mut sorted_modules = modules.iter().enumerate().collect::<Vec<_>>();
        sorted_modules.sort_unstable_by_key(|(_module_index, module)| module.get_base_address());
        let total_snapshot_region_count = snapshots
            .iter()
            .map(|snapshot| snapshot.get_snapshot_regions().len())
            .sum::<usize>();
        let total_snapshot_byte_count = snapshots
            .iter()
            .flat_map(|snapshot| snapshot.get_snapshot_regions().iter())
            .map(|snapshot_region| snapshot_region.get_current_values().len())
            .sum::<usize>();
        let task_byte_size = Self::calculate_snapshot_task_byte_size(total_snapshot_byte_count, total_snapshot_region_count, pointer_size_in_bytes);
        let estimated_task_count = snapshots
            .iter()
            .flat_map(|snapshot| snapshot.get_snapshot_regions().iter())
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

        for snapshot in snapshots {
            for snapshot_region in snapshot.get_snapshot_regions() {
                if snapshot_region.get_current_values().is_empty() {
                    continue;
                }

                let mut uncovered_range_base_address = snapshot_region.get_base_address();
                let snapshot_region_end_address = snapshot_region.get_end_address();

                for (module_index, module) in &sorted_modules {
                    let module_base_address = module.get_base_address();
                    let module_end_address = module_base_address.saturating_add(module.get_region_size());

                    if module_end_address <= uncovered_range_base_address {
                        continue;
                    }

                    if module_base_address >= snapshot_region_end_address {
                        break;
                    }

                    if uncovered_range_base_address < module_base_address {
                        Self::append_snapshot_region_scan_tasks_for_range(
                            snapshot_region,
                            uncovered_range_base_address,
                            module_base_address.min(snapshot_region_end_address),
                            task_byte_size,
                            pointer_size_in_bytes,
                            SnapshotRegionScanTaskKind::Heap,
                            &mut snapshot_region_scan_tasks,
                        );
                    }

                    let static_range_base_address = uncovered_range_base_address.max(module_base_address);
                    let static_range_end_address = snapshot_region_end_address.min(module_end_address);

                    if static_range_base_address < static_range_end_address {
                        Self::append_snapshot_region_scan_tasks_for_range(
                            snapshot_region,
                            static_range_base_address,
                            static_range_end_address,
                            task_byte_size,
                            pointer_size_in_bytes,
                            SnapshotRegionScanTaskKind::Static {
                                module_index: *module_index,
                                module_base_address,
                            },
                            &mut snapshot_region_scan_tasks,
                        );
                    }

                    uncovered_range_base_address = uncovered_range_base_address.max(module_end_address);

                    if uncovered_range_base_address >= snapshot_region_end_address {
                        break;
                    }
                }

                if uncovered_range_base_address < snapshot_region_end_address {
                    Self::append_snapshot_region_scan_tasks_for_range(
                        snapshot_region,
                        uncovered_range_base_address,
                        snapshot_region_end_address,
                        task_byte_size,
                        pointer_size_in_bytes,
                        SnapshotRegionScanTaskKind::Heap,
                        &mut snapshot_region_scan_tasks,
                    );
                }
            }
        }

        (snapshot_region_scan_tasks, total_snapshot_region_count, task_byte_size)
    }

    fn append_snapshot_region_scan_tasks_for_range<'a>(
        snapshot_region: &'a SnapshotRegion,
        range_base_address: u64,
        range_end_address: u64,
        task_byte_size: usize,
        pointer_size_in_bytes: usize,
        task_kind: SnapshotRegionScanTaskKind,
        snapshot_region_scan_tasks: &mut Vec<SnapshotRegionScanTask<'a>>,
    ) {
        if range_end_address <= range_base_address {
            return;
        }

        let range_start_offset = range_base_address.saturating_sub(snapshot_region.get_base_address()) as usize;
        let range_end_offset = range_end_address.saturating_sub(snapshot_region.get_base_address()) as usize;
        let current_values = snapshot_region.get_current_values().as_slice();
        let mut task_start_offset = range_start_offset;

        while task_start_offset < range_end_offset {
            let remaining_logical_byte_count = range_end_offset.saturating_sub(task_start_offset);
            let task_logical_byte_count = remaining_logical_byte_count.min(task_byte_size.max(pointer_size_in_bytes));
            let task_end_offset = task_start_offset.saturating_add(task_logical_byte_count);
            let task_read_end_offset = task_end_offset
                .saturating_add(pointer_size_in_bytes.saturating_sub(1))
                .min(current_values.len());

            snapshot_region_scan_tasks.push(SnapshotRegionScanTask {
                scan_base_address: snapshot_region
                    .get_base_address()
                    .saturating_add(task_start_offset as u64),
                scan_end_address: snapshot_region
                    .get_base_address()
                    .saturating_add(task_end_offset as u64),
                current_values: &current_values[task_start_offset..task_read_end_offset],
                task_kind,
            });

            task_start_offset = task_end_offset;
        }
    }

    fn calculate_snapshot_task_byte_size(
        total_snapshot_byte_count: usize,
        total_snapshot_region_count: usize,
        pointer_size_in_bytes: usize,
    ) -> usize {
        Self::calculate_snapshot_task_byte_size_for_worker_count(
            total_snapshot_byte_count,
            total_snapshot_region_count,
            pointer_size_in_bytes,
            rayon::current_num_threads(),
        )
    }

    pub(crate) fn calculate_snapshot_task_byte_size_for_worker_count(
        total_snapshot_byte_count: usize,
        total_snapshot_region_count: usize,
        pointer_size_in_bytes: usize,
        worker_count: usize,
    ) -> usize {
        let pointer_alignment = pointer_size_in_bytes.max(1);
        let minimum_task_byte_size = POINTER_SCAN_MIN_SNAPSHOT_TASK_BYTE_SIZE.max(pointer_alignment);
        let target_task_count = total_snapshot_region_count
            .max(
                worker_count
                    .max(1)
                    .saturating_mul(POINTER_SCAN_TARGET_TASKS_PER_WORKER),
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
}

#[cfg(test)]
mod tests {
    use super::PointerScanLevelCollector;
    use crate::pointer_scans::structures::snapshot_region_scan_task_kind::SnapshotRegionScanTaskKind;
    use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
    use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
    use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
    use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;

    #[test]
    fn build_snapshot_region_scan_tasks_splits_large_regions() {
        let mut snapshot = Snapshot::new();
        let mut snapshot_region = SnapshotRegion::new(
            NormalizedRegion::new(0x1003, (super::POINTER_SCAN_MIN_SNAPSHOT_TASK_BYTE_SIZE * 2 + 16) as u64),
            Vec::new(),
        );
        snapshot_region.current_values = vec![0_u8; super::POINTER_SCAN_MIN_SNAPSHOT_TASK_BYTE_SIZE * 2 + 16];
        snapshot.set_snapshot_regions(vec![snapshot_region]);

        let (snapshot_region_scan_tasks, total_snapshot_region_count, task_byte_size) =
            PointerScanLevelCollector::build_snapshot_region_scan_tasks(&[&snapshot], &[], PointerScanPointerSize::Pointer64);

        assert_eq!(total_snapshot_region_count, 1);
        assert_eq!(task_byte_size, super::POINTER_SCAN_MIN_SNAPSHOT_TASK_BYTE_SIZE);
        assert_eq!(snapshot_region_scan_tasks.len(), 3);
        assert_eq!(snapshot_region_scan_tasks[0].scan_base_address, 0x1003);
        assert_eq!(
            snapshot_region_scan_tasks[0].current_values.len(),
            super::POINTER_SCAN_MIN_SNAPSHOT_TASK_BYTE_SIZE + 7
        );
        assert_eq!(
            snapshot_region_scan_tasks[1].scan_base_address,
            0x1003_u64.saturating_add(super::POINTER_SCAN_MIN_SNAPSHOT_TASK_BYTE_SIZE as u64)
        );
        assert_eq!(
            snapshot_region_scan_tasks[1].current_values.len(),
            super::POINTER_SCAN_MIN_SNAPSHOT_TASK_BYTE_SIZE + 7
        );
        assert_eq!(
            snapshot_region_scan_tasks[2].scan_base_address,
            0x1003_u64.saturating_add((super::POINTER_SCAN_MIN_SNAPSHOT_TASK_BYTE_SIZE * 2) as u64)
        );
        assert_eq!(snapshot_region_scan_tasks[2].current_values.len(), 16);
    }

    #[test]
    fn calculate_snapshot_task_byte_size_grows_with_total_snapshot_size() {
        let task_byte_size = PointerScanLevelCollector::calculate_snapshot_task_byte_size_for_worker_count(64 * 1024 * 1024, 1, 8, 8);

        assert_eq!(task_byte_size, 2 * 1024 * 1024);
    }

    #[test]
    fn build_snapshot_region_scan_tasks_splits_static_and_heap_ranges_without_losing_boundary_reads() {
        let mut snapshot = Snapshot::new();
        let mut snapshot_region = SnapshotRegion::new(NormalizedRegion::new(0x1000, 0x40), Vec::new());
        snapshot_region.current_values = vec![0_u8; 0x40];
        snapshot.set_snapshot_regions(vec![snapshot_region]);
        let modules = [NormalizedModule::new("game.exe", 0x1010, 0x10)];

        let (snapshot_region_scan_tasks, _total_snapshot_region_count, _task_byte_size) =
            PointerScanLevelCollector::build_snapshot_region_scan_tasks(&[&snapshot], &modules, PointerScanPointerSize::Pointer64);

        assert_eq!(snapshot_region_scan_tasks.len(), 3);
        assert!(matches!(snapshot_region_scan_tasks[0].task_kind, SnapshotRegionScanTaskKind::Heap));
        assert_eq!(snapshot_region_scan_tasks[0].scan_base_address, 0x1000);
        assert_eq!(snapshot_region_scan_tasks[0].scan_end_address, 0x1010);
        assert_eq!(snapshot_region_scan_tasks[0].current_values.len(), 0x17);

        assert!(matches!(
            snapshot_region_scan_tasks[1].task_kind,
            SnapshotRegionScanTaskKind::Static {
                module_index: 0,
                module_base_address: 0x1010
            }
        ));
        assert_eq!(snapshot_region_scan_tasks[1].scan_base_address, 0x1010);
        assert_eq!(snapshot_region_scan_tasks[1].scan_end_address, 0x1020);
        assert_eq!(snapshot_region_scan_tasks[1].current_values.len(), 0x17);

        assert!(matches!(snapshot_region_scan_tasks[2].task_kind, SnapshotRegionScanTaskKind::Heap));
        assert_eq!(snapshot_region_scan_tasks[2].scan_base_address, 0x1020);
        assert_eq!(snapshot_region_scan_tasks[2].scan_end_address, 0x1040);
        assert_eq!(snapshot_region_scan_tasks[2].current_values.len(), 0x20);
    }
}
