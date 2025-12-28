use crate::{
    registries::symbols::symbol_registry::SymbolRegistry,
    structures::{
        scanning::{
            constraints::scan_constraint::ScanConstraint,
            filters::{snapshot_region_filter::SnapshotRegionFilter, snapshot_region_filter_collection::SnapshotRegionFilterCollection},
            plans::element_scan::snapshot_filter_element_scan_plan::SnapshotFilterElementScanPlan,
        },
        snapshots::snapshot_region::SnapshotRegion,
    },
};
use std::sync::{Arc, RwLock};

pub trait ElementScanFilterRule: Send + Sync {
    fn get_id(&self) -> &str;
    fn map_parameters(
        &self,
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_constraint: &ScanConstraint,
        snapshot_filter_element_scan_plan: &mut SnapshotFilterElementScanPlan,
    );
}
