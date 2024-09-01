use crate::results::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::encoders::scalar::scanner_scalar_encoder::ScannerScalarEncoder;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use crate::scanners::snapshot_scanner::Scanner;
use crate::snapshots::snapshot_region::SnapshotRegion;
use rayon::prelude::*;
use squalr_engine_common::values::data_type::DataType;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::sync::Once;

pub struct ScannerScalarIterativeChunked {}

impl ScannerScalarIterativeChunked {
    fn new() -> Self {
        Self {}
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
    unsafe fn scan_region(
        &self,
        snapshot_region: &SnapshotRegion,
        snapshot_region_filter: &SnapshotRegionFilter,
        scan_parameters: &ScanParameters,
        data_type: &DataType,
        memory_alignment: MemoryAlignment,
    ) -> Vec<SnapshotRegionFilter> {
        let current_value_pointer = snapshot_region.get_current_values_pointer(&snapshot_region_filter);
        let previous_value_pointer = snapshot_region.get_previous_values_pointer(&snapshot_region_filter);
        let data_type_size = data_type.get_size_in_bytes();
        let element_count = snapshot_region_filter.get_element_count(data_type_size, memory_alignment) as usize;

        // Convert raw pointers to slices
        let current_values_slice = unsafe { std::slice::from_raw_parts(current_value_pointer, element_count * memory_alignment as usize) };
        let previous_values_slice = unsafe { std::slice::from_raw_parts(previous_value_pointer, element_count * memory_alignment as usize) };

        // Experimentally 1MB seemed to be the optimal chunk size on my CPU to keep all threads busy
        let chunk_size = 1 << 20;
        let num_chunks = (element_count + chunk_size - 1) / chunk_size;

        let mut results: Vec<SnapshotRegionFilter> = (0..num_chunks)
            .into_par_iter()
            .map(|chunk_index| {
                let first_element_index = (chunk_index * chunk_size) as u64;
                let last_element_index = ((chunk_index + 1) * chunk_size).min(element_count) as u64;
                let chunk_address_offset = first_element_index * memory_alignment as u64;
                let local_encoder = ScannerScalarEncoder::get_instance();
                let base_address = snapshot_region_filter.get_base_address() + chunk_address_offset;

                unsafe {
                    return local_encoder.encode(
                        current_values_slice.as_ptr().add(chunk_address_offset as usize),
                        previous_values_slice
                            .as_ptr()
                            .add(chunk_address_offset as usize),
                        scan_parameters,
                        data_type,
                        memory_alignment,
                        base_address,
                        last_element_index - first_element_index,
                    );
                }
            })
            // Build the final vector of all filters in parallel.
            .reduce_with(|mut collection_a, collection_b| {
                collection_a.extend(collection_b);
                return collection_a;
            })
            .unwrap_or_else(Vec::new);

        // Merge adjacent regions directly within the results vector to avoid unecessary reallocations.
        if !results.is_empty() {
            // Ensure that filters are sorted by base address ascending.
            results.sort_by(|a, b| a.get_base_address().cmp(&b.get_base_address()));

            let mut filter_index = 0;
            while filter_index < results.len() - 1 {
                let (left, right) = results.split_at_mut(filter_index + 1);
                let current_region = &mut left[filter_index];
                let next_region = &right[0];

                if current_region.get_end_address() == next_region.get_base_address() {
                    current_region.set_end_address(next_region.get_end_address());
                    // Remove the next region as it has been merged.
                    results.remove(filter_index + 1);
                } else {
                    filter_index += 1;
                }
            }
        }

        return results;
    }
}
