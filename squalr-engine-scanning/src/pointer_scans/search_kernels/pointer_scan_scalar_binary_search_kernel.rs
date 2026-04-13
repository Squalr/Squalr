use crate::pointer_scans::search_kernels::pointer_scan_scalar_region_scanner::scan_region_scalar_with_predicate;
use crate::pointer_scans::structures::pointer_scan_region_match::PointerScanRegionMatch;
use crate::pointer_scans::structures::pointer_scan_target_ranges::PointerScanTargetRangeSet;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;

pub(crate) fn scan_region_scalar_binary<VisitMatch>(
    base_address: u64,
    current_values: &[u8],
    start_offset: usize,
    pointer_size: PointerScanPointerSize,
    target_range_set: &PointerScanTargetRangeSet,
    visit_match: &mut VisitMatch,
) where
    VisitMatch: FnMut(PointerScanRegionMatch),
{
    scan_region_scalar_with_predicate(
        base_address,
        current_values,
        start_offset,
        pointer_size,
        |pointer_value| target_range_set.contains_value_binary(pointer_value),
        visit_match,
    );
}
