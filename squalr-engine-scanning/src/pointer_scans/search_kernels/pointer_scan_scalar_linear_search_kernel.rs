use crate::pointer_scans::search_kernels::pointer_scan_scalar_region_scanner::scan_region_scalar_with_predicate;
use crate::pointer_scans::search_kernels::pointer_scan_search_kernel::{PointerScanRegionMatch, PointerScanSearchKernel};
use crate::pointer_scans::search_kernels::pointer_scan_search_kernel_context::PointerScanSearchKernelContext;
use crate::pointer_scans::search_kernels::pointer_scan_search_kernel_utils::find_scan_start_offset;

pub(crate) struct ScalarLinearPointerScanKernel;

impl PointerScanSearchKernel for ScalarLinearPointerScanKernel {
    fn scan_region_with_visitor(
        &self,
        pointer_scan_search_kernel_context: &PointerScanSearchKernelContext<'_>,
        base_address: u64,
        current_values: &[u8],
        visit_match: &mut dyn FnMut(PointerScanRegionMatch),
    ) {
        let pointer_size = pointer_scan_search_kernel_context.get_pointer_size();
        let target_range_set = pointer_scan_search_kernel_context.get_target_range_set();
        let Some(start_offset) = find_scan_start_offset(base_address, current_values, pointer_size) else {
            return;
        };

        scan_region_scalar_with_predicate(
            base_address,
            current_values,
            start_offset,
            pointer_size,
            |pointer_value| target_range_set.contains_value_linear(pointer_value),
            visit_match,
        );
    }
}
