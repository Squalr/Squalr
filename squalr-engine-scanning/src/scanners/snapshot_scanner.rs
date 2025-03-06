use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_common::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_common::structures::memory_alignment::MemoryAlignment;
use squalr_engine_common::structures::scanning::scan_parameters::ScanParameters;

pub trait Scanner: Send + Sync {
    fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters: &ScanParameters,
        data_type: &DataTypeRef,
        memory_alignment: MemoryAlignment,
    ) -> Vec<SnapshotRegionFilter>;
}
