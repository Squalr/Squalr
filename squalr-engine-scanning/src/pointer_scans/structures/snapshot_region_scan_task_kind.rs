#[derive(Clone, Copy)]
pub(crate) enum SnapshotRegionScanTaskKind {
    Static { module_index: usize, module_base_address: u64 },
    Heap,
}
