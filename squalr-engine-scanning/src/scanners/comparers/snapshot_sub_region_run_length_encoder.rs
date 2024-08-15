use crate::snapshots::snapshot_sub_region::SnapshotSubRegion;
use std::sync::{Arc, RwLock};

pub struct SnapshotSubRegionRunLengthEncoder {
    run_length_encode_offset: usize,
    is_encoding: bool,
    run_length: usize,
    snapshot_sub_region: Arc<RwLock<SnapshotSubRegion>>,
    result_regions: Vec<Arc<RwLock<SnapshotSubRegion>>>,
    parent_region_base_address: u64,
}

/// Implements a run length encoder, which (as far as I know) is the most efficient way for memory scanners to create results.
/// The reason for the speed is that this works extremely well for common case scenarios (ie scanning for 0, 1, 255) as a first scan.
/// The idea is that we iterate over a block of memory (either as a scalar or vector scan), and when the scan passes, we track how many
/// scans succeeded as a run length in bytes. Once we encounter a failed scan, we finish off the region and allocate a new subregion
/// containing the results. We then stop encoding until we reach a new scan that passes, and the cycle repeats until we are done
/// iterating over the entire block of memory. The caller is responsible for this iteration, as it depends highly on alignment,
/// SIMD vs scalar, etc.
impl SnapshotSubRegionRunLengthEncoder {
    pub fn new(snapshot_sub_region: Arc<RwLock<SnapshotSubRegion>>) -> Self {
        Self {
            run_length_encode_offset: 0,
            is_encoding: false,
            run_length: 0,
            snapshot_sub_region: snapshot_sub_region,
            result_regions: Vec::new(),
            parent_region_base_address: 0,
        }
    }

    pub fn initialize(&mut self) {
        self.parent_region_base_address = self.snapshot_sub_region.read().unwrap().parent_region.read().unwrap().get_base_address();
        self.run_length_encode_offset = self.snapshot_sub_region.read().unwrap().get_region_offset();
    }

    pub fn adjust_for_misalignment(&mut self, misalignment_offset: usize) {
        self.run_length_encode_offset = self.run_length_encode_offset.saturating_sub(misalignment_offset);
    }

    pub fn encode_range(&mut self, memory_alignment: usize) {
        self.run_length += memory_alignment;
        self.is_encoding = true;
    }

    pub fn finalize_current_encode_checked(&mut self, memory_alignment: usize, data_type_size: usize) {
        if self.is_encoding {
            let absolute_address_start = self.parent_region_base_address + self.run_length_encode_offset as u64;
            let absolute_address_end = absolute_address_start + self.run_length as u64;

            if absolute_address_start >= self.snapshot_sub_region.read().unwrap().get_base_element_address()
                && absolute_address_end <= self.snapshot_sub_region.read().unwrap().get_end_element_address()
            {
                self.result_regions.push(Arc::new(RwLock::new(SnapshotSubRegion::new_with_offset_and_range(
                    self.snapshot_sub_region.read().unwrap().parent_region.clone(),
                    self.run_length_encode_offset,
                    self.run_length + (data_type_size - 1),
                ))));
            }

            self.run_length_encode_offset += self.run_length;
            self.run_length = 0;
            self.is_encoding = false;
        }

        self.run_length_encode_offset += memory_alignment;
    }

    pub fn finalize_current_encode_unchecked(&mut self, memory_alignment: usize, data_type_size: usize) {
        if self.is_encoding && self.run_length > 0 {
            self.result_regions.push(Arc::new(RwLock::new(SnapshotSubRegion::new_with_offset_and_range(
                self.snapshot_sub_region.read().unwrap().parent_region.clone(),
                self.run_length_encode_offset,
                self.run_length + (data_type_size - 1),
            ))));
            self.run_length_encode_offset += self.run_length;
            self.run_length = 0;
            self.is_encoding = false;
        }

        self.run_length_encode_offset += memory_alignment;
    }

    pub fn get_collected_regions(&self) -> &Vec<Arc<RwLock<SnapshotSubRegion>>> {
        return &self.result_regions;
    }
}
