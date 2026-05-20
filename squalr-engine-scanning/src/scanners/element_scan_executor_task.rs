use crate::element_scans::element_scanner::ElementScanner;
use crate::scanners::scan_execution_context::ScanExecutionContext;
use crate::scanners::snapshot_region_memory_reader::SnapshotRegionMemoryReader;
use crate::scanners::value_collector_task::ValueCollector;
use squalr_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use squalr_engine_api::structures::processes::opened_process_info::OpenedProcessInfo;
use squalr_engine_api::structures::scanning::memory_read_mode::MemoryReadMode;
use squalr_engine_api::structures::scanning::plans::element_scan::element_scan_plan::ElementScanPlan;
use squalr_engine_api::structures::snapshots::snapshot::Snapshot;
use std::sync::{Arc, RwLock};
use std::time::Instant;

pub struct ElementScanExecutor;

/// Implementation of a task that performs a scan against the provided snapshot. Does not collect new values.
/// Caller is assumed to have already done this if desired.
impl ElementScanExecutor {
    pub fn execute_scan(
        process_info: OpenedProcessInfo,
        snapshot: Arc<RwLock<Snapshot>>,
        symbol_registry: &SymbolRegistry,
        element_scan_plan: ElementScanPlan,
        with_logging: bool,
        scan_execution_context: &ScanExecutionContext,
    ) {
        Self::scan_task(process_info, snapshot, symbol_registry, element_scan_plan, with_logging, scan_execution_context);
    }

    fn scan_task(
        process_info: OpenedProcessInfo,
        snapshot: Arc<RwLock<Snapshot>>,
        symbol_registry: &SymbolRegistry,
        element_scan_plan: ElementScanPlan,
        with_logging: bool,
        scan_execution_context: &ScanExecutionContext,
    ) {
        let total_start_time = Instant::now();

        // If the parameter is set, first collect values before the scan.
        // This is slower overall than interleaving the reads, but better for capturing values that may soon change.
        if element_scan_plan.get_memory_read_mode() == MemoryReadMode::ReadBeforeScan {
            ValueCollector::collect_values(process_info.clone(), snapshot.clone(), with_logging, scan_execution_context);
        }

        if with_logging {
            log::info!("Performing manual scan...");
        }

        let mut snapshot = match snapshot.write() {
            Ok(guard) => guard,
            Err(error) => {
                if with_logging {
                    log::error!("Failed to acquire write lock on snapshot: {}", error);
                }

                return;
            }
        };

        ElementScanner::scan_snapshot_with_region_refresh(
            &mut snapshot,
            symbol_registry,
            &element_scan_plan,
            scan_execution_context.as_scan_control(),
            with_logging,
            |snapshot_region| {
                // Attempt to read new values before scanning this region. Ignore failures as they usually indicate deallocated pages.
                if element_scan_plan.get_memory_read_mode() == MemoryReadMode::ReadInterleavedWithScan {
                    let _ = snapshot_region.read_all_memory(&process_info, scan_execution_context);
                }
            },
        );

        if with_logging {
            let total_duration = total_start_time.elapsed();

            log::info!("Total scan time: {:?}", total_duration);
        }
    }
}
