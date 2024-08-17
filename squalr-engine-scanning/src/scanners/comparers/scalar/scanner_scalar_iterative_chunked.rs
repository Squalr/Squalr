use crate::scanners::comparers::snapshot_scanner::Scanner;
use crate::scanners::comparers::scalar::scanner_scalar_encoder::ScannerScalarEncoder;
use crate::scanners::constraints::scan_constraint::ScanConstraint;
use crate::snapshots::snapshot_region::SnapshotRegion;
use crate::snapshots::snapshot_sub_region::SnapshotSubRegion;
use rayon::prelude::*;
use std::sync::Once;

pub struct ScannerScalarIterativeChunked {
}

impl ScannerScalarIterativeChunked {
    fn new() -> Self {
        Self {
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
        let current_value_pointer = snapshot_region.get_sub_region_current_values_pointer(&snapshot_sub_region);
        let previous_value_pointer = snapshot_region.get_sub_region_previous_values_pointer(&snapshot_sub_region);
        let data_type_size = constraint.get_element_type().size_in_bytes();
        let alignment = constraint.get_alignment();
        let element_count = snapshot_sub_region.get_element_count(alignment, data_type_size) as usize;

        // Convert raw pointers to slices
        let current_values_slice = unsafe {
            std::slice::from_raw_parts(current_value_pointer, element_count * alignment as usize)
        };
        let previous_values_slice = unsafe {
            std::slice::from_raw_parts(previous_value_pointer, element_count * alignment as usize)
        };

        // Experimentally 1MB seemed to be the optimal chunk size on my CPU to keep all threads busy
        let chunk_size = 1 << 20;
        let num_chunks = (element_count + chunk_size - 1) / chunk_size;

        let all_subregions: Vec<SnapshotSubRegion> = (0..num_chunks)
            .into_par_iter()
            .flat_map_iter(|chunk_index| {
                let first_element_index = (chunk_index * chunk_size) as u64;
                let last_element_index = ((chunk_index + 1) * chunk_size).min(element_count) as u64;
                let chunk_address_offset = first_element_index * alignment as u64;
                let local_encoder = ScannerScalarEncoder::get_instance();
                let base_address = snapshot_sub_region.get_base_address() + chunk_address_offset;

                unsafe {
                    return local_encoder.encode(
                        current_values_slice.as_ptr().add(chunk_address_offset as usize),
                        previous_values_slice.as_ptr().add(chunk_address_offset as usize),
                        constraint,
                        base_address,
                        last_element_index - first_element_index,
                    );
                }
            })
            .collect();
        
        // TODO: Boundary merging on adjacent regions

        return all_subregions;
    }
}
