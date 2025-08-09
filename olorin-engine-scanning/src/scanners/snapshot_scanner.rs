use olorin_engine_api::registries::symbols::symbol_registry::SymbolRegistry;
use olorin_engine_api::structures::scanning::{
    filters::snapshot_region_filter::SnapshotRegionFilter, parameters::mapped::mapped_scan_parameters::MappedScanParameters,
};
use olorin_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;
use std::sync::{Arc, RwLock};

pub trait Scanner: Send + Sync {
    fn get_scanner_name(&self) -> &'static str;
    fn scan_region(
        &self,
        data_type_registry: &Arc<RwLock<SymbolRegistry>>,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        mapped_scan_parameters: &MappedScanParameters,
    ) -> Vec<SnapshotRegionFilter>;
}
