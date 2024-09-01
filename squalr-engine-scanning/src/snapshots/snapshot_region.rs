use crate::results::snapshot_region_filter::SnapshotRegionFilter;
use crate::results::snapshot_region_scan_results::SnapshotRegionScanResults;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use squalr_engine_common::values::data_type::DataType;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use squalr_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;
use squalr_engine_memory::memory_reader::MemoryReader;
use squalr_engine_memory::normalized_region::NormalizedRegion;

#[derive(Debug)]
pub struct SnapshotRegion {
    normalized_region: NormalizedRegion,
    current_values: Vec<u8>,
    previous_values: Vec<u8>,
    scan_results: SnapshotRegionScanResults,
    page_boundaries: Vec<u64>,
}

impl SnapshotRegion {
    pub fn new(
        normalized_region: NormalizedRegion,
        page_boundaries: Vec<u64>,
    ) -> Self {
        Self {
            normalized_region: normalized_region,
            current_values: vec![],
            previous_values: vec![],
            scan_results: SnapshotRegionScanResults::new(),
            page_boundaries: page_boundaries,
        }
    }

    pub fn get_current_values(&self) -> &Vec<u8> {
        return &self.current_values;
    }

    pub fn get_previous_values(&self) -> &Vec<u8> {
        return &self.previous_values;
    }

    pub fn read_all_memory(
        &mut self,
        process_handle: u64,
    ) -> Result<(), String> {
        let region_size = self.get_region_size() as usize;

        std::mem::swap(&mut self.current_values, &mut self.previous_values);

        if self.current_values.is_empty() && region_size > 0 {
            self.current_values = vec![0u8; region_size];
        }

        if self.page_boundaries.is_empty() {
            // If this snapshot is part of a standalone memory page, just read the regions as normal.
            MemoryReader::get_instance().read_bytes(process_handle, self.get_base_address(), &mut self.current_values);
        } else {
            // Otherwise, this snapshot is a merging of two or more OS regions, and special care is taken to separate the read calls.
            // This prevents any issues where 1 page deallocates.
            for (boundary_index, &boundary_address) in self.page_boundaries.iter().enumerate() {
                let next_boundary_address = if boundary_index + 1 < self.page_boundaries.len() {
                    self.page_boundaries[boundary_index + 1]
                } else {
                    self.get_end_address()
                };

                let read_size = (next_boundary_address - boundary_address) as usize;
                let offset = (boundary_address - self.get_base_address()) as usize;
                let current_values_slice = &mut self.current_values[offset..offset + read_size];
                MemoryReader::get_instance().read_bytes(process_handle, boundary_address, current_values_slice);
            }
        }

        return Ok(());
    }

    pub fn read_all_memory_parallel(
        &mut self,
        process_handle: u64,
    ) -> Result<(), String> {
        let region_size = self.get_region_size() as usize;
        let chunk_size = 2 << 23; // 16MB seems to be the optimal value for my CPU

        if region_size <= chunk_size {
            return self.read_all_memory(process_handle);
        }

        std::mem::swap(&mut self.current_values, &mut self.previous_values);

        if self.current_values.is_empty() && region_size > 0 {
            self.current_values = vec![0u8; region_size];
        }

        if self.page_boundaries.is_empty() {
            // No boundaries, read the entire region at once
            let mut chunks: Vec<_> = self.current_values.chunks_mut(chunk_size).collect();
            let base_address = self.normalized_region.get_base_address();

            chunks.par_iter_mut().enumerate().for_each(|(index, chunk)| {
                let offset = index * chunk_size;
                MemoryReader::get_instance().read_bytes(process_handle, base_address + offset as u64, chunk);
            });
        } else {
            for (boundary_index, &boundary_address) in self.page_boundaries.iter().enumerate() {
                let next_boundary_address = if boundary_index + 1 < self.page_boundaries.len() {
                    self.page_boundaries[boundary_index + 1]
                } else {
                    self.get_end_address()
                };

                let read_size = (next_boundary_address - boundary_address) as usize;
                let offset = (boundary_address - self.get_base_address()) as usize;
                let current_values_slice = &mut self.current_values[offset..offset + read_size];

                if read_size <= chunk_size {
                    MemoryReader::get_instance().read_bytes(process_handle, boundary_address, current_values_slice);
                } else {
                    // Parallel processing if the chunk size exceeds the defined optimal chunk size
                    let mut chunks: Vec<_> = current_values_slice.chunks_mut(chunk_size).collect();
                    let base_address = boundary_address;

                    chunks.par_iter_mut().enumerate().for_each(|(index, chunk)| {
                        let offset = index * chunk_size;
                        MemoryReader::get_instance().read_bytes(process_handle, base_address + offset as u64, chunk);
                    });
                }
            }
        }

        return Ok(());
    }

    pub fn get_current_values_pointer(
        &self,
        snapshot_region_filter: &SnapshotRegionFilter,
    ) -> *const u8 {
        unsafe {
            let offset = snapshot_region_filter.get_base_address() - self.get_base_address();
            return self.get_current_values().as_ptr().add(offset as usize);
        }
    }

    pub fn get_previous_values_pointer(
        &self,
        snapshot_region_filter: &SnapshotRegionFilter,
    ) -> *const u8 {
        unsafe {
            let offset = snapshot_region_filter.get_base_address() - self.get_base_address();
            return self.get_previous_values().as_ptr().add(offset as usize);
        }
    }

    pub fn get_base_address(&self) -> u64 {
        return self.normalized_region.get_base_address();
    }

    pub fn get_end_address(&self) -> u64 {
        return self.normalized_region.get_base_address() + self.normalized_region.get_region_size();
    }

    pub fn get_region_size(&self) -> u64 {
        return self.normalized_region.get_region_size();
    }

    pub fn set_alignment(
        &mut self,
        alignment: MemoryAlignment,
    ) {
        self.normalized_region.set_alignment(alignment);
    }

    pub fn has_current_values(&self) -> bool {
        return !self.current_values.is_empty();
    }

    pub fn has_previous_values(&self) -> bool {
        return !self.previous_values.is_empty();
    }

    pub fn can_compare_using_parameters(
        &self,
        scan_parameters: &ScanParameters,
    ) -> bool {
        if !scan_parameters.is_valid() || !self.has_current_values() {
            return false;
        }

        if !scan_parameters.is_immediate_comparison() && !self.has_previous_values() {
            return false;
        }

        return true;
    }

    pub fn get_scan_results(&self) -> &SnapshotRegionScanResults {
        return &self.scan_results;
    }

    pub fn set_scan_results(
        &mut self,
        scan_results: SnapshotRegionScanResults,
    ) {
        self.scan_results = scan_results;

        let original_base_address = self.get_base_address();
        let original_end_address = self.get_end_address();

        // Constrict this snapshot region based on the highest and lowest addresses in the contained scan result filters.
        if let Some((new_base_address, new_end_address)) = self
            .scan_results
            .get_filter_bounds(original_base_address, original_end_address)
        {
            let new_region_size = new_end_address - new_base_address;
            self.normalized_region.set_base_address(new_base_address);
            self.normalized_region.set_region_size(new_region_size);

            let start_offset = (new_base_address - original_base_address) as usize;
            let new_length = (new_end_address - new_base_address) as usize;

            if !self.current_values.is_empty() {
                self.current_values.drain(..start_offset);
                self.current_values.truncate(new_length);
            }

            if !self.previous_values.is_empty() {
                self.previous_values.drain(..start_offset);
                self.previous_values.truncate(new_length);
            }

            // Remove any page boundaries outside of the resized region
            self.page_boundaries
                .retain(|&boundary| boundary >= new_base_address && boundary <= new_end_address);

            self.scan_results.build_scan_results();
        } else {
            // No filters remaining! Set this regions size to 0 so that it can be cleaned up later.
            self.normalized_region.set_region_size(0);
        }
    }

    pub fn create_initial_scan_results(
        &mut self,
        data_types: &Vec<DataType>,
    ) {
        let base_address = self.get_base_address();
        let region_size = self.get_region_size();

        self.scan_results
            .create_initial_scan_results(base_address, region_size, data_types);
    }
}
