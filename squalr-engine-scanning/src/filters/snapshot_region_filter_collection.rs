use crate::filters::snapshot_region_filter::SnapshotRegionFilter;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use squalr_engine_common::{structures::memory_alignment::MemoryAlignment, values::data_type::DataType};

/// A custom type that defines a set of filters (scan results) discovered by scanners.
pub struct SnapshotRegionFilterCollection {
    /// The filters contained in this collection. This is kept as a vector of vectors for better parallelization.
    snapshot_region_filters: Vec<Vec<SnapshotRegionFilter>>,

    /// The data type of all elements in this filter.
    data_type: DataType,

    // The memory alignment of all elements in this filter.
    memory_alignment: MemoryAlignment,
}

impl SnapshotRegionFilterCollection {
    /// Creates a new collection of filters over a snapshot region,
    /// representing regions of memory with the specified data type and alignment.
    pub fn new(
        mut snapshot_region_filters: Vec<Vec<SnapshotRegionFilter>>,
        data_type: DataType,
        memory_alignment: MemoryAlignment,
    ) -> Self {
        // Sort each inner vector by base address.
        // JIRA: This data is likely already sorted. Should we just cut this?
        for filters in &mut snapshot_region_filters {
            filters.sort_by_key(|filter| filter.get_base_address());
        }

        // Sort the outer vector by the base address of the first element in each inner vector.
        snapshot_region_filters.sort_by_key(|filters| {
            filters
                .first()
                .map(|filter| filter.get_base_address())
                .unwrap_or(u64::MAX)
        });

        Self {
            snapshot_region_filters,
            data_type,
            memory_alignment,
        }
    }

    /// Gets the minimum address across all filters contained by this filter collection.
    /// This is O(1), as the filters are sorted upon creation of the filter collection.
    pub fn get_filter_minimum_address(&self) -> u64 {
        let min_address = self
            .snapshot_region_filters
            .first()
            .and_then(|filters| filters.first())
            .map_or(0, |filter| filter.get_base_address());

        min_address
    }

    /// Gets the maximum address across all filters contained by this filter collection.
    /// This is O(1), as the filters are sorted upon creation of the filter collection.
    pub fn get_filter_maximum_address(&self) -> u64 {
        let max_address = self
            .snapshot_region_filters
            .last()
            .and_then(|filters| filters.last())
            .map_or(0, |filter| filter.get_base_address());

        max_address
    }

    pub fn get_data_type(&self) -> &DataType {
        &self.data_type
    }

    pub fn get_memory_alignment(&self) -> MemoryAlignment {
        self.memory_alignment.clone()
    }

    pub fn iter(&self) -> std::iter::Flatten<std::slice::Iter<'_, Vec<SnapshotRegionFilter>>> {
        self.snapshot_region_filters.iter().flatten()
    }

    pub fn par_iter(&self) -> rayon::iter::Flatten<rayon::slice::Iter<'_, Vec<SnapshotRegionFilter>>> {
        self.snapshot_region_filters.par_iter().flatten()
    }
}
