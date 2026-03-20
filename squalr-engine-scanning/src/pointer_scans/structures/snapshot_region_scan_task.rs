#[derive(Clone, Copy)]
pub(crate) struct SnapshotRegionScanTask<'a> {
    pub(crate) base_address: u64,
    pub(crate) current_values: &'a [u8],
}
