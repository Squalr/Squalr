use crate::registries::symbols::symbol_registry::SymbolRegistry;
use crate::structures::scan_results::scan_result_ref::ScanResultRef;
use crate::structures::snapshots::snapshot_region::SnapshotRegion;
use crate::structures::{
    data_types::data_type_ref::DataTypeRef,
    scan_results::{scan_result_data_type_count::ScanResultDataTypeCount, scan_result_valued::ScanResultValued},
    scanning::filters::snapshot_region_filter::SnapshotRegionFilter,
    scanning::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection,
};
use std::{
    cmp::{Reverse, max},
    collections::BinaryHeap,
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
        let data_type_size_in_bytes = SymbolRegistry::get_instance().get_unit_size_in_bytes(collection.get_data_type_ref());
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
        global_scan_result_index: u64,
    ) -> Option<ScanResultValued> {
        let current_address = self.get_current_address()?;
        let symbol_registry = SymbolRegistry::get_instance();
        let data_type_ref = self.collection.get_data_type_ref();
        let current_value = snapshot_region.get_current_value(current_address, data_type_ref);
        let previous_value = snapshot_region.get_previous_value(current_address, data_type_ref);
        let current_display_values = current_value
            .as_ref()
            .and_then(|data_value| {
                symbol_registry
                    .anonymize_value_to_supported_formats(data_value)
                    .ok()
            })
            .unwrap_or_default();
        let previous_display_values = previous_value
            .as_ref()
            .and_then(|data_value| {
                symbol_registry
                    .anonymize_value_to_supported_formats(data_value)
                    .ok()
            })
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
    pub fn get_scan_result(
        &self,
        snapshot_region: &SnapshotRegion,
        global_scan_result_index: u64,
        local_scan_result_index: u64,
    ) -> Option<ScanResultValued> {
        let region_global_scan_result_index_base = global_scan_result_index.saturating_sub(local_scan_result_index);

        self.get_scan_results_page(snapshot_region, None, region_global_scan_result_index_base, local_scan_result_index, 1)
            .into_iter()
            .next()
    }

    /// Collects an address-ascending page of scan results for the specified data type filters.
    pub fn get_scan_results_page(
        &self,
        snapshot_region: &SnapshotRegion,
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
                if let Some(scan_result) = collection_cursor.build_scan_result(snapshot_region, next_global_scan_result_index) {
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

    /// Gets the number of results contained in this lookup table.
    pub fn get_number_of_results(&self) -> u64 {
        self.snapshot_region_filter_collections
            .iter()
            .map(|collection| collection.get_number_of_results())
            .sum()
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
    use crate::structures::data_types::data_type_ref::DataTypeRef;
    use crate::structures::memory::memory_alignment::MemoryAlignment;
    use crate::structures::memory::normalized_region::NormalizedRegion;
    use crate::structures::scan_results::scan_result_data_type_count::ScanResultDataTypeCount;
    use crate::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
    use crate::structures::scanning::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
    use crate::structures::snapshots::snapshot_region::SnapshotRegion;

    #[test]
    fn get_filter_bounds_ignores_zero_result_collections() {
        let high_address = 0x7FFF_1234_0020;
        let zero_result_collection = SnapshotRegionFilterCollection::new(vec![], DataTypeRef::new("u32"), MemoryAlignment::Alignment1);
        let populated_collection = SnapshotRegionFilterCollection::new(
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
        let u32_collection = SnapshotRegionFilterCollection::new(
            vec![vec![SnapshotRegionFilter::new(0x1010, 4)]],
            DataTypeRef::new("u32"),
            MemoryAlignment::Alignment1,
        );
        let u16_collection = SnapshotRegionFilterCollection::new(
            vec![vec![
                SnapshotRegionFilter::new(0x1004, 2),
                SnapshotRegionFilter::new(0x1014, 2),
            ]],
            DataTypeRef::new("u16"),
            MemoryAlignment::Alignment1,
        );
        let scan_results = SnapshotRegionScanResults::new(vec![u32_collection, u16_collection]);

        let scan_results_page = scan_results.get_scan_results_page(&snapshot_region, None, 0, 0, 3);
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
        let u32_collection = SnapshotRegionFilterCollection::new(
            vec![vec![SnapshotRegionFilter::new(0x2010, 4)]],
            DataTypeRef::new("u32"),
            MemoryAlignment::Alignment1,
        );
        let u16_collection = SnapshotRegionFilterCollection::new(
            vec![vec![SnapshotRegionFilter::new(0x2008, 2)]],
            DataTypeRef::new("u16"),
            MemoryAlignment::Alignment1,
        );
        let u8_collection = SnapshotRegionFilterCollection::new(
            vec![vec![SnapshotRegionFilter::new(0x2020, 1)]],
            DataTypeRef::new("u8"),
            MemoryAlignment::Alignment1,
        );
        let scan_results = SnapshotRegionScanResults::new(vec![u32_collection, u16_collection, u8_collection]);
        let filtered_data_types = vec![DataTypeRef::new("u32"), DataTypeRef::new("u8")];

        let scan_results_page = scan_results.get_scan_results_page(&snapshot_region, Some(&filtered_data_types), 100, 0, 2);
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
    fn get_result_counts_by_data_type_skips_zero_result_collections() {
        let zero_result_collection = SnapshotRegionFilterCollection::new(vec![], DataTypeRef::new("u32"), MemoryAlignment::Alignment1);
        let populated_collection = SnapshotRegionFilterCollection::new(
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
}
