use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::structures::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use squalr_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use squalr_engine_api::structures::scanning::plans::element_scan::snapshot_filter_element_scan_plan::SnapshotFilterElementScanPlan;
use squalr_engine_api::structures::snapshots::snapshot_region::SnapshotRegion;

pub struct ScannerScalarByteArrayBooyerMooreMasked {}

impl ScannerScalarByteArrayBooyerMooreMasked {}

/// Implements a masked scalar byte-array scan using Boyer-Moore-style overlap-preserving shifts.
impl Scanner for ScannerScalarByteArrayBooyerMooreMasked {
    fn get_scanner_name(&self) -> &'static str {
        &"Byte Array (Masked Booyer-Moore)"
    }

    fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        snapshot_filter_element_scan_plan: &SnapshotFilterElementScanPlan,
    ) -> Vec<SnapshotRegionFilter> {
        let data_value = match snapshot_filter_element_scan_plan.get_compare_type() {
            ScanCompareType::Immediate(scan_compare_type_immediate) => match scan_compare_type_immediate {
                ScanCompareTypeImmediate::Equal => snapshot_filter_element_scan_plan.get_data_value(),
                _ => {
                    log::error!("Unsupported immediate scan constraint. Only equality is supported for masked array of byte scans.");
                    return vec![];
                }
            },
            ScanCompareType::Relative(_scan_compare_type_relative) => {
                log::error!("Relative masked array of byte scans are not supported yet (or maybe ever).");
                return vec![];
            }
            ScanCompareType::Delta(_scan_compare_type_delta) => {
                log::error!("Delta masked array of byte scans are not supported yet (or maybe ever).");
                return vec![];
            }
        };

        let scan_mask = match snapshot_filter_element_scan_plan
            .get_scan_constraint()
            .get_mask()
        {
            Some(scan_mask) => scan_mask,
            None => {
                log::error!("Masked byte-array scanner was selected without mask data in scan constraint.");
                return vec![];
            }
        };

        let current_value_pointer = snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter);
        let base_address = snapshot_region_filter.get_base_address();
        let region_size = snapshot_region_filter.get_region_size();
        let memory_alignment_size = snapshot_filter_element_scan_plan.get_memory_alignment() as u64;

        let scan_pattern = data_value.get_value_bytes();

        if scan_pattern.len() != scan_mask.len() {
            log::error!(
                "Masked byte-array scan pattern/mask length mismatch: pattern={}, mask={}",
                scan_pattern.len(),
                scan_mask.len()
            );
            return vec![];
        }

        let pattern_length = scan_pattern.len() as u64;
        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let mut scan_index: u64 = 0;
        let data_type_size_padding = pattern_length.saturating_sub(memory_alignment_size);

        while scan_index <= region_size.saturating_sub(pattern_length) {
            let mut match_found = true;
            let mut shift_value = memory_alignment_size;

            for inverse_pattern_index in (0..pattern_length as usize).rev() {
                let current_byte = unsafe { *current_value_pointer.add((scan_index + inverse_pattern_index as u64) as usize) };
                let pattern_byte = scan_pattern[inverse_pattern_index];
                let mask_byte = scan_mask[inverse_pattern_index];
                let masked_current_byte = current_byte & mask_byte;
                let masked_pattern_byte = pattern_byte & mask_byte;
                let is_mismatch = masked_current_byte != masked_pattern_byte;

                if is_mismatch {
                    match_found = false;
                    shift_value = Self::get_safe_mismatch_shift(scan_pattern, scan_mask, current_byte, inverse_pattern_index, memory_alignment_size);
                    break;
                }
            }

            if match_found {
                scan_index = scan_index.saturating_add(memory_alignment_size);
                run_length_encoder.encode_range(memory_alignment_size);
            } else {
                run_length_encoder.finalize_current_encode_with_padding(shift_value, data_type_size_padding);
                scan_index += shift_value;
            }
        }

        run_length_encoder.finalize_current_encode_with_padding(0, data_type_size_padding);
        run_length_encoder.take_result_regions()
    }
}

impl ScannerScalarByteArrayBooyerMooreMasked {
    fn get_safe_mismatch_shift(
        scan_pattern: &[u8],
        scan_mask: &[u8],
        mismatched_byte: u8,
        mismatch_pattern_index: usize,
        memory_alignment_size: u64,
    ) -> u64 {
        for compatible_pattern_index in (0..mismatch_pattern_index).rev() {
            let pattern_mask = scan_mask[compatible_pattern_index];
            let masked_pattern_byte = scan_pattern[compatible_pattern_index] & pattern_mask;
            let masked_mismatched_byte = mismatched_byte & pattern_mask;

            if masked_mismatched_byte == masked_pattern_byte {
                let raw_shift = mismatch_pattern_index.saturating_sub(compatible_pattern_index) as u64;

                return Self::round_up_to_alignment(raw_shift.max(1), memory_alignment_size);
            }
        }

        Self::round_up_to_alignment(mismatch_pattern_index.saturating_add(1) as u64, memory_alignment_size)
    }

    fn round_up_to_alignment(
        value: u64,
        alignment: u64,
    ) -> u64 {
        let remainder = value % alignment;

        value + (alignment - remainder) % alignment
    }
}

#[cfg(test)]
mod tests {
    use super::ScannerScalarByteArrayBooyerMooreMasked;

    fn scan_offsets_masked(
        haystack: &[u8],
        pattern: &[u8],
        mask: &[u8],
        memory_alignment_size: u64,
    ) -> Vec<usize> {
        let pattern_length = pattern.len();
        let mut scan_index = 0usize;
        let mut results = Vec::new();

        while scan_index + pattern_length <= haystack.len() {
            let mut match_found = true;
            let mut shift_value = memory_alignment_size as usize;

            for inverse_pattern_index in (0..pattern_length).rev() {
                let masked_current_byte = haystack[scan_index + inverse_pattern_index] & mask[inverse_pattern_index];
                let masked_pattern_byte = pattern[inverse_pattern_index] & mask[inverse_pattern_index];

                if masked_current_byte != masked_pattern_byte {
                    match_found = false;
                    shift_value = ScannerScalarByteArrayBooyerMooreMasked::get_safe_mismatch_shift(
                        pattern,
                        mask,
                        haystack[scan_index + inverse_pattern_index],
                        inverse_pattern_index,
                        memory_alignment_size,
                    ) as usize;
                    break;
                }
            }

            if match_found {
                results.push(scan_index);
                scan_index += memory_alignment_size as usize;
            } else {
                scan_index += shift_value;
            }
        }

        results
    }

    #[test]
    fn masked_shift_preserves_wildcard_prefixed_match() {
        let pattern = [0x35_u8, 0xD4_u8];
        let mask = [0x00_u8, 0xFF_u8];
        let haystack = [0x20_u8, 0x32_u8, 0xD4_u8, 0x4F_u8, 0x0F_u8, 0xE4_u8];

        assert_eq!(scan_offsets_masked(&haystack, &pattern, &mask, 1), vec![1]);
    }
}
