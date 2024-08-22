use crate::scanners::comparers::snapshot_sub_region_scanner::Scanner;
use crate::scanners::constraints::scan_parameters::{ScanParameters,ScanCompareType};
use crate::scanners::constraints::scan_parameterss::ScanParameterss;
use crate::snapshots::snapshot_sub_region::NormalizedRegion;
use squalr_engine_common::dynamic_struct::field_value::FieldValue;
use std::ops::{BitAnd, BitOr, BitXor};
use std::simd::cmp::{SimdOrd, SimdPartialEq, SimdPartialOrd};
use std::simd::{u8x16, u16x8, u32x4, u64x2, i8x16, i16x8, i32x4, i64x2, f32x4, f64x2};

pub struct SnapshotElementScannerVector {
    pub base_scanner: Scanner,
    pub vector_read_offset: u64,
    pub vector_read_base: u64,
    pub first_scan_vector_misalignment: u64,
    pub last_scan_vector_overread: u64,
}

impl SnapshotElementScannerVector {
    pub fn new() -> Self {
        Self {
            base_scanner: Scanner::new(),
            vector_read_offset: 0,
            vector_read_base: 0,
            first_scan_vector_misalignment: 0,
            last_scan_vector_overread: 0,
        }
    }

    pub fn initialize(&mut self, snapshot_sub_region: &Arc<RwLock<NormalizedRegion>>, constraints: &ScanParameters) {
        self.base_scanner.initialize(snapshot_sub_region, constraints);
        self.vector_read_offset = 0;
        self.vector_compare_func = Some(self.build_compare_actions(constraints));
        self.first_scan_vector_misalignment = self.calculate_first_scan_vector_misalignment();
        self.vector_read_base = snapshot_sub_region.region_offset - self.first_scan_vector_misalignment;
        self.last_scan_vector_overread = self.calculate_last_scan_vector_overread();
    }

    pub fn set_custom_compare_action(&mut self, custom_compare: Box<dyn Fn() -> u8x16 + 'a>) {
        self.custom_vector_compare = Some(custom_compare);
    }

    pub fn perform_vector_scan(
        &mut self,
        false_mask: u8x16,
        vector_increment_size: u64,
        vector_comparer: Box<dyn Fn() -> u8x16 + 'a>,
    ) -> Vec<NormalizedRegion> {
        // This algorithm has three stages:
        // 1) Scan the first vector of memory, which may contain elements outside of the intended range. For example, in <x, x, x, x ... y, y, y, y>,
        //      where x is data outside the element range (but within the snapshot region), and y is within the region we are scanning.
        //      to solve this, we mask out the x values such that these will always be considered false by our scan.
        // 2) Scan the middle parts of. These will all fit perfectly into vectors.
        // 3) Scan the final vector, if it exists. This may spill outside of the element range (but within the snapshot region).
        //      This works exactly like the first scan, but reversed. ie <y, y, y, y, ... x, x, x, x>, where x values are masked to be false.
        //      Note: This mask may also be applied to the first scan, if it is also the last scan (ie only 1 scan total for this region).
        
        let mut scan_count = (self.base_scanner.get_snapshot_sub_region().as_ref().unwrap().range / 16)
            + if self.last_scan_vector_overread > 0 { 1 } else { 0 };
        let misalignment_mask = self.build_vector_misalignment_mask();
        let overread_mask = self.build_vector_overread_mask();
        let mut run_length_encoded_scan_result;

        // Perform the first scan
        {
            run_length_encoded_scan_result = misalignment_mask.bitand(vector_comparer());
            run_length_encoded_scan_result =
                run_length_encoded_scan_result.bitand(if scan_count == 1 {
                    overread_mask
                } else {
                    u8x16::splat(0xFF)
                });
            self.base_scanner
                .get_run_length_encoder()
                .adjust_for_misalignment(self.first_scan_vector_misalignment);
            self.encode_scan_results(run_length_encoded_scan_result);
            self.vector_read_offset += vector_increment_size;
        }

        // Perform middle scans
        while self.vector_read_offset < self.base_scanner.get_snapshot_sub_region().as_ref().unwrap().range - 16 {
            run_length_encoded_scan_result = vector_comparer();
            self.encode_scan_results(run_length_encoded_scan_result);
            self.vector_read_offset += vector_increment_size;
        }

        // Perform final scan
        // TODO: Didn't the algorithm comments say that this could still be applied to a single scan? Revisit this.
        if scan_count > 1 {
            run_length_encoded_scan_result = overread_mask.bitand(vector_comparer());
            self.encode_scan_results_with_mask(run_length_encoded_scan_result, false_mask);
            self.vector_read_offset += vector_increment_size;
        }

        self.base_scanner.get_run_length_encoder().finalize_current_encode_unchecked(0);
        
        return self.base_scanner.get_run_length_encoder().get_collected_regions().clone();
    }

    fn build_vector_misalignment_mask(&self) -> u8x16 {
        let mut misalignment_mask = [0u8; 16];
        for i in self.first_scan_vector_misalignment..16 {
            misalignment_mask[i] = 0xFF;
        }
        u8x16::from_array(misalignment_mask)
    }
    
    fn build_vector_overread_mask(&self) -> u8x16 {
        let mut overread_mask = [0u8; 16];
        let fill_up_to = 16 - self.last_scan_vector_overread;
        for i in 0..fill_up_to {
            overread_mask[i] = 0xFF;
        }
        u8x16::from_array(overread_mask)
    }

    fn encode_scan_results(&mut self, scan_results: u8x16) {
        self.encode_scan_results_with_mask(scan_results, u8x16::splat(0));
    }

    fn encode_scan_results_with_mask(
        &mut self,
        scan_results: u8x16,
        false_mask: u8x16,
    ) {
        // Collect necessary information with immutable borrows
        let all_true = scan_results.simd_gt(false_mask).all(); //.to_array().iter().all(|&x| x);
        let all_false = scan_results.simd_eq(false_mask).all(); //.to_array().iter().all(|&x| x);
        let alignment = self.base_scanner.get_alignment() as u64;
    
        // Perform encoding with mutable borrows
        let encoder = self.base_scanner.get_run_length_encoder();
    
        if all_true {
            encoder.encode_range(16);
        } else if all_false {
            encoder.finalize_current_encode_unchecked(16);
        } else {
            for i in (0..16).step_by(alignment) {
                if scan_results[i] != false_mask[i] {
                    encoder.encode_range(alignment);
                } else {
                    encoder.finalize_current_encode_unchecked(alignment);
                }
            }
        }
    }
    

    fn calculate_first_scan_vector_misalignment(&self) -> u64 {
        let parent_region_size = self.base_scanner.get_snapshot_sub_region().as_ref().unwrap().parent_region.borrow().get_region_size();
        let range_region_offset = self.base_scanner.get_snapshot_sub_region().as_ref().unwrap().region_offset;
        let available_byte_count = parent_region_size - range_region_offset;
        let vector_spill_over = available_byte_count % 16;
        if vector_spill_over == 0 || range_region_offset + self.base_scanner.get_snapshot_sub_region().as_ref().unwrap().range + vector_spill_over < parent_region_size {
            0
        } else {
            16 - vector_spill_over
        }
    }

    fn calculate_last_scan_vector_overread(&self) -> u64 {
        let remaining_bytes = self.base_scanner.get_snapshot_sub_region().as_ref().unwrap().range % 16;
        if remaining_bytes == 0 {
            0
        } else {
            16 - remaining_bytes
        }
    }
}
