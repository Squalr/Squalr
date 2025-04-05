use squalr_engine_api::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;

pub struct SnapshotRegionFilterRunLengthEncoder {
    result_regions: Vec<SnapshotRegionFilter>,
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
/// This can be parallelized by the caller by simply creating N run length encoders over a snapshot region filter that has been divided into N chunks.
/// This requires a post-step of map-reducing the gathered regions and stitching together boundary regions once the run length encoders are complete.
/// Additionally, some care would need to be taken to avoid discarding any small regions at the start or end of the encoder.
/// For this reason, we currently do not try to parallelize them, as we can get high CPU utilization by parallelizing in other ways.
impl SnapshotRegionFilterRunLengthEncoder {
    pub fn new(run_length_current_address: u64) -> Self {
        Self {
            result_regions: vec![],
            run_length_current_address,
            is_encoding: false,
            run_length: 0,
        }
    }

    /// Takes ownership of the resulting region filters from this encoder. Note that once this is called, the regions are emptied from this struct.
    pub fn take_result_regions(&mut self) -> Vec<SnapshotRegionFilter> {
        std::mem::take(&mut self.result_regions)
    }

    pub fn get_current_address(&self) -> u64 {
        self.run_length_current_address
    }

    pub fn get_current_run_length(&self) -> u64 {
        self.run_length
    }

    pub fn is_encoding(&self) -> bool {
        self.is_encoding
    }

    /// Encodes the next N bytes as true (ie passing the scan).
    pub fn encode_range(
        &mut self,
        // The number of bytes to advance the run length. For scalar scans, this is the memory alignment.
        // For scalar scans, this is generally the size of the memory alignment.
        byte_advance_count: u64,
    ) {
        self.run_length += byte_advance_count;
        self.is_encoding = true;
    }

    pub fn unshift_current_encode(
        &mut self,
        shift_amount: u64,
    ) {
        self.run_length_current_address = self.run_length_current_address.saturating_sub(shift_amount);
    }

    pub fn shift_current_encode(
        &mut self,
        shift_amount: u64,
    ) {
        self.run_length_current_address = self.run_length_current_address.saturating_add(shift_amount);
    }

    /// Completes the current run length encoding, creating a region filter from the result.
    pub fn finalize_current_encode(
        &mut self,
        // The number of bytes to advance the run length. For scalar scans, this is the memory alignment.
        // For vector scans, this is generally the size of the hardware vector.
        byte_advance_count: u64,
    ) {
        if self.is_encoding && self.run_length > 0 {
            self.result_regions
                .push(SnapshotRegionFilter::new(self.run_length_current_address, self.run_length));
            self.run_length_current_address += self.run_length;
            self.run_length = 0;
            self.is_encoding = false;
        }

        self.run_length_current_address += byte_advance_count;
    }

    /// Completes the current run length encoding, creating a region filter from the result, padding it for a given data size.
    pub fn finalize_current_encode_with_padding(
        &mut self,
        // The number of bytes to advance the run length. For scalar scans, this is the memory alignment.
        // For vector scans, this is generally the size of the hardware vector.
        byte_advance_count: u64,
        // The padding to add to the length of the values being encoded in the current run length.
        length_padding: u64,
    ) {
        if self.is_encoding && self.run_length > 0 {
            self.result_regions.push(SnapshotRegionFilter::new(
                self.run_length_current_address,
                self.run_length.saturating_add(length_padding),
            ));
            self.run_length_current_address += self.run_length;
            self.run_length = 0;
            self.is_encoding = false;
        }

        self.run_length_current_address += byte_advance_count;
    }

    /// Completes the current run length encoding, creating a region filter from the result. Discards regions below the minimum size.
    pub fn finalize_current_encode_with_minimum_size(
        &mut self,
        // The number of bytes to advance the run length. For scalar scans, this is the memory alignment.
        // For vector scans, this is generally the size of the hardware vector.
        byte_advance_count: u64,
        // The minimum size allowed to create a region.
        minimum_size: u64,
    ) {
        if self.is_encoding && self.run_length > 0 {
            if self.run_length >= minimum_size {
                self.result_regions
                    .push(SnapshotRegionFilter::new(self.run_length_current_address, self.run_length));
            }
            self.run_length_current_address += self.run_length;
            self.run_length = 0;
            self.is_encoding = false;
        }

        self.run_length_current_address += byte_advance_count;
    }

    /// Completes the current run length encoding, creating a region filter from the result. Discards regions below the minimum size.
    /// Additionally this takes a range adjustor to allow custom tweaks before finalizing an encoding.
    pub fn finalize_current_encode_periodic(
        &mut self,
        // The number of bytes to advance the run length. For scalar scans, this is the memory alignment.
        // For vector scans, this is generally the size of the hardware vector.
        byte_advance_count: u64,
        // The size at which a periodic region is split into multiple-filters. This also serves as the minimum possible filter size.
        split_size: u64,
        // A custom adjustor to the current address and run length.
        range_adjustor: &dyn Fn(u64, u64) -> (u64, u64),
    ) {
        if self.is_encoding && self.run_length > 0 {
            if self.run_length >= split_size {
                // Allow the callera to adjust the range of our run length encoding to meet periodicity requirements.
                let (new_current_address, new_run_length) = range_adjustor(self.run_length_current_address, self.run_length);
                let filter_count = new_run_length / split_size;

                for index in 0..filter_count {
                    self.result_regions
                        .push(SnapshotRegionFilter::new(new_current_address.saturating_add(index * split_size), split_size));
                }
            }
            self.run_length_current_address += self.run_length;
            self.run_length = 0;
            self.is_encoding = false;
        }

        self.run_length_current_address += byte_advance_count;
    }
}
