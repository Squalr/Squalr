use crate::structures::{
    scanning::{
        constraints::scan_constraint_finalized::ScanConstraintFinalized,
        filters::{snapshot_region_filter::SnapshotRegionFilter, snapshot_region_filter_collection::SnapshotRegionFilterCollection},
        plans::element_scan::snapshot_filter_element_scan_plan::SnapshotFilterElementScanPlan,
    },
    snapshots::snapshot_region::SnapshotRegion,
};

pub trait ElementScanFilterRule: Send + Sync {
    fn get_id(&self) -> &str;
    fn map_parameters(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter_collection: &SnapshotRegionFilterCollection,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_constraint_finalized: &ScanConstraintFinalized,
        snapshot_filter_element_scan_plan: &mut SnapshotFilterElementScanPlan,
    );
}
