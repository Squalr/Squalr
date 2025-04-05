use crate::structures::memory::memory_alignment::MemoryAlignment;
use crate::structures::scanning::parameters::user_scan_parameters_local::UserScanParametersLocal;
use crate::structures::{data_types::data_type_ref::DataTypeRef, scanning::filters::snapshot_region_filter::SnapshotRegionFilter};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

/// A custom type that defines a set of filters (scan results) discovered by scanners.
pub struct SnapshotRegionFilterCollection {
    /// The filters contained in this collection. This is kept as a vector of vectors for better parallelization.
    snapshot_region_filters: Vec<Vec<SnapshotRegionFilter>>,

    // The data type and memory alignment of all elements in this filter.
    user_scan_parameters_local: UserScanParametersLocal,

    // The total number of results contained in this collection.
    number_of_results: u64,
}

impl SnapshotRegionFilterCollection {
    /// Creates a new collection of filters over a snapshot region,
    /// representing regions of memory with the specified data type and alignment.
    pub fn new(
        mut snapshot_region_filters: Vec<Vec<SnapshotRegionFilter>>,
        user_scan_parameters_local: UserScanParametersLocal,
    ) -> Self {
        // Sort each inner vector by base address.
        // JIRA: This data is likely already sorted. Should we just cut this?
        for filters in &mut snapshot_region_filters {
            filters.sort_by_key(|filter| filter.get_base_address());
        }

        // Sort the outer vector by the base address of the first element in each inner vector.
        // JIRA: Cut this if we don't need it in our scan results querying.
        snapshot_region_filters.sort_by_key(|filters| {
            filters
                .first()
                .map(|filter| filter.get_base_address())
                .unwrap_or(u64::MAX)
        });

        let data_type = user_scan_parameters_local.get_data_type();
        let memory_alignment = user_scan_parameters_local.get_memory_alignment_or_default();
        let number_of_results = snapshot_region_filters
            .iter()
            .flatten()
            .map(|filter| filter.get_element_count(&data_type, memory_alignment))
            .sum();

        Self {
            snapshot_region_filters,
            number_of_results,
            user_scan_parameters_local,
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
            .map_or(0, |filter| filter.get_end_address());

        max_address
    }

    // Get the total number of results contained in this collection.
    pub fn get_number_of_results(&self) -> u64 {
        self.number_of_results
    }

    /// Gets the scan filter parameters of this snapshot region filter collection.
    pub fn get_user_scan_parameters_local(&self) -> &UserScanParametersLocal {
        &self.user_scan_parameters_local
    }

    /// Gets the data type of this snapshot region filter collection.
    pub fn get_data_type(&self) -> &DataTypeRef {
        &self.user_scan_parameters_local.get_data_type()
    }

    /// Gets the memory alignment of this snapshot region filter collection.
    pub fn get_memory_alignment(&self) -> MemoryAlignment {
        self.user_scan_parameters_local
            .get_memory_alignment_or_default()
    }

    /// Iterates the snapshot region filters sequentially, which are sorted by base address ascending.
    pub fn iter(&self) -> std::iter::Flatten<std::slice::Iter<'_, Vec<SnapshotRegionFilter>>> {
        self.snapshot_region_filters.iter().flatten()
    }

    /// Iterates the snapshot region filters in parallel, which are sorted by base address ascending.
    pub fn par_iter(&self) -> rayon::iter::Flatten<rayon::slice::Iter<'_, Vec<SnapshotRegionFilter>>> {
        self.snapshot_region_filters.par_iter().flatten()
    }
}
