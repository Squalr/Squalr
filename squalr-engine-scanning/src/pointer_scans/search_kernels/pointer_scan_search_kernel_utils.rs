use squalr_engine_api::structures::pointer_scans::pointer_scan_pointer_size::PointerScanPointerSize;

pub(super) fn find_scan_start_offset(
    base_address: u64,
    current_values: &[u8],
    pointer_size: PointerScanPointerSize,
) -> Option<usize> {
    let pointer_size_in_bytes = pointer_size.get_size_in_bytes() as usize;

    if current_values.len() < pointer_size_in_bytes {
        return None;
    }

    let pointer_alignment = pointer_size_in_bytes as u64;
    let alignment_remainder = base_address % pointer_alignment;
    let start_offset = if alignment_remainder == 0 {
        0_usize
    } else {
        pointer_alignment.saturating_sub(alignment_remainder) as usize
    };

    (start_offset.saturating_add(pointer_size_in_bytes) <= current_values.len()).then_some(start_offset)
}
