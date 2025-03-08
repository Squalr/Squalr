use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
use crate::results::snapshot_region_scan_results::SnapshotRegionScanResults;
use rayon::iter::IntoParallelIterator;
use rayon::iter::ParallelIterator;
use squalr_engine_api::structures::data_types::data_type_ref::DataTypeRef;
use squalr_engine_api::structures::data_values::data_value::DataValue;
use squalr_engine_api::structures::processes::process_info::OpenedProcessInfo;
use squalr_engine_api::structures::scanning::comparisons::scan_compare_type::ScanCompareType;
use squalr_engine_api::structures::scanning::scan_parameters_global::ScanParametersGlobal;
use squalr_engine_api::structures::scanning::scan_parameters_local::ScanParametersLocal;
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

    /// The current scan results on this snapshot region.
    scan_results: SnapshotRegionScanResults,
}

impl SnapshotRegion {
    pub fn new(
        normalized_region: NormalizedRegion,
        page_boundaries: Vec<u64>,
        scan_parameters_local_collection: &Vec<ScanParametersLocal>,
    ) -> Self {
        // Create an initial filter, spanning the entire region, for each data type that the scan results will represent.
        let scan_filter_collections = scan_parameters_local_collection
            .iter()
            .map(|scan_parameters_local| {
                let initial_filter = vec![vec![SnapshotRegionFilter::new(
                    normalized_region.get_base_address(),
                    normalized_region.get_region_size(),
                )]];

                SnapshotRegionFilterCollection::new(initial_filter, scan_parameters_local.clone())
            })
            .collect();

        Self {
            normalized_region,
            current_values: vec![],
            previous_values: vec![],
            page_boundaries,
            scan_results: SnapshotRegionScanResults::new(scan_filter_collections),
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

    /// Gets the most recent values collected from memory within this snapshot region bounds.
    pub fn get_current_value(
        &self,
        element_address: u64,
        data_type: &DataTypeRef,
    ) -> Option<DataValue> {
        let byte_offset: u64 = element_address.saturating_sub(self.get_base_address());
        let data_type_size = data_type.get_size_in_bytes();

        if byte_offset.saturating_add(data_type_size) <= self.current_values.len() as u64 {
            if let Some(mut data_value) = data_type.get_default_value() {
                let start = byte_offset as usize;
                let end = start + data_type_size as usize;
                data_value.copy_from_bytes(&self.current_values[start..end]);

                Some(data_value)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Gets the prior values collected from memory within this snapshot region bounds.
    pub fn get_previous_value(
        &self,
        element_address: u64,
        data_type: &DataTypeRef,
    ) -> Option<DataValue> {
        let byte_offset: u64 = element_address.saturating_sub(self.get_base_address());
        let data_type_size = data_type.get_size_in_bytes();

        if byte_offset.saturating_add(data_type_size) <= self.previous_values.len() as u64 {
            if let Some(mut data_value) = data_type.get_default_value() {
                let start = byte_offset as usize;
                let end = start + data_type_size as usize;
                data_value.copy_from_bytes(&self.previous_values[start..end]);

                Some(data_value)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_current_values_filter_pointer(
        &self,
        snapshot_region_filter: &SnapshotRegionFilter,
    ) -> *const u8 {
        unsafe {
            let filter_base_address = snapshot_region_filter.get_base_address();
            let offset = filter_base_address.saturating_sub(self.get_base_address());
            let ptr = self.get_current_values().as_ptr().add(offset as usize);

            ptr
        }
    }

    pub fn get_previous_values_filter_pointer(
        &self,
        snapshot_region_filter: &SnapshotRegionFilter,
    ) -> *const u8 {
        unsafe {
            let filter_base_address = snapshot_region_filter.get_base_address();
            let offset = filter_base_address.saturating_sub(self.get_base_address());
            let ptr = self.get_previous_values().as_ptr().add(offset as usize);

            ptr
        }
    }

    /// Reads all memory for this snapshot region, updating the current and previous value arrays.
    pub fn read_all_memory(
        &mut self,
        process_info: &OpenedProcessInfo,
    ) -> Result<(), String> {
        let region_size = self.get_region_size() as usize;

        // Move current_values to be the previous_values. This is a very efficient way to move these, as instead of
        // discarding the old previous values, we recycle that array for use in the next scan to create new current_values.
        std::mem::swap(&mut self.current_values, &mut self.previous_values);

        // Create current values vector if none exist.
        if self.current_values.is_empty() && region_size > 0 {
            self.current_values = vec![0u8; region_size];
        }

        if self.page_boundaries.is_empty() {
            // If this snapshot is part of a standalone memory page, just read the regions as normal.
            MemoryReader::get_instance().read_bytes(&process_info, self.get_base_address(), &mut self.current_values);
        } else {
            // Otherwise, this snapshot is a merging of two or more OS regions, and special care is taken to separate the read calls.
            // This prevents the case where one page deallocates, causing the read for both to fail.
            // Additionally, we read these chunks of memory in parallel, as they may be quite large due to our merging.
            let mut read_ranges = Vec::with_capacity(self.page_boundaries.len() + 1);
            let mut next_range_start_address = self.get_base_address();
            let mut current_slice = self.current_values.as_mut_slice();

            // Iterate the page boundaries and pull out non-overlapping mutable slices to satisfy the Rust borrow checker.
            for &next_boundary_address in &self.page_boundaries {
                let range_size = next_boundary_address.saturating_sub(next_range_start_address) as usize;
                let (slice, remaining) = current_slice.split_at_mut(range_size);

                read_ranges.push((next_range_start_address, slice));
                current_slice = remaining;
                next_range_start_address = next_boundary_address;
            }

            // Last slice after final boundary.
            if !current_slice.is_empty() {
                read_ranges.push((next_range_start_address, current_slice));
            }

            // And finally parallel read using the obtained non-overlapping mutable slices.
            read_ranges.into_par_iter().for_each(|(address, buffer)| {
                MemoryReader::get_instance().read_bytes(process_info, address, buffer);
            });
        }

        Ok(())
    }

    pub fn get_base_address(&self) -> u64 {
        self.normalized_region.get_base_address()
    }

    pub fn get_end_address(&self) -> u64 {
        self.normalized_region
            .get_base_address()
            .saturating_add(self.normalized_region.get_region_size())
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
        scan_parameters_global: &ScanParametersGlobal,
    ) -> bool {
        if !scan_parameters_global.is_valid() || !self.has_current_values() {
            false
        } else {
            match scan_parameters_global.get_compare_type() {
                ScanCompareType::Immediate(_) => true,
                ScanCompareType::Relative(_) | ScanCompareType::Delta(_) => self.has_previous_values(),
            }
        }
    }

    pub fn get_scan_results(&self) -> &SnapshotRegionScanResults {
        &self.scan_results
    }

    pub fn set_scan_results(
        &mut self,
        scan_results: SnapshotRegionScanResults,
    ) {
        self.scan_results = scan_results;

        // Upon assigning new scan results, we want to cull memory outside of the bounds of the filters.
        self.resize_to_filters();
    }

    /// Constrict this snapshot region based on the highest and lowest addresses in the contained scan result filters.
    /// JIRA: Shard large gaps into multiple regions?
    fn resize_to_filters(&mut self) {
        let (filter_lowest_address, filter_highest_address) = self.scan_results.get_filter_bounds();
        let original_base_address = self.get_base_address();
        let new_region_size = filter_highest_address.saturating_sub(filter_lowest_address);

        // No filters remaining! Set this regions size to 0 so that it can be cleaned up later.
        if new_region_size <= 0 {
            self.current_values = vec![];
            self.previous_values = vec![];
            self.page_boundaries = vec![];
            self.normalized_region.set_region_size(0);
            return;
        }

        self.normalized_region.set_base_address(filter_lowest_address);
        self.normalized_region.set_region_size(new_region_size);

        let start_offset = (filter_lowest_address.saturating_sub(original_base_address)) as usize;

        if !self.current_values.is_empty() {
            self.current_values.drain(..start_offset);
            self.current_values.truncate(new_region_size as usize);
        }

        if !self.previous_values.is_empty() {
            self.previous_values.drain(..start_offset);
            self.previous_values.truncate(new_region_size as usize);
        }

        // Remove any page boundaries outside of the resized region
        self.page_boundaries
            .retain(|&boundary| boundary >= filter_lowest_address && boundary <= filter_highest_address);
    }
}
