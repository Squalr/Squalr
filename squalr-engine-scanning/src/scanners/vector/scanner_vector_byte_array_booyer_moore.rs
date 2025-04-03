use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::snapshot_scanner::Scanner;
use crate::scanners::structures::boyer_moore_table::BoyerMooreTable;
use crate::scanners::structures::snapshot_region_filter_run_length_encoder::SnapshotRegionFilterRunLengthEncoder;
use crate::snapshots::snapshot_region::SnapshotRegion;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type_immediate::ScanCompareTypeImmediate;
use squalr_engine_api::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use squalr_engine_api::structures::scanning::scan_parameters_local::ScanParametersLocal;

pub struct ScannerVectorByteArrayBooyerMoore {}

impl ScannerVectorByteArrayBooyerMoore {}

/// Implements a vector (CPU-bound, SIMD) array of bytes region scanning algorithm. This works by using a modified version
/// of the Boyer-Moore algorithm to encode matches as they are discovered. This algorithm was adapted to support overlapping results.
/// Boyer-Moore speeds up scans through use of pre-made shifting tables, allowing for skipping multiple elements on failed matches.
impl Scanner for ScannerVectorByteArrayBooyerMoore {
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
        let memory_alignment_size = scan_parameters_local.get_memory_alignment_or_default() as u64;

        let scan_pattern = data_value.get_value_bytes();
        let pattern_length = scan_pattern.len() as u64;
        let boyer_moore_table = BoyerMooreTable::new(&scan_pattern, memory_alignment_size);
        let mut run_length_encoder = SnapshotRegionFilterRunLengthEncoder::new(base_address);
        let mut scan_index: u64 = 0;
        let data_type_size_padding = pattern_length.saturating_sub(memory_alignment_size);

        // Main body of the Boyer-Moore algorithm, see https://en.wikipedia.org/wiki/Boyer%E2%80%93Moore_string-search_algorithm for details.
        // Or honestly go watch a YouTube video, visuals are probably better for actually understanding. It's pretty simple actually.
        while scan_index <= region_size.saturating_sub(pattern_length) {
            let mut match_found = true;
            let mut shift_value = memory_alignment_size;

            for inverse_pattern_index in (0..pattern_length as usize).rev() {
                let current_byte = unsafe { *current_value_pointer.add((scan_index + inverse_pattern_index as u64) as usize) };
                let pattern_byte = scan_pattern[inverse_pattern_index];

                // JIRA: Also check masking table when we decide to support masking.
                let is_mismatch = current_byte != pattern_byte;

                if is_mismatch {
                    match_found = false;

                    let bad_char_shift = boyer_moore_table.get_mismatch_shift(current_byte);
                    let good_suffix_shift = boyer_moore_table.get_good_suffix_shift(inverse_pattern_index + 1);

                    // Unlike classic Booyer-Moore we don't take the max of the shifts. Instead we prioritize a set good shift over the bad shift.
                    // This reduces some skipping, but allows us to safely handle overlapping results.
                    shift_value = (if good_suffix_shift > 0 { good_suffix_shift } else { bad_char_shift }).max(memory_alignment_size);
                    break;
                }
            }

            // A few key differences to vanilla Boyer-Moore. First, our run length encoder needs to advance every time our index advances.
            // Second, we advance by memory alignment for matches, whereas in the original algorithm matches advance by the length of the pattern.
            // This allows us to capture overlapping matches, which is not supported in classic Boyer-Moore. For mismatches, we advance by the shift length.
            if match_found {
                scan_index = scan_index.saturating_add(memory_alignment_size);
                run_length_encoder.encode_range(memory_alignment_size);
            } else {
                // Shift values should always be memory aligned, so no need to worry if not.
                run_length_encoder.finalize_current_encode_with_padding(shift_value, data_type_size_padding);
                scan_index += shift_value;
            }
        }

        run_length_encoder.finalize_current_encode_with_padding(0, data_type_size_padding);
        run_length_encoder.take_result_regions()
    }
}
