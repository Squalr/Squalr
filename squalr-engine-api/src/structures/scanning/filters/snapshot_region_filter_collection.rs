use crate::registries::symbols::symbol_registry::SymbolRegistry;
use crate::structures::data_values::container_type::ContainerType;
use crate::structures::memory::memory_alignment::MemoryAlignment;
use crate::structures::{data_types::data_type_ref::DataTypeRef, scanning::filters::snapshot_region_filter::SnapshotRegionFilter};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

/// A custom type that defines a set of filters (scan results) discovered by scanners.
#[derive(Clone)]
pub struct SnapshotRegionFilterCollection {
    /// The filters contained in this collection. This is kept as a vector of vectors for better parallelization.
    snapshot_region_filters: Vec<Vec<SnapshotRegionFilter>>,

    // The data type of all elements in this filter.
    data_type_ref: DataTypeRef,

    // The memory alignment of all elements in this filter.
    memory_alignment: MemoryAlignment,

    // The width in bytes of each logical result represented by this collection.
    result_value_size_in_bytes: u64,

    // The logical container semantics of each result represented by this collection.
    result_container_type: ContainerType,

    // The total number of results contained in this collection.
    number_of_results: u64,
}

impl SnapshotRegionFilterCollection {
    /// Creates a new collection of filters over a snapshot region,
    /// representing regions of memory with the specified data type and alignment.
    pub fn new(
        symbol_registry: &SymbolRegistry,
        snapshot_region_filters: Vec<Vec<SnapshotRegionFilter>>,
        data_type_ref: DataTypeRef,
        memory_alignment: MemoryAlignment,
    ) -> Self {
        let result_value_size_in_bytes = symbol_registry.get_unit_size_in_bytes(&data_type_ref);

        Self::new_with_result_size(
            symbol_registry,
            snapshot_region_filters,
            data_type_ref,
            memory_alignment,
            result_value_size_in_bytes,
        )
    }

    pub fn new_with_result_size(
        _symbol_registry: &SymbolRegistry,
        mut snapshot_region_filters: Vec<Vec<SnapshotRegionFilter>>,
        data_type_ref: DataTypeRef,
        memory_alignment: MemoryAlignment,
        result_value_size_in_bytes: u64,
    ) -> Self {
        // Some scanner paths can emit trailing windows that are smaller than the logical result width.
        // These windows cannot materialize any results, so drop them before sorting/counting.
        for filters in &mut snapshot_region_filters {
            filters.retain(|filter| filter.get_region_size() >= result_value_size_in_bytes);
        }

        snapshot_region_filters.retain(|filters| !filters.is_empty());

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

        let number_of_results = snapshot_region_filters
            .iter()
            .flatten()
            .map(|filter| filter.get_element_count(result_value_size_in_bytes, memory_alignment))
            .sum();

        Self {
            snapshot_region_filters,
            number_of_results,
            data_type_ref,
            memory_alignment,
            result_value_size_in_bytes,
            result_container_type: ContainerType::None,
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

    /// Gets the data type of this snapshot region filter collection.
    pub fn get_data_type_ref(&self) -> &DataTypeRef {
        &self.data_type_ref
    }

    /// Gets the memory alignment of this snapshot region filter collection.
    pub fn get_memory_alignment(&self) -> MemoryAlignment {
        self.memory_alignment
    }

    pub fn get_result_value_size_in_bytes(&self) -> u64 {
        self.result_value_size_in_bytes
    }

    pub fn get_result_container_type(&self) -> ContainerType {
        self.result_container_type
    }

    pub fn with_result_container_type(
        mut self,
        result_container_type: ContainerType,
    ) -> Self {
        self.result_container_type = result_container_type;
        self
    }

    /// Iterates the snapshot region filters sequentially, which are sorted by base address ascending.
    pub fn iter(&self) -> std::iter::Flatten<std::slice::Iter<'_, Vec<SnapshotRegionFilter>>> {
        self.snapshot_region_filters.iter().flatten()
    }

    /// Iterates the snapshot region filters in parallel, which are sorted by base address ascending.
    pub fn par_iter(&self) -> rayon::iter::Flatten<rayon::slice::Iter<'_, Vec<SnapshotRegionFilter>>> {
        self.snapshot_region_filters.par_iter().flatten()
    }

    /// Rebuilds this collection with the requested collection-local result positions removed.
    pub fn remove_results_at_indices(
        &self,
        symbol_registry: &SymbolRegistry,
        deleted_collection_result_indices: &[u64],
    ) -> Self {
        if deleted_collection_result_indices.is_empty() {
            return self.clone();
        }

        let mut deleted_collection_result_indices = deleted_collection_result_indices.to_vec();
        deleted_collection_result_indices.sort_unstable();
        deleted_collection_result_indices.dedup();

        let mut collection_result_index_base = 0u64;
        let mut rebuilt_filters = Vec::new();

        for snapshot_region_filter in self.iter() {
            let filter_result_count = snapshot_region_filter.get_element_count(self.result_value_size_in_bytes, self.memory_alignment);
            let filter_result_index_end = collection_result_index_base.saturating_add(filter_result_count);
            let deleted_filter_result_indices = deleted_collection_result_indices
                .iter()
                .copied()
                .skip_while(|deleted_collection_result_index| *deleted_collection_result_index < collection_result_index_base)
                .take_while(|deleted_collection_result_index| *deleted_collection_result_index < filter_result_index_end)
                .map(|deleted_collection_result_index| deleted_collection_result_index.saturating_sub(collection_result_index_base))
                .collect::<Vec<_>>();

            if deleted_filter_result_indices.is_empty() {
                rebuilt_filters.push(snapshot_region_filter.clone());
                collection_result_index_base = filter_result_index_end;
                continue;
            }

            Self::push_filter_segments_after_deletions(
                &mut rebuilt_filters,
                snapshot_region_filter,
                filter_result_count,
                &deleted_filter_result_indices,
                self.result_value_size_in_bytes,
                self.memory_alignment,
            );

            collection_result_index_base = filter_result_index_end;
        }

        SnapshotRegionFilterCollection::new_with_result_size(
            symbol_registry,
            vec![rebuilt_filters],
            self.data_type_ref.clone(),
            self.memory_alignment,
            self.result_value_size_in_bytes,
        )
        .with_result_container_type(self.result_container_type)
    }

    fn push_filter_segments_after_deletions(
        rebuilt_filters: &mut Vec<SnapshotRegionFilter>,
        snapshot_region_filter: &SnapshotRegionFilter,
        filter_result_count: u64,
        deleted_filter_result_indices: &[u64],
        result_value_size_in_bytes: u64,
        memory_alignment: MemoryAlignment,
    ) {
        let mut kept_segment_start_result_index = 0u64;

        for deleted_filter_result_index in deleted_filter_result_indices {
            if *deleted_filter_result_index > kept_segment_start_result_index {
                let kept_segment_result_count = deleted_filter_result_index.saturating_sub(kept_segment_start_result_index);
                rebuilt_filters.push(Self::create_filter_segment(
                    snapshot_region_filter,
                    kept_segment_start_result_index,
                    kept_segment_result_count,
                    result_value_size_in_bytes,
                    memory_alignment,
                ));
            }

            kept_segment_start_result_index = deleted_filter_result_index.saturating_add(1);
        }

        if kept_segment_start_result_index < filter_result_count {
            let kept_segment_result_count = filter_result_count.saturating_sub(kept_segment_start_result_index);
            rebuilt_filters.push(Self::create_filter_segment(
                snapshot_region_filter,
                kept_segment_start_result_index,
                kept_segment_result_count,
                result_value_size_in_bytes,
                memory_alignment,
            ));
        }
    }

    fn create_filter_segment(
        snapshot_region_filter: &SnapshotRegionFilter,
        segment_start_result_index: u64,
        segment_result_count: u64,
        result_value_size_in_bytes: u64,
        memory_alignment: MemoryAlignment,
    ) -> SnapshotRegionFilter {
        let memory_alignment_in_bytes = std::cmp::max(memory_alignment as u64, 1);
        let segment_base_address = snapshot_region_filter
            .get_base_address()
            .saturating_add(segment_start_result_index.saturating_mul(memory_alignment_in_bytes));
        let segment_size_in_bytes = result_value_size_in_bytes.saturating_add(
            segment_result_count
                .saturating_sub(1)
                .saturating_mul(memory_alignment_in_bytes),
        );

        SnapshotRegionFilter::new(segment_base_address, segment_size_in_bytes)
    }
}
