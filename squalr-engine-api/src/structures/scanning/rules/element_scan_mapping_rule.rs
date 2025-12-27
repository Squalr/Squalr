use crate::{
    registries::symbols::symbol_registry::SymbolRegistry,
    structures::{
        scanning::{
            constraints::optimized_scan_constraint::OptimizedScanConstraint,
            filters::{snapshot_region_filter::SnapshotRegionFilter, snapshot_region_filter_collection::SnapshotRegionFilterCollection},
            parameters::element_scan::element_scan_parameters::ElementScanParameters,
        },
        snapshots::snapshot_region::SnapshotRegion,
    },
};
use std::sync::{Arc, RwLock};

pub trait ElementScanMappingRule: Send + Sync {
    fn get_id(&self) -> &str;
    fn map_parameters(
        &self,
        symbol_registry: &Arc<RwLock<SymbolRegistry>>,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        snapshot_region_filter: &SnapshotRegionFilter,
        element_scan_parameters: &ElementScanParameters,
        mapped_scan_parameters: &mut OptimizedScanConstraint,
    );
}
