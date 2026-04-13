use crate::pointer_scans::search_kernels::pointer_scan_scalar_region_scanner::scan_region_scalar_with_predicate;
use crate::pointer_scans::search_kernels::pointer_scan_search_kernel::{PointerScanRegionMatch, PointerScanSearchKernel};
use crate::pointer_scans::search_kernels::pointer_scan_search_kernel_utils::find_scan_start_offset;
use crate::pointer_scans::structures::pointer_scan_target_ranges::PointerScanTargetRangeSet;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;

pub(crate) struct ScalarBinaryPointerScanKernel<'a> {
    target_range_set: &'a PointerScanTargetRangeSet,
    pointer_size: PointerScanPointerSize,
}

impl<'a> ScalarBinaryPointerScanKernel<'a> {
    pub(crate) fn new(
        target_range_set: &'a PointerScanTargetRangeSet,
        pointer_size: PointerScanPointerSize,
    ) -> Self {
        Self {
            target_range_set,
            pointer_size,
        }
    }
}

impl PointerScanSearchKernel for ScalarBinaryPointerScanKernel<'_> {
    fn is_empty(&self) -> bool {
        self.target_range_set.is_empty()
    }

    fn scan_region_with_visitor(
        &self,
        base_address: u64,
        current_values: &[u8],
        visit_match: &mut dyn FnMut(PointerScanRegionMatch),
    ) {
        let Some(start_offset) = find_scan_start_offset(base_address, current_values, self.pointer_size) else {
            return;
        };

        scan_region_scalar_with_predicate(
            base_address,
            current_values,
            start_offset,
            self.pointer_size,
            |pointer_value| self.target_range_set.contains_value_binary(pointer_value),
            visit_match,
        );
    }
}
