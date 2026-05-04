use crate::pointer_scans::structures::snapshot_region_scan_task::SnapshotRegionScanTask;
use crate::pointer_scans::structures::snapshot_region_scan_task_kind::SnapshotRegionScanTaskKind;
use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;

pub(crate) struct PointerScanTaskBuilder;

impl PointerScanTaskBuilder {
    pub(crate) fn build_snapshot_region_scan_tasks<'a>(
        snapshots: &[&'a Snapshot],
        modules: &[NormalizedModule],
        pointer_size: PointerScanPointerSize,
    ) -> (Vec<SnapshotRegionScanTask<'a>>, usize) {
        let total_snapshot_region_count = snapshots
            .iter()
            .map(|snapshot| snapshot.get_snapshot_regions().len())
            .sum::<usize>();
        let snapshot_region_scan_tasks = Self::build_scan_tasks_from_snapshot_regions(
            snapshots
                .iter()
                .flat_map(|snapshot| snapshot.get_snapshot_regions().iter()),
            modules,
            pointer_size,
            true,
        );

        (snapshot_region_scan_tasks, total_snapshot_region_count)
    }

    pub(crate) fn build_heap_scan_tasks<'a>(
        snapshot_regions: &'a [SnapshotRegion],
        modules: &[NormalizedModule],
        pointer_size: PointerScanPointerSize,
    ) -> Vec<SnapshotRegionScanTask<'a>> {
        Self::build_scan_tasks_from_snapshot_regions(snapshot_regions.iter(), modules, pointer_size, false)
    }

    fn build_scan_tasks_from_snapshot_regions<'a, SnapshotRegions>(
        snapshot_regions: SnapshotRegions,
        modules: &[NormalizedModule],
        pointer_size: PointerScanPointerSize,
        include_static_ranges: bool,
    ) -> Vec<SnapshotRegionScanTask<'a>>
    where
        SnapshotRegions: Iterator<Item = &'a SnapshotRegion>,
    {
        let pointer_size_in_bytes = pointer_size.get_size_in_bytes() as usize;
        let mut sorted_modules = modules.iter().enumerate().collect::<Vec<_>>();
        sorted_modules.sort_unstable_by_key(|(_module_index, module)| module.get_base_address());
        let mut snapshot_region_scan_tasks = Vec::new();

        for snapshot_region in snapshot_regions {
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
                    Self::append_snapshot_region_scan_task_for_range(
                        snapshot_region,
                        uncovered_range_base_address,
                        module_base_address.min(snapshot_region_end_address),
                        pointer_size_in_bytes,
                        SnapshotRegionScanTaskKind::Heap,
                        &mut snapshot_region_scan_tasks,
                    );
                }

                if include_static_ranges {
                    let static_range_base_address = uncovered_range_base_address.max(module_base_address);
                    let static_range_end_address = snapshot_region_end_address.min(module_end_address);

                    if static_range_base_address < static_range_end_address {
                        Self::append_snapshot_region_scan_task_for_range(
                            snapshot_region,
                            static_range_base_address,
                            static_range_end_address,
                            pointer_size_in_bytes,
                            SnapshotRegionScanTaskKind::Static {
                                module_index: *module_index,
                                module_base_address,
                            },
                            &mut snapshot_region_scan_tasks,
                        );
                    }
                }

                uncovered_range_base_address = uncovered_range_base_address.max(module_end_address);

                if uncovered_range_base_address >= snapshot_region_end_address {
                    break;
                }
            }

            if uncovered_range_base_address < snapshot_region_end_address {
                Self::append_snapshot_region_scan_task_for_range(
                    snapshot_region,
                    uncovered_range_base_address,
                    snapshot_region_end_address,
                    pointer_size_in_bytes,
                    SnapshotRegionScanTaskKind::Heap,
                    &mut snapshot_region_scan_tasks,
                );
            }
        }

        snapshot_region_scan_tasks
    }

    fn append_snapshot_region_scan_task_for_range<'a>(
        snapshot_region: &'a SnapshotRegion,
        range_base_address: u64,
        range_end_address: u64,
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
        let task_read_end_offset = range_end_offset
            .saturating_add(pointer_size_in_bytes.saturating_sub(1))
            .min(current_values.len());

        snapshot_region_scan_tasks.push(SnapshotRegionScanTask {
            scan_base_address: range_base_address,
            scan_end_address: range_end_address,
            current_values: &current_values[range_start_offset..task_read_end_offset],
            task_kind,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::PointerScanTaskBuilder;
    use crate::pointer_scans::structures::snapshot_region_scan_task_kind::SnapshotRegionScanTaskKind;
    use squalr_engine_api::structures::memory::normalized_module::NormalizedModule;
    use squalr_engine_api::structures::memory::normalized_region::NormalizedRegion;
    use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;
    use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
    use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;

    #[test]
    fn build_snapshot_region_scan_tasks_keeps_large_regions_as_single_natural_tasks() {
        let mut snapshot = Snapshot::new();
        let mut snapshot_region = SnapshotRegion::new(NormalizedRegion::new(0x1003, (2 * 1024 * 1024 + 16) as u64), Vec::new());
        snapshot_region.current_values = vec![0_u8; 2 * 1024 * 1024 + 16];
        snapshot.set_snapshot_regions(vec![snapshot_region]);

        let (snapshot_region_scan_tasks, total_snapshot_region_count) =
            PointerScanTaskBuilder::build_snapshot_region_scan_tasks(&[&snapshot], &[], PointerScanPointerSize::Pointer64);

        assert_eq!(total_snapshot_region_count, 1);
        assert_eq!(snapshot_region_scan_tasks.len(), 1);
        assert_eq!(snapshot_region_scan_tasks[0].scan_base_address, 0x1003);
        assert_eq!(snapshot_region_scan_tasks[0].current_values.len(), 2 * 1024 * 1024 + 16);
    }

    #[test]
    fn build_snapshot_region_scan_tasks_splits_static_and_heap_ranges_without_losing_boundary_reads() {
        let mut snapshot = Snapshot::new();
        let mut snapshot_region = SnapshotRegion::new(NormalizedRegion::new(0x1000, 0x40), Vec::new());
        snapshot_region.current_values = vec![0_u8; 0x40];
        snapshot.set_snapshot_regions(vec![snapshot_region]);
        let modules = [NormalizedModule::new("game.exe", 0x1010, 0x10)];

        let (snapshot_region_scan_tasks, _total_snapshot_region_count) =
            PointerScanTaskBuilder::build_snapshot_region_scan_tasks(&[&snapshot], &modules, PointerScanPointerSize::Pointer64);

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
