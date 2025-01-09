use squalr_engine_memory::memory_alignment::MemoryAlignment;
use squalr_engine_memory::normalized_region::NormalizedRegion;
use std::cmp::max;

/// Defines a range of filtered memory within a snapshot region. These filters are created by
/// scans to narrow down on the desired addresses.
#[derive(Debug)]
pub struct SnapshotRegionFilter {
    filter_range: NormalizedRegion,
}

impl SnapshotRegionFilter {
    pub fn new(
        base_address: u64,
        get_size_in_bytes: u64,
    ) -> Self {
        Self {
            filter_range: NormalizedRegion::new(base_address, get_size_in_bytes),
        }
    }

    pub fn get_base_address(&self) -> u64 {
        return self.filter_range.get_base_address();
    }

    pub fn set_base_address(
        &mut self,
        base_address: u64,
    ) {
        self.filter_range.set_base_address(base_address);
    }

    pub fn set_end_address(
        &mut self,
        end_address: u64,
    ) {
        self.filter_range.set_end_address(end_address);
    }

    pub fn get_end_address(&self) -> u64 {
        return self.filter_range.get_end_address();
    }

    pub fn get_region_size(&self) -> u64 {
        return self.filter_range.get_region_size();
    }

    pub fn get_element_count(
        &self,
        data_type_size: u64,
        memory_alignment: MemoryAlignment,
    ) -> u64 {
        let misalignment = self.get_misaligned_starting_byte_count(memory_alignment);
        let memory_alignment: u64 = max(memory_alignment as u64, 1);
        let trailing_bytes = data_type_size.saturating_sub(memory_alignment);
        let size_in_bytes = self.get_region_size();
        let effective_size_in_bytes = size_in_bytes.saturating_sub(trailing_bytes);

        // Check for things that have gone horribly wrong. None of these should ever happen. Happy debugging!
        debug_assert!(memory_alignment > 0);
        debug_assert!(misalignment == 0);
        debug_assert!(size_in_bytes >= data_type_size);
        debug_assert!(size_in_bytes >= trailing_bytes);

        return effective_size_in_bytes / memory_alignment;
    }

    fn get_misaligned_starting_byte_count(
        &self,
        alignment: MemoryAlignment,
    ) -> u64 {
        let alignment = max(alignment as u64, 1);
        let base_address = self.get_base_address();
        let misalignment = base_address % alignment;

        // Additional modulo to handle the case where misalignment is 0.
        return (alignment.saturating_sub(misalignment)) % alignment;
    }
}
