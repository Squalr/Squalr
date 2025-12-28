use squalr_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::{
    registries::symbols::symbol_registry::SymbolRegistry,
    structures::scanning::plans::element_scan::snapshot_filter_element_scan_plan::SnapshotFilterElementScanPlan,
};
use std::sync::{Arc, RwLock};

pub trait Scanner: Send + Sync {
    fn get_scanner_name(&self) -> &'static str;
    fn scan_region(
        &self,
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Vec<SnapshotRegionFilter>;
}
