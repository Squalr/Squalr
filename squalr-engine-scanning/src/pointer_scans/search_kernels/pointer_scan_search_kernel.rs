pub use crate::pointer_scans::structures::pointer_scan_region_match::PointerScanRegionMatch;

/// Represents a concrete pointer-scan search kernel implementation.
pub trait PointerScanSearchKernel: Sync {
    fn is_empty(&self) -> bool;

    fn scan_region_with_visitor(
        &self,
        base_address: u64,
        current_values: &[u8],
        visit_match: &mut dyn FnMut(PointerScanRegionMatch),
    );

    #[cfg(test)]
    fn scan_region(
        &self,
        base_address: u64,
        current_values: &[u8],
    ) -> Vec<PointerScanRegionMatch> {
        let mut pointer_matches = Vec::new();
        let mut visit_match = |pointer_match| pointer_matches.push(pointer_match);
        self.scan_region_with_visitor(base_address, current_values, &mut visit_match);

        pointer_matches
    }
}
