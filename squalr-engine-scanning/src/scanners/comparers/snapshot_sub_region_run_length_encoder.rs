use crate::snapshots::snapshot_sub_region::SnapshotSubRegion;

pub struct SnapshotSubRegionRunLengthEncoder {
    run_length_encode_offset: u64,
    is_encoding: bool,
    run_length: u64,
    result_regions: Vec<SnapshotSubRegion>,
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
impl SnapshotSubRegionRunLengthEncoder {
    pub fn new(snapshot_sub_region: &SnapshotSubRegion) -> Self {
        Self {
            run_length_encode_offset: snapshot_sub_region.get_base_address(),
            is_encoding: false,
            run_length: 0,
            result_regions: vec![],
        }
    }

    pub fn get_collected_regions(&self) -> &Vec<SnapshotSubRegion> {
        return &self.result_regions;
    }
    
    pub fn merge_from_other_encoder(&mut self,
        other: &SnapshotSubRegionRunLengthEncoder
    ) {
        self.result_regions.extend_from_slice(&other.result_regions);
    }
    
    pub fn combine_adjacent_sub_regions(&mut self) {
        // unimplemented!("Implement me!");
    }

    pub fn adjust_for_misalignment(&mut self,
        misalignment_offset: u64
    ) {
        self.run_length_encode_offset = self.run_length_encode_offset.saturating_sub(misalignment_offset);
    }

    pub fn encode_range(&mut self,
        memory_alignment: u64
    ) {
        self.run_length += memory_alignment;
        self.is_encoding = true;
    }

    pub fn finalize_current_encode_unchecked(&mut self,
        memory_alignment: u64,
        data_type_size: u64
    ) {
        if self.is_encoding && self.run_length > 0 {
            self.result_regions.push(SnapshotSubRegion::new_with_offset_and_size_in_bytes(
                self.run_length_encode_offset,
                self.run_length + (data_type_size - 1),
            ));
            self.run_length_encode_offset += self.run_length;
            self.run_length = 0;
            self.is_encoding = false;
        }

        self.run_length_encode_offset += memory_alignment;
    }
}
