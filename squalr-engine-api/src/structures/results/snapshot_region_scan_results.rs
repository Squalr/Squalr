use crate::registries::symbols::symbol_registry::SymbolRegistry;
use crate::structures::scan_results::scan_result_ref::ScanResultRef;
use crate::structures::snapshots::snapshot_region::SnapshotRegion;
use crate::structures::{
    data_types::data_type_ref::DataTypeRef,
    data_values::{anonymous_value_string::AnonymousValueString, container_type::ContainerType},
    scan_results::{scan_result_data_type_count::ScanResultDataTypeCount, scan_result_valued::ScanResultValued},
    scanning::filters::snapshot_region_filter::SnapshotRegionFilter,
    scanning::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection,
};
use std::{
    cmp::{Reverse, max},
    collections::{BTreeSet, BinaryHeap},
    iter::Flatten,
    slice::Iter,
};

/// Tracks the scan results for a region while preserving the scan-time filter layout.
pub struct SnapshotRegionScanResults {
    /// The collection of filters produced by a scan for a specific snapshot region.
    snapshot_region_filter_collections: Vec<SnapshotRegionFilterCollection>,
}

struct ScanResultsPageCursor<'a> {
    collection: &'a SnapshotRegionFilterCollection,
    filter_iterator: Flatten<Iter<'a, Vec<SnapshotRegionFilter>>>,
    current_filter: Option<&'a SnapshotRegionFilter>,
    current_filter_result_offset: u64,
    data_type_size_in_bytes: u64,
    is_selected: bool,
}

impl<'a> ScanResultsPageCursor<'a> {
    fn new(
        collection: &'a SnapshotRegionFilterCollection,
        is_selected: bool,
    ) -> Option<Self> {
        let data_type_size_in_bytes = collection.get_result_value_size_in_bytes();
        let mut filter_iterator = collection.iter();
        let current_filter = filter_iterator.next();

        Some(Self {
            collection,
            filter_iterator,
            current_filter,
            current_filter_result_offset: 0,
            data_type_size_in_bytes,
            is_selected,
        })
        .filter(|cursor| cursor.current_filter.is_some())
    }

    fn get_current_address(&self) -> Option<u64> {
        self.current_filter.map(|current_filter| {
            current_filter.get_base_address().saturating_add(
                self.current_filter_result_offset
                    .saturating_mul(max(self.collection.get_memory_alignment() as u64, 1)),
            )
        })
    }

    fn get_remaining_results_in_current_filter(&self) -> u64 {
        self.current_filter
            .map(|current_filter| {
                current_filter
                    .get_element_count(self.data_type_size_in_bytes, self.collection.get_memory_alignment())
                    .saturating_sub(self.current_filter_result_offset)
            })
            .unwrap_or(0)
    }

    fn get_max_consecutive_results_before(
        &self,
        next_competing_address: Option<u64>,
    ) -> u64 {
        let remaining_results_in_current_filter = self.get_remaining_results_in_current_filter();
        let Some(current_address) = self.get_current_address() else {
            return 0;
        };

        if remaining_results_in_current_filter == 0 {
            return 0;
        }

        match next_competing_address {
            Some(next_competing_address) if next_competing_address <= current_address => 1,
            Some(next_competing_address) => {
                let alignment_in_bytes = max(self.collection.get_memory_alignment() as u64, 1);
                let address_distance = next_competing_address.saturating_sub(current_address);
                let consecutive_result_count = address_distance.saturating_sub(1) / alignment_in_bytes + 1;

                remaining_results_in_current_filter.min(consecutive_result_count)
            }
            None => remaining_results_in_current_filter,
        }
    }

    fn advance_results(
        &mut self,
        results_to_advance: u64,
    ) {
        let mut remaining_results_to_advance = results_to_advance;

        while remaining_results_to_advance > 0 {
            let remaining_results_in_current_filter = self.get_remaining_results_in_current_filter();

            if remaining_results_in_current_filter == 0 {
                self.current_filter = self.filter_iterator.next();
                self.current_filter_result_offset = 0;
                continue;
            }

            if remaining_results_to_advance < remaining_results_in_current_filter {
                self.current_filter_result_offset = self
                    .current_filter_result_offset
                    .saturating_add(remaining_results_to_advance);
                return;
            }

            remaining_results_to_advance = remaining_results_to_advance.saturating_sub(remaining_results_in_current_filter);
            self.current_filter = self.filter_iterator.next();
            self.current_filter_result_offset = 0;
        }
    }

    fn build_scan_result(
        &self,
        snapshot_region: &SnapshotRegion,
        symbol_registry: &SymbolRegistry,
        global_scan_result_index: u64,
    ) -> Option<ScanResultValued> {
        let current_address = self.get_current_address()?;
        let data_type_ref = self.collection.get_data_type_ref();
        let current_value = snapshot_region.get_current_value(
            current_address,
            symbol_registry,
            data_type_ref,
            self.collection.get_result_value_size_in_bytes(),
        );
        let previous_value = snapshot_region.get_previous_value(
            current_address,
            symbol_registry,
            data_type_ref,
            self.collection.get_result_value_size_in_bytes(),
        );
        let current_display_values = current_value
            .as_ref()
            .and_then(|data_value| {
                symbol_registry
                    .anonymize_value_to_supported_formats(data_value)
                    .ok()
            })
            .map(|display_values| Self::apply_result_container_type(display_values, self.collection, symbol_registry))
            .unwrap_or_default();
        let previous_display_values = previous_value
            .as_ref()
            .and_then(|data_value| {
                symbol_registry
                    .anonymize_value_to_supported_formats(data_value)
                    .ok()
            })
            .map(|display_values| Self::apply_result_container_type(display_values, self.collection, symbol_registry))
            .unwrap_or_default();
        let icon_id = symbol_registry.get_icon_id(data_type_ref);

        Some(ScanResultValued::new(
            current_address,
            data_type_ref.clone(),
            icon_id,
            current_value,
            current_display_values,
            previous_value,
            previous_display_values,
            ScanResultRef::new(global_scan_result_index),
        ))
    }

    fn apply_result_container_type(
        mut display_values: Vec<AnonymousValueString>,
        collection: &SnapshotRegionFilterCollection,
        symbol_registry: &SymbolRegistry,
    ) -> Vec<AnonymousValueString> {
        let result_container_type = collection.get_result_container_type();

        if !matches!(result_container_type, ContainerType::Array | ContainerType::ArrayFixed(_)) {
            return display_values;
        }

        let unit_size_in_bytes = symbol_registry
            .get_unit_size_in_bytes(collection.get_data_type_ref())
            .max(1);
        let fixed_array_length = match result_container_type {
            ContainerType::ArrayFixed(array_length) => array_length.max(1),
            ContainerType::Array => (collection.get_result_value_size_in_bytes() / unit_size_in_bytes).max(1),
            _ => 1,
        };

        for display_value in &mut display_values {
            display_value.set_container_type(ContainerType::ArrayFixed(fixed_array_length));
        }

        display_values
    }
}

impl SnapshotRegionScanResults {
    pub fn new(snapshot_region_filter_collections: Vec<SnapshotRegionFilterCollection>) -> Self {
        Self {
            snapshot_region_filter_collections,
        }
    }

    /// Gets the scan results (as a snapshot region filter collection) corresponding to the provided data type.
    pub fn get_scan_results_by_data_type(
        &self,
        data_type: &DataTypeRef,
    ) -> Option<&SnapshotRegionFilterCollection> {
        for collection in &self.snapshot_region_filter_collections {
            if collection.get_data_type_ref() == data_type {
                return Some(&collection);
            }
        }

        None
    }

    /// Performs an address-ascending seek to find the specified scan result by index.
    pub fn get_structural_scan_result(
        &self,
        snapshot_region: &SnapshotRegion,
        symbol_registry: &SymbolRegistry,
        global_scan_result_index: u64,
        local_scan_result_index: u64,
    ) -> Option<ScanResultValued> {
        let region_global_scan_result_index_base = global_scan_result_index.saturating_sub(local_scan_result_index);

        self.get_structural_scan_results_page(
            snapshot_region,
            symbol_registry,
            None,
            region_global_scan_result_index_base,
            local_scan_result_index,
            1,
        )
        .into_iter()
        .next()
    }

    /// Collects an address-ascending page of scan results for the specified data type filters.
    pub fn get_scan_results_page(
        &self,
        snapshot_region: &SnapshotRegion,
        symbol_registry: &SymbolRegistry,
        filtered_data_types: Option<&[DataTypeRef]>,
        region_global_scan_result_index_base: u64,
        filtered_scan_result_index_offset: u64,
        page_size: u64,
        deleted_scan_result_indices: &BTreeSet<u64>,
    ) -> Vec<ScanResultValued> {
        if deleted_scan_result_indices.is_empty() {
            return self.get_structural_scan_results_page(
                snapshot_region,
                symbol_registry,
                filtered_data_types,
                region_global_scan_result_index_base,
                filtered_scan_result_index_offset,
                page_size,
            );
        }

        if page_size == 0 {
            return Vec::new();
        }

        let mut collection_cursors = Vec::new();
        let mut collection_cursor_heap: BinaryHeap<Reverse<(u64, usize)>> = BinaryHeap::new();

        for snapshot_region_filter_collection in &self.snapshot_region_filter_collections {
            let is_selected = Self::is_data_type_selected(filtered_data_types, snapshot_region_filter_collection.get_data_type_ref());
            let Some(collection_cursor) = ScanResultsPageCursor::new(snapshot_region_filter_collection, is_selected) else {
                continue;
            };
            let Some(current_address) = collection_cursor.get_current_address() else {
                continue;
            };
            let collection_cursor_index = collection_cursors.len();

            collection_cursors.push(collection_cursor);
            collection_cursor_heap.push(Reverse((current_address, collection_cursor_index)));
        }

        let mut remaining_filtered_results_to_skip = filtered_scan_result_index_offset;
        let mut next_global_scan_result_index = region_global_scan_result_index_base;

        while remaining_filtered_results_to_skip > 0 {
            let Some(Reverse((_current_address, collection_cursor_index))) = collection_cursor_heap.pop() else {
                return Vec::new();
            };
            let next_competing_address = collection_cursor_heap
                .peek()
                .map(|Reverse((current_address, _collection_cursor_index))| *current_address);
            let collection_cursor = &mut collection_cursors[collection_cursor_index];
            let max_consecutive_results = collection_cursor.get_max_consecutive_results_before(next_competing_address);
            let results_to_advance = if collection_cursor.is_selected {
                let visible_selected_result_count = max_consecutive_results.saturating_sub(Self::count_deleted_scan_results_in_range(
                    deleted_scan_result_indices,
                    next_global_scan_result_index,
                    max_consecutive_results,
                ));

                if visible_selected_result_count <= remaining_filtered_results_to_skip {
                    remaining_filtered_results_to_skip = remaining_filtered_results_to_skip.saturating_sub(visible_selected_result_count);
                    max_consecutive_results
                } else {
                    Self::get_structural_results_to_advance_for_visible_skip(
                        deleted_scan_result_indices,
                        next_global_scan_result_index,
                        max_consecutive_results,
                        remaining_filtered_results_to_skip,
                    )
                }
            } else {
                max_consecutive_results
            };

            collection_cursor.advance_results(results_to_advance);
            next_global_scan_result_index = next_global_scan_result_index.saturating_add(results_to_advance);

            if let Some(updated_address) = collection_cursor.get_current_address() {
                collection_cursor_heap.push(Reverse((updated_address, collection_cursor_index)));
            }
        }

        let mut scan_results_page = Vec::with_capacity(page_size as usize);

        while scan_results_page.len() < page_size as usize {
            let Some(Reverse((_current_address, collection_cursor_index))) = collection_cursor_heap.pop() else {
                break;
            };
            let next_competing_address = collection_cursor_heap
                .peek()
                .map(|Reverse((current_address, _collection_cursor_index))| *current_address);
            let collection_cursor = &mut collection_cursors[collection_cursor_index];

            if collection_cursor.is_selected {
                let is_deleted = deleted_scan_result_indices.contains(&next_global_scan_result_index);

                if !is_deleted {
                    if let Some(scan_result) = collection_cursor.build_scan_result(snapshot_region, symbol_registry, next_global_scan_result_index) {
                        scan_results_page.push(scan_result);
                    }
                }

                collection_cursor.advance_results(1);
                next_global_scan_result_index = next_global_scan_result_index.saturating_add(1);
            } else {
                let max_consecutive_results = collection_cursor.get_max_consecutive_results_before(next_competing_address);

                collection_cursor.advance_results(max_consecutive_results);
                next_global_scan_result_index = next_global_scan_result_index.saturating_add(max_consecutive_results);
            }

            if let Some(updated_address) = collection_cursor.get_current_address() {
                collection_cursor_heap.push(Reverse((updated_address, collection_cursor_index)));
            }
        }

        scan_results_page
    }

    /// Gets the number of results contained in this lookup table.
    pub fn get_number_of_results(&self) -> u64 {
        self.snapshot_region_filter_collections
            .iter()
            .map(|collection| collection.get_number_of_results())
            .sum()
    }

    /// Gets the number of visible results contained in this lookup table.
    pub fn get_visible_result_count(
        &self,
        region_global_scan_result_index_base: u64,
        deleted_scan_result_indices: &BTreeSet<u64>,
    ) -> u64 {
        if deleted_scan_result_indices.is_empty() {
            return self.get_number_of_results();
        }

        self.get_number_of_results()
            .saturating_sub(Self::count_deleted_scan_results_in_range(
                deleted_scan_result_indices,
                region_global_scan_result_index_base,
                self.get_number_of_results(),
            ))
    }

    /// Gets the number of results contained in the selected data type filters.
    pub fn get_number_of_results_for_data_types(
        &self,
        filtered_data_types: Option<&[DataTypeRef]>,
    ) -> u64 {
        self.snapshot_region_filter_collections
            .iter()
            .filter(|collection| Self::is_data_type_selected(filtered_data_types, collection.get_data_type_ref()))
            .map(|collection| collection.get_number_of_results())
            .sum()
    }

    /// Gets the number of visible results contained in the selected data type filters.
    pub fn get_visible_result_count_for_data_types(
        &self,
        symbol_registry: &SymbolRegistry,
        filtered_data_types: Option<&[DataTypeRef]>,
        region_global_scan_result_index_base: u64,
        deleted_scan_result_indices: &BTreeSet<u64>,
    ) -> u64 {
        if deleted_scan_result_indices.is_empty() {
            return self.get_number_of_results_for_data_types(filtered_data_types);
        }

        self.get_number_of_results_for_data_types(filtered_data_types)
            .saturating_sub(self.get_deleted_result_count_for_data_types(
                symbol_registry,
                filtered_data_types,
                region_global_scan_result_index_base,
                deleted_scan_result_indices,
            ))
    }

    /// Gets the surviving result counts for each data type in this region.
    pub fn get_result_counts_by_data_type(&self) -> Vec<ScanResultDataTypeCount> {
        self.snapshot_region_filter_collections
            .iter()
            .filter_map(|collection| {
                let result_count = collection.get_number_of_results();

                (result_count > 0).then(|| ScanResultDataTypeCount::new(collection.get_data_type_ref().clone(), result_count))
            })
            .collect()
    }

    /// Gets the visible surviving result counts for each data type in this region.
    pub fn get_visible_result_counts_by_data_type(
        &self,
        symbol_registry: &SymbolRegistry,
        region_global_scan_result_index_base: u64,
        deleted_scan_result_indices: &BTreeSet<u64>,
    ) -> Vec<ScanResultDataTypeCount> {
        let mut visible_result_counts_by_data_type = self.get_result_counts_by_data_type();

        if deleted_scan_result_indices.is_empty() {
            return visible_result_counts_by_data_type;
        }

        let region_global_scan_result_index_end = region_global_scan_result_index_base.saturating_add(self.get_number_of_results());

        for deleted_scan_result_index in deleted_scan_result_indices
            .range(region_global_scan_result_index_base..region_global_scan_result_index_end)
            .copied()
        {
            let local_scan_result_index = deleted_scan_result_index.saturating_sub(region_global_scan_result_index_base);

            if let Some((_address, data_type_ref)) = self.get_scan_result_metadata_by_local_scan_result_index(symbol_registry, local_scan_result_index) {
                if let Some(existing_result_count) = visible_result_counts_by_data_type
                    .iter_mut()
                    .find(|existing_result_count| existing_result_count.data_type_ref == data_type_ref)
                {
                    existing_result_count.result_count = existing_result_count.result_count.saturating_sub(1);
                }
            }
        }

        visible_result_counts_by_data_type.retain(|result_count| result_count.result_count > 0);

        visible_result_counts_by_data_type
    }

    /// Gets the collections of snapshot filters contained by this snapshot region. Generally one collection per data type scanned.
    pub fn get_filter_collections(&self) -> &Vec<SnapshotRegionFilterCollection> {
        &self.snapshot_region_filter_collections
    }

    /// Gets the minimum and maximum bounds across every filter contained by this snapshot region.
    pub fn get_filter_bounds(&self) -> (u64, u64) {
        let mut filter_min_address = u64::MAX;
        let mut filter_max_address = 0u64;
        let mut has_results = false;

        // Collect the minimum and maximum filter bounds. These are used to efficiently build our lookup table.
        for snapshot_region_filter_collection in &self.snapshot_region_filter_collections {
            if snapshot_region_filter_collection.get_number_of_results() == 0 {
                continue;
            }

            has_results = true;
            filter_min_address = filter_min_address.min(snapshot_region_filter_collection.get_filter_minimum_address());
            filter_max_address = filter_max_address.max(snapshot_region_filter_collection.get_filter_maximum_address());
        }

        if !has_results {
            return (0, 0);
        }

        // In the case where there are no filters (or something gone horribly wrong), correct the min to be <= max.
        filter_min_address = filter_min_address.clamp(0u64, filter_max_address);

        (filter_min_address, filter_max_address)
    }

    fn get_structural_scan_results_page(
        &self,
        snapshot_region: &SnapshotRegion,
        symbol_registry: &SymbolRegistry,
        filtered_data_types: Option<&[DataTypeRef]>,
        region_global_scan_result_index_base: u64,
        filtered_scan_result_index_offset: u64,
        page_size: u64,
    ) -> Vec<ScanResultValued> {
        if page_size == 0 {
            return Vec::new();
        }

        let mut collection_cursors = Vec::new();
        let mut collection_cursor_heap: BinaryHeap<Reverse<(u64, usize)>> = BinaryHeap::new();

        for snapshot_region_filter_collection in &self.snapshot_region_filter_collections {
            let is_selected = Self::is_data_type_selected(filtered_data_types, snapshot_region_filter_collection.get_data_type_ref());
            let Some(collection_cursor) = ScanResultsPageCursor::new(snapshot_region_filter_collection, is_selected) else {
                continue;
            };
            let Some(current_address) = collection_cursor.get_current_address() else {
                continue;
            };
            let collection_cursor_index = collection_cursors.len();

            collection_cursors.push(collection_cursor);
            collection_cursor_heap.push(Reverse((current_address, collection_cursor_index)));
        }

        let mut remaining_filtered_results_to_skip = filtered_scan_result_index_offset;
        let mut next_global_scan_result_index = region_global_scan_result_index_base;

        while remaining_filtered_results_to_skip > 0 {
            let Some(Reverse((_current_address, collection_cursor_index))) = collection_cursor_heap.pop() else {
                return Vec::new();
            };
            let next_competing_address = collection_cursor_heap
                .peek()
                .map(|Reverse((current_address, _collection_cursor_index))| *current_address);
            let collection_cursor = &mut collection_cursors[collection_cursor_index];
            let max_consecutive_results = collection_cursor.get_max_consecutive_results_before(next_competing_address);
            let results_to_advance = if collection_cursor.is_selected {
                max_consecutive_results.min(remaining_filtered_results_to_skip)
            } else {
                max_consecutive_results
            };

            collection_cursor.advance_results(results_to_advance);
            next_global_scan_result_index = next_global_scan_result_index.saturating_add(results_to_advance);

            if collection_cursor.is_selected {
                remaining_filtered_results_to_skip = remaining_filtered_results_to_skip.saturating_sub(results_to_advance);
            }

            if let Some(updated_address) = collection_cursor.get_current_address() {
                collection_cursor_heap.push(Reverse((updated_address, collection_cursor_index)));
            }
        }

        let mut scan_results_page = Vec::with_capacity(page_size as usize);

        while scan_results_page.len() < page_size as usize {
            let Some(Reverse((_current_address, collection_cursor_index))) = collection_cursor_heap.pop() else {
                break;
            };
            let next_competing_address = collection_cursor_heap
                .peek()
                .map(|Reverse((current_address, _collection_cursor_index))| *current_address);
            let collection_cursor = &mut collection_cursors[collection_cursor_index];

            if collection_cursor.is_selected {
                if let Some(scan_result) = collection_cursor.build_scan_result(snapshot_region, symbol_registry, next_global_scan_result_index) {
                    scan_results_page.push(scan_result);
                }

                collection_cursor.advance_results(1);
                next_global_scan_result_index = next_global_scan_result_index.saturating_add(1);
            } else {
                let max_consecutive_results = collection_cursor.get_max_consecutive_results_before(next_competing_address);

                collection_cursor.advance_results(max_consecutive_results);
                next_global_scan_result_index = next_global_scan_result_index.saturating_add(max_consecutive_results);
            }

            if let Some(updated_address) = collection_cursor.get_current_address() {
                collection_cursor_heap.push(Reverse((updated_address, collection_cursor_index)));
            }
        }

        scan_results_page
    }

    fn get_scan_result_metadata_by_local_scan_result_index(
        &self,
        _symbol_registry: &SymbolRegistry,
        local_scan_result_index: u64,
    ) -> Option<(u64, DataTypeRef)> {
        let mut collection_cursors = Vec::new();
        let mut collection_cursor_heap: BinaryHeap<Reverse<(u64, usize)>> = BinaryHeap::new();

        for snapshot_region_filter_collection in &self.snapshot_region_filter_collections {
            let Some(collection_cursor) = ScanResultsPageCursor::new(snapshot_region_filter_collection, true) else {
                continue;
            };
            let Some(current_address) = collection_cursor.get_current_address() else {
                continue;
            };
            let collection_cursor_index = collection_cursors.len();

            collection_cursors.push(collection_cursor);
            collection_cursor_heap.push(Reverse((current_address, collection_cursor_index)));
        }

        let mut remaining_results_to_skip = local_scan_result_index;

        while remaining_results_to_skip > 0 {
            let Some(Reverse((_current_address, collection_cursor_index))) = collection_cursor_heap.pop() else {
                return None;
            };
            let next_competing_address = collection_cursor_heap
                .peek()
                .map(|Reverse((current_address, _collection_cursor_index))| *current_address);
            let collection_cursor = &mut collection_cursors[collection_cursor_index];
            let max_consecutive_results = collection_cursor.get_max_consecutive_results_before(next_competing_address);
            let results_to_advance = max_consecutive_results.min(remaining_results_to_skip);

            collection_cursor.advance_results(results_to_advance);
            remaining_results_to_skip = remaining_results_to_skip.saturating_sub(results_to_advance);

            if let Some(updated_address) = collection_cursor.get_current_address() {
                collection_cursor_heap.push(Reverse((updated_address, collection_cursor_index)));
            }
        }

        let Reverse((_current_address, collection_cursor_index)) = collection_cursor_heap.pop()?;
        let collection_cursor = &collection_cursors[collection_cursor_index];
        let current_address = collection_cursor.get_current_address()?;

        Some((current_address, collection_cursor.collection.get_data_type_ref().clone()))
    }

    fn get_deleted_result_count_for_data_types(
        &self,
        symbol_registry: &SymbolRegistry,
        filtered_data_types: Option<&[DataTypeRef]>,
        region_global_scan_result_index_base: u64,
        deleted_scan_result_indices: &BTreeSet<u64>,
    ) -> u64 {
        if deleted_scan_result_indices.is_empty() {
            return 0;
        }

        if filtered_data_types.is_none() {
            return Self::count_deleted_scan_results_in_range(deleted_scan_result_indices, region_global_scan_result_index_base, self.get_number_of_results());
        }

        let region_global_scan_result_index_end = region_global_scan_result_index_base.saturating_add(self.get_number_of_results());
        let mut deleted_result_count: u64 = 0;

        for deleted_scan_result_index in deleted_scan_result_indices
            .range(region_global_scan_result_index_base..region_global_scan_result_index_end)
            .copied()
        {
            let local_scan_result_index = deleted_scan_result_index.saturating_sub(region_global_scan_result_index_base);

            if let Some((_address, data_type_ref)) = self.get_scan_result_metadata_by_local_scan_result_index(symbol_registry, local_scan_result_index) {
                if Self::is_data_type_selected(filtered_data_types, &data_type_ref) {
                    deleted_result_count = deleted_result_count.saturating_add(1);
                }
            }
        }

        deleted_result_count
    }

    fn count_deleted_scan_results_in_range(
        deleted_scan_result_indices: &BTreeSet<u64>,
        range_start_inclusive: u64,
        range_result_count: u64,
    ) -> u64 {
        if range_result_count == 0 {
            return 0;
        }

        let range_end_exclusive = range_start_inclusive.saturating_add(range_result_count);

        deleted_scan_result_indices
            .range(range_start_inclusive..range_end_exclusive)
            .count() as u64
    }

    fn get_structural_results_to_advance_for_visible_skip(
        deleted_scan_result_indices: &BTreeSet<u64>,
        structural_range_start_inclusive: u64,
        structural_range_result_count: u64,
        visible_results_to_skip: u64,
    ) -> u64 {
        if visible_results_to_skip == 0 || structural_range_result_count == 0 {
            return 0;
        }

        let structural_range_end_exclusive = structural_range_start_inclusive.saturating_add(structural_range_result_count);
        let mut current_structural_index = structural_range_start_inclusive;
        let mut remaining_visible_results_to_skip = visible_results_to_skip;

        for deleted_scan_result_index in deleted_scan_result_indices
            .range(structural_range_start_inclusive..structural_range_end_exclusive)
            .copied()
        {
            let undeleted_result_count_before_deleted = deleted_scan_result_index.saturating_sub(current_structural_index);

            if remaining_visible_results_to_skip <= undeleted_result_count_before_deleted {
                return current_structural_index
                    .saturating_sub(structural_range_start_inclusive)
                    .saturating_add(remaining_visible_results_to_skip)
                    .min(structural_range_result_count);
            }

            remaining_visible_results_to_skip = remaining_visible_results_to_skip.saturating_sub(undeleted_result_count_before_deleted);
            current_structural_index = deleted_scan_result_index.saturating_add(1);

            if current_structural_index >= structural_range_end_exclusive {
                return structural_range_result_count;
            }
        }

        current_structural_index
            .saturating_sub(structural_range_start_inclusive)
            .saturating_add(remaining_visible_results_to_skip)
            .min(structural_range_result_count)
    }

    fn is_data_type_selected(
        filtered_data_types: Option<&[DataTypeRef]>,
        data_type_ref: &DataTypeRef,
    ) -> bool {
        match filtered_data_types {
            Some(filtered_data_types) => filtered_data_types.contains(data_type_ref),
            None => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SnapshotRegionScanResults;
    use crate::registries::symbols::symbol_registry::SymbolRegistry;
    use crate::structures::data_types::data_type_ref::DataTypeRef;
    use crate::structures::data_values::anonymous_value_string_format::AnonymousValueStringFormat;
    use crate::structures::data_values::container_type::ContainerType;
    use crate::structures::memory::memory_alignment::MemoryAlignment;
    use crate::structures::memory::normalized_region::NormalizedRegion;
    use crate::structures::scan_results::scan_result_data_type_count::ScanResultDataTypeCount;
    use crate::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
    use crate::structures::scanning::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
    use crate::structures::snapshots::snapshot_region::SnapshotRegion;
    use std::collections::BTreeSet;

    #[test]
    fn get_filter_bounds_ignores_zero_result_collections() {
        let high_address = 0x7FFF_1234_0020;
        let symbol_registry = SymbolRegistry::new();
        let zero_result_collection = SnapshotRegionFilterCollection::new(&symbol_registry, vec![], DataTypeRef::new("u32"), MemoryAlignment::Alignment1);
        let populated_collection = SnapshotRegionFilterCollection::new(
            &symbol_registry,
            vec![vec![SnapshotRegionFilter::new(high_address, 4)]],
            DataTypeRef::new("u32"),
            MemoryAlignment::Alignment1,
        );
        let scan_results = SnapshotRegionScanResults::new(vec![zero_result_collection, populated_collection]);

        assert_eq!(scan_results.get_filter_bounds(), (high_address, high_address + 4));
    }

    #[test]
    fn get_scan_results_page_zips_results_by_address_ascending() {
        let snapshot_region = SnapshotRegion::new(NormalizedRegion::new(0x1000, 0x100), Vec::new());
        let symbol_registry = SymbolRegistry::new();
        let u32_collection = SnapshotRegionFilterCollection::new(
            &symbol_registry,
            vec![vec![SnapshotRegionFilter::new(0x1010, 4)]],
            DataTypeRef::new("u32"),
            MemoryAlignment::Alignment1,
        );
        let u16_collection = SnapshotRegionFilterCollection::new(
            &symbol_registry,
            vec![vec![
                SnapshotRegionFilter::new(0x1004, 2),
                SnapshotRegionFilter::new(0x1014, 2),
            ]],
            DataTypeRef::new("u16"),
            MemoryAlignment::Alignment1,
        );
        let scan_results = SnapshotRegionScanResults::new(vec![u32_collection, u16_collection]);

        let scan_results_page = scan_results.get_scan_results_page(&snapshot_region, &symbol_registry, None, 0, 0, 3, &BTreeSet::new());
        let page_addresses = scan_results_page
            .iter()
            .map(|scan_result| scan_result.get_address())
            .collect::<Vec<_>>();
        let page_data_type_ids = scan_results_page
            .iter()
            .map(|scan_result| scan_result.get_data_type_ref().get_data_type_id().to_string())
            .collect::<Vec<_>>();

        assert_eq!(page_addresses, vec![0x1004, 0x1010, 0x1014]);
        assert_eq!(page_data_type_ids, vec!["u16".to_string(), "u32".to_string(), "u16".to_string()]);
    }

    #[test]
    fn get_scan_results_page_preserves_unfiltered_global_indices_when_filtered() {
        let snapshot_region = SnapshotRegion::new(NormalizedRegion::new(0x2000, 0x100), Vec::new());
        let symbol_registry = SymbolRegistry::new();
        let u32_collection = SnapshotRegionFilterCollection::new(
            &symbol_registry,
            vec![vec![SnapshotRegionFilter::new(0x2010, 4)]],
            DataTypeRef::new("u32"),
            MemoryAlignment::Alignment1,
        );
        let u16_collection = SnapshotRegionFilterCollection::new(
            &symbol_registry,
            vec![vec![SnapshotRegionFilter::new(0x2008, 2)]],
            DataTypeRef::new("u16"),
            MemoryAlignment::Alignment1,
        );
        let u8_collection = SnapshotRegionFilterCollection::new(
            &symbol_registry,
            vec![vec![SnapshotRegionFilter::new(0x2020, 1)]],
            DataTypeRef::new("u8"),
            MemoryAlignment::Alignment1,
        );
        let scan_results = SnapshotRegionScanResults::new(vec![u32_collection, u16_collection, u8_collection]);
        let filtered_data_types = vec![DataTypeRef::new("u32"), DataTypeRef::new("u8")];

        let scan_results_page = scan_results.get_scan_results_page(&snapshot_region, &symbol_registry, Some(&filtered_data_types), 100, 0, 2, &BTreeSet::new());
        let scan_result_global_indices = scan_results_page
            .iter()
            .map(|scan_result| {
                scan_result
                    .get_base_result()
                    .get_scan_result_ref()
                    .get_scan_result_global_index()
            })
            .collect::<Vec<_>>();

        assert_eq!(scan_result_global_indices, vec![101, 102]);
    }

    #[test]
    fn get_scan_results_page_skips_deleted_results_and_preserves_global_indices() {
        let snapshot_region = SnapshotRegion::new(NormalizedRegion::new(0x3000, 0x100), Vec::new());
        let symbol_registry = SymbolRegistry::new();
        let u32_collection = SnapshotRegionFilterCollection::new(
            &symbol_registry,
            vec![vec![SnapshotRegionFilter::new(0x3010, 4)]],
            DataTypeRef::new("u32"),
            MemoryAlignment::Alignment1,
        );
        let u16_collection = SnapshotRegionFilterCollection::new(
            &symbol_registry,
            vec![vec![
                SnapshotRegionFilter::new(0x3004, 2),
                SnapshotRegionFilter::new(0x3014, 2),
            ]],
            DataTypeRef::new("u16"),
            MemoryAlignment::Alignment1,
        );
        let scan_results = SnapshotRegionScanResults::new(vec![u32_collection, u16_collection]);
        let deleted_scan_result_indices = BTreeSet::from([1]);

        let scan_results_page = scan_results.get_scan_results_page(&snapshot_region, &symbol_registry, None, 0, 0, 3, &deleted_scan_result_indices);
        let page_addresses = scan_results_page
            .iter()
            .map(|scan_result| scan_result.get_address())
            .collect::<Vec<_>>();
        let scan_result_global_indices = scan_results_page
            .iter()
            .map(|scan_result| {
                scan_result
                    .get_base_result()
                    .get_scan_result_ref()
                    .get_scan_result_global_index()
            })
            .collect::<Vec<_>>();

        assert_eq!(page_addresses, vec![0x3004, 0x3014]);
        assert_eq!(scan_result_global_indices, vec![0, 2]);
    }

    #[test]
    fn get_visible_result_counts_by_data_type_decrements_deleted_entries() {
        let symbol_registry = SymbolRegistry::new();
        let u32_collection = SnapshotRegionFilterCollection::new(
            &symbol_registry,
            vec![vec![SnapshotRegionFilter::new(0x4010, 4)]],
            DataTypeRef::new("u32"),
            MemoryAlignment::Alignment1,
        );
        let u16_collection = SnapshotRegionFilterCollection::new(
            &symbol_registry,
            vec![vec![SnapshotRegionFilter::new(0x4004, 4)]],
            DataTypeRef::new("u16"),
            MemoryAlignment::Alignment2,
        );
        let scan_results = SnapshotRegionScanResults::new(vec![u32_collection, u16_collection]);
        let deleted_scan_result_indices = BTreeSet::from([1, 2]);

        assert_eq!(
            scan_results.get_visible_result_counts_by_data_type(&symbol_registry, 0, &deleted_scan_result_indices),
            vec![ScanResultDataTypeCount::new(DataTypeRef::new("u16"), 1)]
        );
    }

    #[test]
    fn get_result_counts_by_data_type_skips_zero_result_collections() {
        let symbol_registry = SymbolRegistry::new();
        let zero_result_collection = SnapshotRegionFilterCollection::new(&symbol_registry, vec![], DataTypeRef::new("u32"), MemoryAlignment::Alignment1);
        let populated_collection = SnapshotRegionFilterCollection::new(
            &symbol_registry,
            vec![vec![SnapshotRegionFilter::new(0x1000, 6)]],
            DataTypeRef::new("u16"),
            MemoryAlignment::Alignment2,
        );
        let scan_results = SnapshotRegionScanResults::new(vec![zero_result_collection, populated_collection]);

        assert_eq!(
            scan_results.get_result_counts_by_data_type(),
            vec![ScanResultDataTypeCount::new(DataTypeRef::new("u16"), 3)]
        );
    }

    #[test]
    fn get_scan_results_page_reads_multi_element_result_payloads() {
        let symbol_registry = SymbolRegistry::new();
        let mut snapshot_region = SnapshotRegion::new(NormalizedRegion::new(0x5000, 0x20), Vec::new());
        snapshot_region.current_values = [1_i32.to_le_bytes().as_slice(), 2_i32.to_le_bytes().as_slice()].concat();

        let array_collection = SnapshotRegionFilterCollection::new_with_result_size(
            &symbol_registry,
            vec![vec![SnapshotRegionFilter::new(0x5000, 8)]],
            DataTypeRef::new("i32"),
            MemoryAlignment::Alignment4,
            8,
        );
        snapshot_region.set_scan_results(SnapshotRegionScanResults::new(vec![array_collection]));

        let scan_results_page = snapshot_region
            .get_scan_results()
            .get_scan_results_page(&snapshot_region, &symbol_registry, None, 0, 0, 1, &BTreeSet::new());

        assert_eq!(scan_results_page.len(), 1);
        assert_eq!(
            scan_results_page[0]
                .get_current_value()
                .as_ref()
                .expect("Expected current value for array result.")
                .get_value_bytes(),
            &[1, 0, 0, 0, 2, 0, 0, 0]
        );

        let decimal_display_value = scan_results_page[0]
            .get_current_display_value(AnonymousValueStringFormat::Decimal)
            .expect("Expected decimal display value for array result.");
        assert_eq!(decimal_display_value.get_anonymous_value_string(), "1, 2");
        assert_eq!(decimal_display_value.get_container_type(), ContainerType::ArrayFixed(2));
    }

    #[test]
    fn get_scan_results_page_preserves_single_element_array_container_hints() {
        let symbol_registry = SymbolRegistry::new();
        let mut snapshot_region = SnapshotRegion::new(NormalizedRegion::new(0x6000, 0x20), Vec::new());
        snapshot_region.current_values = 1_i32.to_le_bytes().to_vec();

        let array_collection = SnapshotRegionFilterCollection::new_with_result_size(
            &symbol_registry,
            vec![vec![SnapshotRegionFilter::new(0x6000, 4)]],
            DataTypeRef::new("i32"),
            MemoryAlignment::Alignment4,
            4,
        )
        .with_result_container_type(ContainerType::Array);
        snapshot_region.set_scan_results(SnapshotRegionScanResults::new(vec![array_collection]));

        let scan_results_page = snapshot_region
            .get_scan_results()
            .get_scan_results_page(&snapshot_region, &symbol_registry, None, 0, 0, 1, &BTreeSet::new());

        assert_eq!(scan_results_page.len(), 1);

        let decimal_display_value = scan_results_page[0]
            .get_current_display_value(AnonymousValueStringFormat::Decimal)
            .expect("Expected decimal display value for single-element array result.");
        assert_eq!(decimal_display_value.get_anonymous_value_string(), "1");
        assert_eq!(decimal_display_value.get_container_type(), ContainerType::ArrayFixed(1));
    }
}
