use crate::filters::snapshot_region_filter::SnapshotRegionFilter;

pub struct SnapshotRegionFilterRunLengthEncoder {
    // Public so that this can be directly taken by callers
    pub result_regions: Vec<SnapshotRegionFilter>,
    run_length_current_address: u64,
    is_encoding: bool,
    run_length: u64,
}

/// Implements a run length encoder, which (as far as I know) is the most efficient way for memory scanners to create results.
/// The reason for the speed is that this works extremely well for common case scenarios (ie scanning for 0, 1, 255) as a first scan.
/// The idea is that we iterate over a block of memory (either as a scalar or vector scan), and when the scan passes, we track how many
/// scans succeeded as a run length in bytes. Once we encounter a failed scan, we finish off the region and allocate a new subregion
/// containing the results. We then stop encoding until we reach a new scan that passes, and the cycle repeats until we are done
/// iterating over the entire block of memory. The caller is responsible for this iteration, as it depends highly on alignment,
/// SIMD vs scalar, etc.
/// 
/// The caller can get additional performance gains by dividing the work among several run length encoders, then stitching together
/// boundary regions once the run length encoders are complete.
impl SnapshotRegionFilterRunLengthEncoder {
    pub fn new(
        run_length_current_address: u64
    ) -> Self {
        Self {
            result_regions: vec![],
            run_length_current_address: run_length_current_address,
            is_encoding: false,
            run_length: 0,
        }
    }

    pub fn adjust_for_misalignment(
        &mut self,
        misalignment_offset: u64
    ) {
        self.run_length_current_address = self.run_length_current_address.saturating_sub(misalignment_offset);
    }

    /// Encodes the next N bytes as true (ie passing the scan).
    pub fn encode_range(
        &mut self,
        // The number of bytes to advance the run length. For scalar scans, this is the memory alignment.
        // For scalar scans, this is generally the size of the hardware vector.
        byte_advance_count: u64
    ) {
        self.run_length += byte_advance_count;
        self.is_encoding = true;
    }

    /// Completes the current run length encoding, creating a region filter from the result.
    pub fn finalize_current_encode_unchecked(
        &mut self,
        // The number of bytes to advance the run length. For scalar scans, this is the memory alignment.
        // For scalar scans, this is generally the size of the hardware vector.
        byte_advance_count: u64,
        // The size of the data type being encoded. This allows us to properly adjust the size of the final run length encoding.
        data_type_size: u64
    ) {
        if self.is_encoding {
            self.result_regions.push(SnapshotRegionFilter::new(
                self.run_length_current_address,
                self.run_length + (data_type_size - byte_advance_count),
            ));
            self.run_length_current_address += self.run_length;
            self.run_length = 0;
            self.is_encoding = false;
        }

        self.run_length_current_address += byte_advance_count;
    }
}
