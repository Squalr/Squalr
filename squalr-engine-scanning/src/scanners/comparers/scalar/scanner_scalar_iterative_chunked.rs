use crate::scanners::comparers::scalar::scanner_scalar::ScannerScalar;
use crate::scanners::comparers::snapshot_scanner::Scanner;
use crate::scanners::comparers::snapshot_sub_region_run_length_encoder::SnapshotSubRegionRunLengthEncoder;
use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::snapshots::snapshot_region::SnapshotRegion;
use crate::snapshots::snapshot_sub_region::SnapshotSubRegion;
use rayon::prelude::*;
use std::borrow::BorrowMut;
use std::sync::{Arc, Once, RwLock};

pub struct ScannerScalarIterativeChunked {
    scalar_scanner: ScannerScalar,
}

impl ScannerScalarIterativeChunked {
    fn new() -> Self {
        Self {
            scalar_scanner: ScannerScalar::new(),
        }
    }
    
    pub fn get_instance() -> &'static ScannerScalarIterativeChunked {
        static mut INSTANCE: Option<ScannerScalarIterativeChunked> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = ScannerScalarIterativeChunked::new();
                INSTANCE = Some(instance);
            });

            return INSTANCE.as_ref().unwrap_unchecked();
        }
    }
}

impl Scanner for ScannerScalarIterativeChunked {

    /// Performs a parallel iteration over a region of memory, performing the scan comparison. A parallelized run-length encoding algorithm
    /// is used to generate new sub-regions as the scan progresses.
    /// 
    /// This is substantially faster than the sequential version, but requires a post-step of stitching together subregions that are adjacent.
    fn scan_region(&self,
        snapshot_region: &SnapshotRegion,
        snapshot_sub_region: &SnapshotSubRegion,
        constraint: &ScanConstraint
    ) -> Vec<SnapshotSubRegion> {
        let run_length_encoder = Arc::new(RwLock::new(SnapshotSubRegionRunLengthEncoder::new(snapshot_sub_region)));
        let current_value_pointer = snapshot_region.get_sub_region_current_values_pointer(&snapshot_sub_region);
        let previous_value_pointer = snapshot_region.get_sub_region_previous_values_pointer(&snapshot_sub_region);
        let data_type_size = constraint.get_element_type().size_in_bytes();
        let alignment = constraint.get_alignment();
        let aligned_element_count = snapshot_sub_region.get_element_count(alignment, data_type_size) as usize;

        // Convert raw pointers to slices
        let current_slice = unsafe {
            std::slice::from_raw_parts(current_value_pointer, aligned_element_count * alignment as usize)
        };
        let previous_slice = unsafe {
            std::slice::from_raw_parts(previous_value_pointer, aligned_element_count * alignment as usize)
        };

        // Experimentally 1MB seemed to be the optimal chunk size on my CPU to keep all threads busy
        let chunk_size = 1 << 20;
        let num_chunks = (aligned_element_count + chunk_size - 1) / chunk_size;

        (0..num_chunks).into_par_iter().for_each(|chunk_index| {
            let start_index = chunk_index * chunk_size;
            let end_index = ((chunk_index + 1) * chunk_size).min(aligned_element_count);
            let mut local_encoder = SnapshotSubRegionRunLengthEncoder::new(snapshot_sub_region);

            let constraint_value = constraint.get_constraint_value().unwrap();
            let mut current_value = constraint_value.clone();
            let mut previous_value = constraint_value.clone();
            let current_value = current_value.borrow_mut();
            let previous_value = previous_value.borrow_mut();

            if constraint.is_immediate_constraint() {
                let compare_func = self.scalar_scanner.get_immediate_compare_func(constraint.get_constraint_type());
    
                for index in start_index..end_index {
                    let current_value_pointer = &current_slice[index as usize * alignment as usize];

                    if compare_func(current_value_pointer, current_value, previous_value) {
                        local_encoder.encode_range(alignment as u64);
                    } else {
                        local_encoder.finalize_current_encode_unchecked(alignment as u64, data_type_size);
                    }
                }
            } else if constraint.is_relative_constraint() {
                let compare_func = self.scalar_scanner.get_relative_compare_func(constraint.get_constraint_type());
    
                for index in start_index..end_index {
                    let current_value_pointer = &current_slice[index as usize * alignment as usize];
                    let previous_value_pointer = &previous_slice[index as usize * alignment as usize];
                    
                    if compare_func(current_value_pointer, previous_value_pointer, current_value, previous_value) {
                        local_encoder.encode_range(alignment as u64);
                    } else {
                        local_encoder.finalize_current_encode_unchecked(alignment as u64, data_type_size);
                    }
                }
            } else if constraint.is_immediate_constraint() {
                let compare_func = self.scalar_scanner.get_relative_delta_compare_func(constraint.get_constraint_type());
                let delta_arg = constraint.get_constraint_delta_value().unwrap(); // TODO: Handle and complain
    
                for index in start_index..end_index {
                    let current_value_pointer = &current_slice[index as usize * alignment as usize];
                    let previous_value_pointer = &previous_slice[index as usize * alignment as usize];
                    
                    if compare_func(current_value_pointer, previous_value_pointer, current_value, previous_value, delta_arg) {
                        local_encoder.encode_range(alignment as u64);
                    } else {
                        local_encoder.finalize_current_encode_unchecked(alignment as u64, data_type_size);
                    }
                }
            } else {
                panic!("Unrecognized constraint");
            }
        
            local_encoder.finalize_current_encode_unchecked(0, data_type_size);

            // Merge parallel results
            run_length_encoder.write().unwrap().merge_from_other_encoder(&local_encoder);
        });

        let mut run_length_encoder = run_length_encoder.write().unwrap();

        run_length_encoder.combine_adjacent_sub_regions();

        return run_length_encoder.get_collected_regions().to_owned();
    }
}
