use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::structures::boyer_moore_table::BoyerMooreTable;
use crate::scanners::structures::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use squalr_engine_api::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use squalr_engine_api::structures::scanning::scan_parameters_local::ScanParametersLocal;

pub struct ScannerScalarByteArrayNonOverlapping {}

impl ScannerScalarByteArrayNonOverlapping {}

/// Implements a scalar (ie CPU bound, non-SIMD) array of bytes region scanning algorithm. This works by using the Boyer-Moore
/// algorithm to encode matches as they are discovered. This algorithm does not support overlapping results.
impl Scanner for ScannerScalarByteArrayNonOverlapping {
    /// Performs a sequential iteration over a region of memory, performing the scan comparison. A run-length encoding algorithm
    /// is used to generate new sub-regions as the scan progresses.
    fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters_global: &ScanParametersGlobal,
        scan_parameters_local: &ScanParametersLocal,
    ) -> Vec<SnapshotRegionFilter> {
        let data_value = match scan_parameters_global.get_compare_type() {
            ScanCompareType::Immediate(scan_compare_type_immediate) => match scan_compare_type_immediate {
                ScanCompareTypeImmediate::Equal => {
                    if let Some(data_value) = scan_parameters_global.deanonymize_immediate(scan_parameters_local.get_data_type()) {
                        data_value
                    } else {
                        log::error!("Failed to deanonymize array of byte value.");
                        return vec![];
                    }
                }
                _ => {
                    log::error!("Unsupported immediate scan constraint. Only equality is supported for array of byte scans.");
                    return vec![];
                }
            },
            ScanCompareType::Relative(_scan_compare_type_relative) => {
                log::error!("Relative array of byte scans are not supported yet (or maybe ever).");
                return vec![];
            }
            ScanCompareType::Delta(_scan_compare_type_delta) => {
                log::error!("Delta array of byte scans are not supported yet (or maybe ever).");
                return vec![];
            }
        };

        let current_value_pointer = snapshot_region.get_current_values_filter_pointer(&snapshot_region_filter);
        let base_address = snapshot_region_filter.get_base_address();
        let region_size = snapshot_region_filter.get_region_size();
        let memory_alignment = scan_parameters_local.get_memory_alignment_or_default() as u64;

        let scan_pattern = data_value.get_value_bytes();
        let pattern_length = scan_pattern.len() as u64;
        let boyer_moore_table = BoyerMooreTable::new(&scan_pattern, memory_alignment);
        let aligned_pattern_length = boyer_moore_table.get_aligned_pattern_length();
        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let mut scan_index: u64 = 0;

        // Main body of the Boyer-Moore algorithm, see https://en.wikipedia.org/wiki/Boyer%E2%80%93Moore_string-search_algorithm for details.
        // Or honestly go watch a YouTube video, visuals are probably better for actually understanding. It's pretty simple actually.
        while scan_index <= region_size - pattern_length {
            let mut match_found = true;
            let mut shift_value = aligned_pattern_length;

            for inverse_pattern_index in (0..pattern_length as usize).rev() {
                let current_byte = unsafe { *current_value_pointer.add((scan_index + inverse_pattern_index as u64) as usize) };
                let pattern_byte = scan_pattern[inverse_pattern_index];

                // JIRA: Also check masking table when we decide to support masking.
                let is_mismatch = current_byte != pattern_byte;

                if is_mismatch {
                    match_found = false;

                    let bad_char_shift = boyer_moore_table.get_mismatch_shift(current_byte);
                    let good_suffix_shift = boyer_moore_table.get_good_suffix_shift(inverse_pattern_index);
                    shift_value = bad_char_shift.max(good_suffix_shift).max(memory_alignment);
                    break;
                }
            }

            // Two key differences to vanilla Boyer-Moore. First, our run length encoder needs to advance every time our
            // index advances. This is either going to be by aligned pattern length (for a match), or the shift length (for a mismatch).
            // Note that the original algorithm advances by a full pattern length, but we may advance by a little bit more to keep alignment.
            if match_found {
                scan_index = scan_index.saturating_add(aligned_pattern_length);
                run_length_encoder.encode_range(aligned_pattern_length);
            } else {
                // Shift values should always be memory aligned, so no need to worry if not.
                run_length_encoder.finalize_current_encode_with_minimum_size(shift_value, pattern_length);
                scan_index += shift_value;
            }
        }

        run_length_encoder.finalize_current_encode_with_minimum_size(0, pattern_length);
        run_length_encoder.take_result_regions()
    }
}
