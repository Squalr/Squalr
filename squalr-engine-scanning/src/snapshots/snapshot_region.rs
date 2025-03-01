use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::scanners::parameters::scan_parameters::ScanParameters;
use squalr_engine_common::structures::process_info::OpenedProcessInfo;
use squalr_engine_memory::memory_reader::MemoryReader;
use squalr_engine_memory::memory_reader::memory_reader_trait::IMemoryReader;
use squalr_engine_memory::normalized_region::NormalizedRegion;

/// Defines a contiguous region of memory within a snapshot.
pub struct SnapshotRegion {
    /// The underlying region that contains the start address and length of this snapshot.
    normalized_region: NormalizedRegion,

    /// The most recent values collected from memory within this snapshot region bounds.
    current_values: Vec<u8>,

    /// The prior values collected from memory within this snapshot region bounds.
    previous_values: Vec<u8>,

    /// Any OS level page boundaries that may sub-divide this snapshot region.
    page_boundaries: Vec<u64>,
}

impl SnapshotRegion {
    pub fn new(
        normalized_region: NormalizedRegion,
        page_boundaries: Vec<u64>,
    ) -> Self {
        Self {
            normalized_region,
            current_values: vec![],
            previous_values: vec![],
            page_boundaries,
        }
    }

    /// Gets the most recent values collected from memory within this snapshot region bounds.
    pub fn get_current_values(&self) -> &Vec<u8> {
        &self.current_values
    }

    /// Gets the prior values collected from memory within this snapshot region bounds.
    pub fn get_previous_values(&self) -> &Vec<u8> {
        &self.previous_values
    }

    pub fn read_all_memory(
        &mut self,
        process_info: &OpenedProcessInfo,
    ) -> Result<(), String> {
        self.resize_to_filters();

        let region_size = self.get_region_size() as usize;

        std::mem::swap(&mut self.current_values, &mut self.previous_values);

        if self.current_values.is_empty() && region_size > 0 {
            self.current_values = vec![0u8; region_size];
        }

        if self.page_boundaries.is_empty() {
            // If this snapshot is part of a standalone memory page, just read the regions as normal.
            MemoryReader::get_instance().read_bytes(&process_info, self.get_base_address(), &mut self.current_values);
        } else {
            // Otherwise, this snapshot is a merging of two or more OS regions, and special care is taken to separate the read calls.
            // This prevents the case where one page deallocates, causing the read for both to fail.
            for (boundary_index, &boundary_address) in self.page_boundaries.iter().enumerate() {
                let next_boundary_address = if boundary_index + 1 < self.page_boundaries.len() {
                    self.page_boundaries[boundary_index + 1]
                } else {
                    self.get_end_address()
                };

                let read_size = (next_boundary_address - boundary_address) as usize;
                let offset = (boundary_address - self.get_base_address()) as usize;
                let current_values_slice = &mut self.current_values[offset..offset + read_size];
                MemoryReader::get_instance().read_bytes(&process_info, boundary_address, current_values_slice);
            }
        }

        Ok(())
    }

    pub fn get_current_values_pointer(
        &self,
        snapshot_region_filter: &SnapshotRegionFilter,
    ) -> *const u8 {
        unsafe {
            let offset = snapshot_region_filter.get_base_address() - self.get_base_address();
            let ptr = self.get_current_values().as_ptr().add(offset as usize);

            ptr
        }
    }

    pub fn get_previous_values_pointer(
        &self,
        snapshot_region_filter: &SnapshotRegionFilter,
    ) -> *const u8 {
        unsafe {
            let offset = snapshot_region_filter.get_base_address() - self.get_base_address();
            let ptr = self.get_previous_values().as_ptr().add(offset as usize);

            ptr
        }
    }

    pub fn get_base_address(&self) -> u64 {
        self.normalized_region.get_base_address()
    }

    pub fn get_end_address(&self) -> u64 {
        self.normalized_region.get_base_address() + self.normalized_region.get_region_size()
    }

    pub fn get_region_size(&self) -> u64 {
        self.normalized_region.get_region_size()
    }

    pub fn has_current_values(&self) -> bool {
        !self.current_values.is_empty()
    }

    pub fn has_previous_values(&self) -> bool {
        !self.previous_values.is_empty()
    }

    pub fn can_compare_using_parameters(
        &self,
        scan_parameters: &ScanParameters,
    ) -> bool {
        if !scan_parameters.is_valid() || !self.has_current_values() {
            false
        } else if !scan_parameters.is_immediate_comparison() && !self.has_previous_values() {
            false
        } else {
            true
        }
    }

    /// TODO: This probably needs to be a more robust function that handles large gaps too.
    /// The concept is only partially correct -- limit the size of the region to the bounds of provided scan results.
    /// The problem of course is if there are scan results at the ends of the region, and not in the middle.
    /// There is likely some profiling to be done, where we decide if:
    /// A) Any inner page boundaries are eclipsed. Okay, remove those. Easy.
    /// B) Decide if its worth sharding a fragmented region into two regions. Less easy.
    pub fn resize_to_filters(&mut self) {
        let original_base_address = self.get_base_address();

        /*
        // Constrict this snapshot region based on the highest and lowest addresses in the contained scan result filters.
        for scan_results in self.region_scan_results_by_data_type.iter() {
            let scan_results = scan_results.value();
            let (filter_lowest_address, filter_highest_address) = scan_results.get_filter_bounds();

            let new_region_size = filter_highest_address - filter_lowest_address;

            // No filters remaining! Set this regions size to 0 so that it can be cleaned up later.
            if new_region_size <= 0 {
                self.normalized_region.set_region_size(0);
                self.page_boundaries.clear();
                return;
            }

            self.normalized_region.set_base_address(filter_lowest_address);
            self.normalized_region.set_region_size(new_region_size);

            let start_offset = (filter_lowest_address - original_base_address) as usize;
            let new_length = (filter_highest_address - filter_lowest_address) as usize;

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
                .retain(|&boundary| boundary >= filter_lowest_address && boundary <= filter_highest_address);
        } */
    }
}
