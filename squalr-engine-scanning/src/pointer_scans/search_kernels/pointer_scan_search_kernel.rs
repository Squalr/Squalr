use crate::pointer_scans::search_kernels::pointer_scan_search_kernel_context::PointerScanSearchKernelContext;

pub use crate::pointer_scans::structures::pointer_scan_region_match::PointerScanRegionMatch;

/// Represents a concrete pointer-scan search kernel implementation.
pub trait PointerScanSearchKernel: Send + Sync {
    fn is_empty(
        &self,
        pointer_scan_search_kernel_context: &PointerScanSearchKernelContext<'_>,
    ) -> bool {
        pointer_scan_search_kernel_context
            .get_target_range_set()
            .is_empty()
    }

    fn scan_region_with_visitor(
        &self,
        pointer_scan_search_kernel_context: &PointerScanSearchKernelContext<'_>,
        base_address: u64,
        current_values: &[u8],
        visit_match: &mut dyn FnMut(PointerScanRegionMatch),
    );

    #[cfg(test)]
    fn scan_region(
        &self,
        pointer_scan_search_kernel_context: &PointerScanSearchKernelContext<'_>,
        base_address: u64,
        current_values: &[u8],
    ) -> Vec<PointerScanRegionMatch> {
        let mut pointer_matches = Vec::new();
        let mut visit_match = |pointer_match| pointer_matches.push(pointer_match);
        self.scan_region_with_visitor(pointer_scan_search_kernel_context, base_address, current_values, &mut visit_match);

        pointer_matches
    }
}
