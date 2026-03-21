use crate::pointer_scans::structures::snapshot_region_scan_task_kind::SnapshotRegionScanTaskKind;

#[derive(Clone, Copy)]
pub(crate) struct SnapshotRegionScanTask<'a> {
    pub(crate) scan_base_address: u64,
    pub(crate) scan_end_address: u64,
    pub(crate) current_values: &'a [u8],
    pub(crate) task_kind: SnapshotRegionScanTaskKind,
}
