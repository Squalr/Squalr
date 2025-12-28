use crate::{
    registries::symbols::symbol_registry::SymbolRegistry,
    structures::{
        data_types::data_type_ref::DataTypeRef,
        memory::{memory_alignment::MemoryAlignment, normalized_region::NormalizedRegion},
    },
};
use std::cmp::max;

/// Defines a range of filtered memory within a snapshot region. These filters are created by
/// scans to narrow down on a set of desired addresses within the parent snapshot region.
#[derive(Clone)]
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

    /// Gets the base/start address of this filter.
    pub fn get_base_address(&self) -> u64 {
        self.filter_range.get_base_address()
    }

    /// Sets the base/start address of this filter.
    pub fn set_base_address(
        &mut self,
        base_address: u64,
    ) {
        self.filter_range.set_base_address(base_address);
    }

    /// Sets the end address of this filter.
    pub fn set_end_address(
        &mut self,
        end_address: u64,
    ) {
        self.filter_range.set_end_address(end_address);
    }

    /// Gets the end address of this filter.
    pub fn get_end_address(&self) -> u64 {
        self.filter_range.get_end_address()
    }

    /// Gets the size of this filter.
    pub fn get_region_size(&self) -> u64 {
        self.filter_range.get_region_size()
    }

    /// Gets the number of elements contained by this filter for the given data type and alignment.
    pub fn get_element_count(
        &self,
        data_type_ref: &DataTypeRef,
        memory_alignment: MemoryAlignment,
    ) -> u64 {
        let data_type_size_bytes = SymbolRegistry::get_instance().get_unit_size_in_bytes(data_type_ref);
        let misalignment = self.get_misaligned_starting_byte_count(memory_alignment);
        let memory_alignment: u64 = max(memory_alignment as u64, 1);
        let trailing_bytes = data_type_size_bytes.saturating_sub(memory_alignment);
        let size_in_bytes = self.get_region_size();
        let effective_size_in_bytes = size_in_bytes.saturating_sub(trailing_bytes);

        // Check for things that have gone horribly wrong. None of these should ever happen. Happy debugging!
        debug_assert!(memory_alignment > 0);
        debug_assert!(misalignment == 0);
        debug_assert!(size_in_bytes >= data_type_size_bytes);
        debug_assert!(size_in_bytes >= trailing_bytes);

        effective_size_in_bytes / memory_alignment
    }

    /// Gets the number of misaligned bytes at the base address for this region. This should always
    /// be zero. For instance, an alignment of 4 should always have a base address ending in 0, 4, 8, or C.
    /// Any other values would produce a non-zero misalignment, and would be evidence of something gone wrong.
    fn get_misaligned_starting_byte_count(
        &self,
        alignment: MemoryAlignment,
    ) -> u64 {
        let alignment = max(alignment as u64, 1);
        let base_address = self.get_base_address();
        let misalignment = base_address % alignment;

        // Additional modulo to handle the case where misalignment is 0.
        (alignment.saturating_sub(misalignment)) % alignment
    }
}

impl PartialEq for SnapshotRegionFilter {
    fn eq(
        &self,
        other: &Self,
    ) -> bool {
        self.filter_range == other.filter_range
    }
}
