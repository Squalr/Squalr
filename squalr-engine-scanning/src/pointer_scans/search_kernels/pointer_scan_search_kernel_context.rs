use crate::pointer_scans::structures::pointer_scan_target_ranges::PointerScanTargetRangeSet;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;

/// Shared per-dispatch inputs for a concrete pointer-scan search kernel.
pub(crate) struct PointerScanSearchKernelContext<'a> {
    target_range_set: &'a PointerScanTargetRangeSet,
    pointer_size: PointerScanPointerSize,
}

impl<'a> PointerScanSearchKernelContext<'a> {
    pub(crate) fn new(
        target_range_set: &'a PointerScanTargetRangeSet,
        pointer_size: PointerScanPointerSize,
    ) -> Self {
        Self {
            target_range_set,
            pointer_size,
        }
    }

    pub(crate) fn get_target_range_set(&self) -> &'a PointerScanTargetRangeSet {
        self.target_range_set
    }

    pub(crate) fn get_pointer_size(&self) -> PointerScanPointerSize {
        self.pointer_size
    }
}
