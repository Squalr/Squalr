use crate::scanners::comparers::scalar::scanner_scalar::ScannerScalar;
use crate::scanners::comparers::snapshot_scanner::Scanner;
use crate::scanners::comparers::snapshot_sub_region_run_length_encoder::SnapshotSubRegionRunLengthEncoder;
use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::snapshots::snapshot_sub_region::SnapshotSubRegion;
use rayon::prelude::*;
use std::sync::{Arc, Once, RwLock};

pub struct ScannerScalarIterativeParallel {
    scalar_scanner: ScannerScalar,
}

impl ScannerScalarIterativeParallel {
    fn new() -> Self {
        Self {
            scalar_scanner: ScannerScalar::new(),
        }
    }

    pub fn get_instance() -> Arc<RwLock<ScannerScalarIterativeParallel>> {
        static mut INSTANCE: Option<Arc<RwLock<ScannerScalarIterativeParallel>>> = None;
        static INIT: Once = Once::new();

        unsafe {
            INIT.call_once(|| {
                let instance = Arc::new(RwLock::new(ScannerScalarIterativeParallel::new()));
                INSTANCE = Some(instance);
            });

            return INSTANCE.as_ref().unwrap().clone();
        }
    }
}

impl Scanner for ScannerScalarIterativeParallel {
    fn scan_region(&self, snapshot_sub_region: &Arc<RwLock<SnapshotSubRegion>>, constraint: &ScanConstraint) -> Vec<Arc<RwLock<SnapshotSubRegion>>> {
        let run_length_encoder = Arc::new(RwLock::new(SnapshotSubRegionRunLengthEncoder::new(snapshot_sub_region.clone())));
        run_length_encoder.write().unwrap().initialize();

        let current_value_pointer = snapshot_sub_region.read().unwrap().get_current_values_pointer();
        let previous_value_pointer = snapshot_sub_region.read().unwrap().get_previous_values_pointer();
        let data_type = constraint.get_element_type();
        let alignment = constraint.get_alignment();
        let aligned_element_count = snapshot_sub_region.read().unwrap().get_element_count(alignment, data_type.size_in_bytes());

        // Convert raw pointers to slices
        let current_slice = unsafe {
            std::slice::from_raw_parts(current_value_pointer, aligned_element_count as usize * alignment as usize)
        };
        let previous_slice = unsafe {
            std::slice::from_raw_parts(previous_value_pointer, aligned_element_count as usize * alignment as usize)
        };

        // Experimentally 1MB seemed to be the optimal chunk size on my CPU to keep all threads busy
        let chunk_size = 1 << 20;
        let num_chunks = (aligned_element_count + chunk_size - 1) / chunk_size;

        (0..num_chunks).into_par_iter().for_each(|chunk_index| {
            let start_index = chunk_index * chunk_size;
            let end_index = ((chunk_index + 1) * chunk_size).min(aligned_element_count);

            let local_encoder = Arc::new(RwLock::new(SnapshotSubRegionRunLengthEncoder::new(snapshot_sub_region.clone())));

            for i in start_index..end_index {
                let current_offset = &current_slice[i as usize * alignment as usize];
                let previous_offset = &previous_slice[i as usize * alignment as usize];

                if self.scalar_scanner.do_compare_action(current_offset, previous_offset, &constraint, &data_type) {
                    local_encoder.write().unwrap().encode_range(alignment as usize);
                } else {
                    local_encoder.write().unwrap().finalize_current_encode_unchecked(alignment as usize, data_type.size_in_bytes());
                }
            }

            let mut global_encoder = run_length_encoder.write().unwrap();
            let local_encoder = local_encoder.write().unwrap();
            
            // Manually merge results
            global_encoder.merge_from_other_encoder(&*local_encoder);
        });

        run_length_encoder.write().unwrap().finalize_current_encode_unchecked(0, data_type.size_in_bytes());

        return run_length_encoder.write().unwrap().get_collected_regions().to_owned();
    }
}
