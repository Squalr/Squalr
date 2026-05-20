use crate::element_scans::element_scan_report::ElementScanReport;
use crate::scanners::element_scan_dispatcher::ElementScanDispatcher;
use crate::scanners::scan_control::ScanControl;
use rayon::iter::{IntoParallelRefIterator, IntoParallelRefMutIterator, ParallelIterator};
use squalr_engine_api::conversions::storage_size_conversions::StorageSizeConversions;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::results::snapshot_region_scan_results::SnapshotRegionScanResults;
use squalr_engine_api::structures::scanning::plans::element_scan::element_scan_plan::ElementScanPlan;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

pub struct ElementScanner;

impl ElementScanner {
    /// Scans already-populated snapshot bytes without reading from any process or target.
    pub fn scan_snapshot(
        snapshot: &mut Snapshot,
        symbol_registry: &SymbolRegistry,
        element_scan_plan: &ElementScanPlan,
        scan_control: &ScanControl,
    ) -> ElementScanReport {
        Self::scan_snapshot_with_region_refresh(snapshot, symbol_registry, element_scan_plan, scan_control, false, |_| {})
    }

    /// Scans a snapshot and lets the caller refresh each region before it is scanned.
    pub fn scan_snapshot_with_region_refresh<RefreshRegion>(
        snapshot: &mut Snapshot,
        symbol_registry: &SymbolRegistry,
        element_scan_plan: &ElementScanPlan,
        scan_control: &ScanControl,
        with_logging: bool,
        refresh_region: RefreshRegion,
    ) -> ElementScanReport
    where
        RefreshRegion: Fn(&mut SnapshotRegion) + Sync,
    {
        let scan_start_time = Instant::now();
        let committed_deleted_result_count = snapshot.commit_deleted_scan_result_indices(symbol_registry);
        let processed_region_count = Arc::new(AtomicU64::new(0));
        let total_region_count = snapshot.get_region_count();
        let snapshot_regions = snapshot.get_snapshot_regions_mut();

        if with_logging && committed_deleted_result_count > 0 {
            log::info!("Committed {} manually deleted scan result(s) before scanning.", committed_deleted_result_count);
        }

        let scan_region = |snapshot_region: &mut SnapshotRegion| {
            if scan_control.should_cancel() {
                return;
            }

            snapshot_region.initialize_scan_results(
                symbol_registry,
                element_scan_plan.get_data_type_refs_iterator(),
                element_scan_plan.get_memory_alignment(),
            );
            refresh_region(snapshot_region);

            let element_scan_dispatcher = |snapshot_region_filter_collection| {
                ElementScanDispatcher::dispatch_scan(symbol_registry, snapshot_region, snapshot_region_filter_collection, element_scan_plan)
            };

            let scan_results_collection = snapshot_region.get_scan_results().get_filter_collections();
            let single_thread_scan = element_scan_plan.get_is_single_thread_scan() || scan_results_collection.len() == 1;
            let scan_results = SnapshotRegionScanResults::new(if single_thread_scan {
                scan_results_collection
                    .iter()
                    .map(element_scan_dispatcher)
                    .collect()
            } else {
                scan_results_collection
                    .par_iter()
                    .map(element_scan_dispatcher)
                    .collect()
            });

            snapshot_region.set_scan_results(scan_results);

            let processed_region_index = processed_region_count.fetch_add(1, Ordering::SeqCst);

            if processed_region_index % 32 == 0 && total_region_count > 0 {
                let progress = (processed_region_index as f32 / total_region_count as f32) * 100.0;
                scan_control.report_progress(progress);
            }
        };

        let single_thread_scan = element_scan_plan.get_is_single_thread_scan() || snapshot_regions.len() == 1;
        if single_thread_scan {
            snapshot_regions.iter_mut().for_each(scan_region);
        } else {
            snapshot_regions.par_iter_mut().for_each(scan_region);
        };

        snapshot.discard_empty_regions();

        let scan_duration = scan_start_time.elapsed();
        let scanned_byte_count = snapshot.get_byte_count();
        let result_count = snapshot.get_number_of_results();

        if with_logging {
            log::info!(
                "Retained scan bytes: {}",
                StorageSizeConversions::value_to_metric_size(scanned_byte_count as u128)
            );
            log::info!("Scan complete in: {:?}", scan_duration);
        }

        ElementScanReport::new(
            scanned_byte_count,
            processed_region_count.load(Ordering::SeqCst),
            result_count,
            committed_deleted_result_count,
            scan_duration,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::ElementScanner;
    use crate::scanners::scan_control::ScanControl;
    use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
    use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
    use squalr_engine_api::structures::data_types::floating_point_tolerance::FloatingPointTolerance;
    use squalr_engine_api::structures::data_values::data_value::DataValue;
    use squalr_engine_api::structures::memory::memory_alignment::MemoryAlignment;
    use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
    use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
    use squalr_engine_api::structures::scanning::constraints::scan_constraint::ScanConstraint;
    use squalr_engine_api::structures::scanning::constraints::scan_constraint_finalized::ScanConstraintFinalized;
    use squalr_engine_api::structures::scanning::memory_read_mode::MemoryReadMode;
    use squalr_engine_api::structures::scanning::plans::element_scan::element_scan_plan::ElementScanPlan;
    use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
    use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn scan_snapshot_scans_precollected_bytes_without_process_io() {
        let symbol_registry = SymbolRegistry::new();
        let data_type_ref = DataTypeRef::new("u8");
        let element_scan_plan = build_equal_u8_plan(&symbol_registry, data_type_ref.clone(), 0x7F);
        let mut snapshot = Snapshot::from_regions(vec![SnapshotRegion::from_bytes(0x1000, vec![0x00, 0x7F, 0x7F, 0x01])]);

        let scan_report = ElementScanner::scan_snapshot(&mut snapshot, &symbol_registry, &element_scan_plan, &ScanControl::default());

        assert_eq!(scan_report.get_processed_region_count(), 1);
        assert_eq!(scan_report.get_result_count(), 2);
        assert_eq!(
            snapshot.collect_scan_result_addresses_for_data_type(&symbol_registry, &data_type_ref),
            vec![0x1001, 0x1002]
        );
    }

    #[test]
    fn scan_snapshot_with_region_refresh_lets_callers_materialize_values() {
        let symbol_registry = SymbolRegistry::new();
        let data_type_ref = DataTypeRef::new("u8");
        let element_scan_plan = build_equal_u8_plan(&symbol_registry, data_type_ref.clone(), 0x42);
        let mut snapshot = Snapshot::from_regions(vec![SnapshotRegion::from_bytes(0x2000, vec![0x00, 0x00, 0x00])]);
        let refresh_count = Arc::new(AtomicUsize::new(0));
        let refresh_count_for_closure = refresh_count.clone();

        ElementScanner::scan_snapshot_with_region_refresh(
            &mut snapshot,
            &symbol_registry,
            &element_scan_plan,
            &ScanControl::default(),
            false,
            move |snapshot_region| {
                refresh_count_for_closure.fetch_add(1, Ordering::SeqCst);
                snapshot_region.current_values = vec![0x42, 0x00, 0x42];
            },
        );

        assert_eq!(refresh_count.load(Ordering::SeqCst), 1);
        assert_eq!(
            snapshot.collect_scan_result_addresses_for_data_type(&symbol_registry, &data_type_ref),
            vec![0x2000, 0x2002]
        );
    }

    fn build_equal_u8_plan(
        symbol_registry: &SymbolRegistry,
        data_type_ref: DataTypeRef,
        expected_value: u8,
    ) -> ElementScanPlan {
        let scan_constraint = ScanConstraint::new(
            ScanCompareType::Immediate(ScanCompareTypeImmediate::Equal),
            DataValue::new(data_type_ref.clone(), vec![expected_value]),
            FloatingPointTolerance::default(),
        );
        let scan_constraint_finalized = ScanConstraintFinalized::new(symbol_registry, scan_constraint);
        let mut scan_constraints_by_data_type = HashMap::new();

        scan_constraints_by_data_type.insert(data_type_ref, vec![scan_constraint_finalized]);

        ElementScanPlan::new(
            scan_constraints_by_data_type,
            MemoryAlignment::Alignment1,
            FloatingPointTolerance::default(),
            MemoryReadMode::Skip,
            true,
            false,
        )
    }
}
