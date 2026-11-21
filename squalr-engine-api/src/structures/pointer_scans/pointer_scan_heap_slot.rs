/// A heap slot in one discovery level. The level supplies its depth and candidate ID.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PointerScanHeapSlot {
    pub pointer_address: u64,
    pub pointer_value: u64,
}
