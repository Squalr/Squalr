use crate::pointer_scans::search_kernels::pointer_scan_pointer_value_reader::read_pointer_value_unchecked;
use crate::pointer_scans::structures::pointer_scan_region_match::PointerScanRegionMatch;
use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;

pub(crate) fn scan_region_scalar<MatchesPointerValue, VisitMatch>(
    base_address: u64,
    current_values: &[u8],
    start_offset: usize,
    pointer_size: PointerScanPointerSize,
    mut matches_pointer_value: MatchesPointerValue,
    visit_match: &mut VisitMatch,
) where
    MatchesPointerValue: FnMut(u64) -> bool,
    VisitMatch: FnMut(PointerScanRegionMatch),
{
    let pointer_size_in_bytes = pointer_size.get_size_in_bytes() as usize;
    let current_values_ptr = current_values.as_ptr();
    let mut pointer_value_offset = start_offset;

    while pointer_value_offset.saturating_add(pointer_size_in_bytes) <= current_values.len() {
        // The loop guard guarantees a full pointer-sized unaligned load.
        let pointer_value = unsafe { read_pointer_value_unchecked(current_values_ptr.add(pointer_value_offset), pointer_size) };

        if matches_pointer_value(pointer_value) {
            visit_match(PointerScanRegionMatch::new(
                base_address.saturating_add(pointer_value_offset as u64),
                pointer_value,
            ));
        }

        pointer_value_offset = pointer_value_offset.saturating_add(pointer_size_in_bytes);
    }
}
