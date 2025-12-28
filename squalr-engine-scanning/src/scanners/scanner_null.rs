use crate::scanners::snapshot_scanner::Scanner;
use squalr_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::{
    registries::symbols::symbol_registry::SymbolRegistry,
    structures::scanning::plans::element_scan::snapshot_filter_element_scan_plan::SnapshotFilterElementScanPlan,
};
use std::sync::{Arc, RwLock};

/// A scanner that does nothing.
pub struct ScannerNull {}

/// Implements a scanner that does nothing.
impl Scanner for ScannerNull {
    fn get_scanner_name(&self) -> &'static str {
        &"Null Scanner"
    }

    fn scan_region(
        &self,
        _symbol_registry: &Arc<RwLock<SymbolRegistry>>,
        _snapshot_region: &SnapshotRegion,
        _snapshot_region_filter: &SnapshotRegionFilter,
        _snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Vec<SnapshotRegionFilter> {
        Vec::new()
    }
}
