use crate::pointer_scans::pointer_scan_dispatcher::PointerScanDispatcher;
use crate::pointer_scans::search_kernels::pointer_scan_search_kernel::PointerScanSearchKernel;
use crate::pointer_scans::search_kernels::pointer_scan_search_kernel_context::PointerScanSearchKernelContext;
use crate::pointer_scans::structures::pointer_scan_collected_candidate::PointerScanCollectedCandidate;
use crate::pointer_scans::structures::pointer_scan_collected_level::PointerScanCollectedLevel;
use crate::pointer_scans::structures::pointer_scan_target_ranges::PointerScanTargetRangeSet;
use crate::pointer_scans::structures::snapshot_region_scan_task::SnapshotRegionScanTask;
use crate::pointer_scans::structures::snapshot_region_scan_task_kind::SnapshotRegionScanTaskKind;
use rayon::prelude::*;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::scanning::plans::pointer_scan::pointer_scan_execution_plan::PointerScanExecutionPlan;

pub(crate) struct PointerScanCandidateCollector;

impl PointerScanCandidateCollector {
    pub(crate) fn build_execution_plan(
        frontier_target_ranges: &PointerScanTargetRangeSet,
        pointer_size: PointerScanPointerSize,
        snapshot_region_scan_tasks: &[SnapshotRegionScanTask<'_>],
    ) -> PointerScanExecutionPlan {
        let scan_region_byte_count = snapshot_region_scan_tasks
            .iter()
            .map(|snapshot_region_scan_task| snapshot_region_scan_task.current_values.len())
            .max()
            .unwrap_or(0);

        PointerScanDispatcher::build_execution_plan(frontier_target_ranges, pointer_size, scan_region_byte_count)
    }

    pub(crate) fn collect_candidates(
        snapshot_region_scan_tasks: &[SnapshotRegionScanTask<'_>],
        frontier_target_ranges: &PointerScanTargetRangeSet,
        pointer_scan_execution_plan: &PointerScanExecutionPlan,
        retain_static_candidates: bool,
        retain_heap_candidates: bool,
    ) -> PointerScanCollectedLevel {
        let range_search_kernel = PointerScanDispatcher::acquire_range_search_kernel(frontier_target_ranges, pointer_scan_execution_plan);
        let pointer_scan_search_kernel_context = PointerScanSearchKernelContext::new(frontier_target_ranges, pointer_scan_execution_plan.get_pointer_size());

        Self::collect_candidates_with_kernel(
            snapshot_region_scan_tasks,
            range_search_kernel,
            &pointer_scan_search_kernel_context,
            retain_static_candidates,
            retain_heap_candidates,
        )
    }

    fn collect_candidates_with_kernel(
        snapshot_region_scan_tasks: &[SnapshotRegionScanTask<'_>],
        range_search_kernel: &dyn PointerScanSearchKernel,
        pointer_scan_search_kernel_context: &PointerScanSearchKernelContext<'_>,
        retain_static_candidates: bool,
        retain_heap_candidates: bool,
    ) -> PointerScanCollectedLevel {
        if range_search_kernel.is_empty(pointer_scan_search_kernel_context) {
            return PointerScanCollectedLevel::default();
        }

        let collected_levels_by_task = snapshot_region_scan_tasks
            .par_iter()
            .map(|snapshot_region_scan_task| {
                let mut collected_level = PointerScanCollectedLevel::default();

                Self::collect_task_matches(
                    snapshot_region_scan_task,
                    range_search_kernel,
                    pointer_scan_search_kernel_context,
                    retain_static_candidates,
                    retain_heap_candidates,
                    &mut collected_level,
                );

                collected_level
            })
            .collect::<Vec<_>>();
        let total_static_candidate_count = collected_levels_by_task
            .iter()
            .map(|collected_level| collected_level.static_candidates.len())
            .sum();
        let total_heap_candidate_count = collected_levels_by_task
            .iter()
            .map(|collected_level| collected_level.heap_candidates.len())
            .sum();
        let mut merged_collected_level = PointerScanCollectedLevel {
            static_candidates: Vec::with_capacity(total_static_candidate_count),
            heap_candidates: Vec::with_capacity(total_heap_candidate_count),
        };

        for mut collected_level in collected_levels_by_task {
            merged_collected_level
                .static_candidates
                .append(&mut collected_level.static_candidates);
            merged_collected_level
                .heap_candidates
                .append(&mut collected_level.heap_candidates);
        }

        merged_collected_level
    }

    fn collect_task_matches(
        snapshot_region_scan_task: &SnapshotRegionScanTask<'_>,
        range_search_kernel: &dyn PointerScanSearchKernel,
        pointer_scan_search_kernel_context: &PointerScanSearchKernelContext<'_>,
        retain_static_candidates: bool,
        retain_heap_candidates: bool,
        collected_level: &mut PointerScanCollectedLevel,
    ) {
        match snapshot_region_scan_task.task_kind {
            SnapshotRegionScanTaskKind::Heap if !retain_heap_candidates => return,
            SnapshotRegionScanTaskKind::Static { .. } if !retain_static_candidates => return,
            _ => {}
        }
        range_search_kernel.scan_region_with_visitor(
            pointer_scan_search_kernel_context,
            snapshot_region_scan_task.scan_base_address,
            snapshot_region_scan_task.current_values,
            &mut |pointer_match| {
                let pointer_address = pointer_match.get_pointer_address();

                if pointer_address >= snapshot_region_scan_task.scan_end_address {
                    return;
                }

                match snapshot_region_scan_task.task_kind {
                    SnapshotRegionScanTaskKind::Static {
                        module_index,
                        module_base_address,
                    } => {
                        collected_level
                            .static_candidates
                            .push(PointerScanCollectedCandidate {
                                pointer_address,
                                pointer_value: pointer_match.get_pointer_value(),
                                module_index,
                                module_offset: pointer_address.saturating_sub(module_base_address),
                            });
                    }
                    SnapshotRegionScanTaskKind::Heap => {
                        collected_level
                            .heap_candidates
                            .push(PointerScanCollectedCandidate {
                                pointer_address,
                                pointer_value: pointer_match.get_pointer_value(),
                                module_index: 0,
                                module_offset: 0,
                            });
                    }
                }
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pointer_scans::structures::pointer_scan_region_match::PointerScanRegionMatch;
    use std::sync::Mutex;

    #[derive(Default)]
    struct RecordingKernel(Mutex<Vec<u64>>);

    impl PointerScanSearchKernel for RecordingKernel {
        fn scan_region_with_visitor(
            &self,
            _context: &PointerScanSearchKernelContext<'_>,
            base_address: u64,
            _values: &[u8],
            _visit: &mut dyn FnMut(PointerScanRegionMatch),
        ) {
            self.0.lock().expect("Recorded calls").push(base_address);
        }
    }

    #[test]
    fn final_discovery_level_only_invokes_kernel_for_static_tasks() {
        let ranges = PointerScanTargetRangeSet::from_target_addresses(&[0x3000], 0x20);
        let context = PointerScanSearchKernelContext::new(&ranges, PointerScanPointerSize::Pointer64);
        let kernel = RecordingKernel::default();
        let tasks = [
            SnapshotRegionScanTask {
                scan_base_address: 0x1000,
                scan_end_address: 0x1008,
                current_values: &[0; 8],
                task_kind: SnapshotRegionScanTaskKind::Static {
                    module_index: 0,
                    module_base_address: 0x1000,
                },
            },
            SnapshotRegionScanTask {
                scan_base_address: 0x2000,
                scan_end_address: 0x2008,
                current_values: &[0; 8],
                task_kind: SnapshotRegionScanTaskKind::Heap,
            },
        ];
        PointerScanCandidateCollector::collect_candidates_with_kernel(&tasks, &kernel, &context, true, false);
        assert_eq!(*kernel.0.lock().expect("Recorded calls"), vec![0x1000]);
    }
}
