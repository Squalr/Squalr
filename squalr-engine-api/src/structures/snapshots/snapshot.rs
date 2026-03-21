use crate::registries::symbols::symbol_registry::SymbolRegistry;
use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::scan_results::{scan_result_data_type_count::ScanResultDataTypeCount, scan_result_valued::ScanResultValued};
use crate::structures::snapshots::snapshot_region::SnapshotRegion;
use std::{cmp, collections::BTreeSet};

pub struct Snapshot {
    snapshot_regions: Vec<SnapshotRegion>,
    deleted_scan_result_indices: BTreeSet<u64>,
}

/// Represents a snapshot of memory in an external process that contains current and previous values of memory pages.
impl Snapshot {
    /// Creates a new snapshot from the given collection of snapshot regions.
    /// This will automatically sort and remove invalid regions.
    pub fn new() -> Self {
        Self {
            snapshot_regions: vec![],
            deleted_scan_result_indices: BTreeSet::new(),
        }
    }

    /// Assigns new snapshot regions to this snapshot.
    pub fn set_snapshot_regions(
        &mut self,
        snapshot_regions: Vec<SnapshotRegion>,
    ) {
        self.snapshot_regions = snapshot_regions;
        self.clear_deleted_scan_result_indices();
        self.discard_empty_regions();
        self.sort_regions();
    }

    /// Gets a reference to the snapshot regions contained by this snapshot.
    pub fn get_snapshot_regions(&self) -> &Vec<SnapshotRegion> {
        &self.snapshot_regions
    }

    /// Gets a mutable reference to the snapshot regions contained by this snapshot.
    pub fn get_snapshot_regions_mut(&mut self) -> &mut Vec<SnapshotRegion> {
        &mut self.snapshot_regions
    }

    /// Discards all snapshot regions with a size of zero.
    pub fn discard_empty_regions(&mut self) {
        self.snapshot_regions
            .retain(|region| region.get_region_size() > 0);
    }

    /// Sorts all snapshot regions by base address ascending.
    pub fn sort_regions_for_read(&mut self) {
        self.snapshot_regions
            .sort_by_key(|region| cmp::Reverse(region.get_region_size()));
    }

    /// Sorts all snapshot regions by base address ascending.
    pub fn sort_regions(&mut self) {
        self.snapshot_regions
            .sort_by_key(|region| region.get_base_address());
    }

    /// Gets the total number of snapshot regions contained in this snapshot.
    pub fn get_region_count(&self) -> u64 {
        self.snapshot_regions.len() as u64
    }

    /// Gets the total number of bytes contained in this snapshot.
    pub fn get_byte_count(&self) -> u64 {
        self.snapshot_regions
            .iter()
            .map(|region| region.get_region_size())
            .sum()
    }

    /// Seeks to the scan result at the specified index. First this performs a linear scan to locate the snapshot region
    /// containing the index, followed by a binary search to find the exact filter, and finally the scan result.
    pub fn get_scan_result(
        &self,
        global_scan_result_index: u64,
    ) -> Option<ScanResultValued> {
        if self
            .deleted_scan_result_indices
            .contains(&global_scan_result_index)
        {
            return None;
        }

        let mut local_scan_result_index = global_scan_result_index;

        for snapshot_region in &self.snapshot_regions {
            let snapshot_region_scan_results = snapshot_region.get_scan_results();
            let number_of_region_results = snapshot_region_scan_results.get_number_of_results();

            if local_scan_result_index < number_of_region_results {
                return snapshot_region_scan_results.get_structural_scan_result(snapshot_region, global_scan_result_index, local_scan_result_index);
            }

            local_scan_result_index = local_scan_result_index.saturating_sub(number_of_region_results);
        }

        None
    }

    /// Marks the provided scan results as deleted while preserving the underlying structural result layout.
    pub fn delete_scan_results(
        &mut self,
        deleted_scan_result_indices: impl IntoIterator<Item = u64>,
    ) -> u64 {
        let structural_result_count = self.get_structural_number_of_results();
        let mut deleted_result_count: u64 = 0;

        for deleted_scan_result_index in deleted_scan_result_indices {
            if deleted_scan_result_index >= structural_result_count {
                continue;
            }

            if self
                .deleted_scan_result_indices
                .insert(deleted_scan_result_index)
            {
                deleted_result_count = deleted_result_count.saturating_add(1);
            }
        }

        deleted_result_count
    }

    /// Clears every manually deleted scan result index.
    pub fn clear_deleted_scan_result_indices(&mut self) {
        self.deleted_scan_result_indices.clear();
    }

    /// Gets the structural number of scan results contained in this snapshot before deleted indices are applied.
    pub fn get_structural_number_of_results(&self) -> u64 {
        self.snapshot_regions
            .iter()
            .map(|snapshot_region| snapshot_region.get_scan_results().get_number_of_results())
            .sum()
    }

    /// Gets the number of scan results contained in this snapshot.
    pub fn get_number_of_results(&self) -> u64 {
        let mut region_global_scan_result_index_base = 0;
        let mut visible_result_count: u64 = 0;

        for snapshot_region in &self.snapshot_regions {
            let snapshot_region_scan_results = snapshot_region.get_scan_results();
            let region_structural_result_count = snapshot_region_scan_results.get_number_of_results();

            visible_result_count = visible_result_count
                .saturating_add(snapshot_region_scan_results.get_visible_result_count(region_global_scan_result_index_base, &self.deleted_scan_result_indices));
            region_global_scan_result_index_base = region_global_scan_result_index_base.saturating_add(region_structural_result_count);
        }

        visible_result_count
    }

    /// Gets the number of scan results contained in this snapshot for the requested data type filters.
    pub fn get_number_of_results_for_data_types(
        &self,
        filtered_data_types: Option<&[DataTypeRef]>,
    ) -> u64 {
        let mut region_global_scan_result_index_base = 0;
        let mut visible_result_count: u64 = 0;

        for snapshot_region in &self.snapshot_regions {
            let snapshot_region_scan_results = snapshot_region.get_scan_results();
            let region_structural_result_count = snapshot_region_scan_results.get_number_of_results();

            visible_result_count = visible_result_count.saturating_add(snapshot_region_scan_results.get_visible_result_count_for_data_types(
                filtered_data_types,
                region_global_scan_result_index_base,
                &self.deleted_scan_result_indices,
            ));
            region_global_scan_result_index_base = region_global_scan_result_index_base.saturating_add(region_structural_result_count);
        }

        visible_result_count
    }

    /// Gets the surviving result counts for every data type in this snapshot.
    pub fn get_result_counts_by_data_type(&self) -> Vec<ScanResultDataTypeCount> {
        let mut result_counts_by_data_type: Vec<ScanResultDataTypeCount> = Vec::new();
        let mut region_global_scan_result_index_base = 0;

        for snapshot_region in &self.snapshot_regions {
            let snapshot_region_scan_results = snapshot_region.get_scan_results();

            for region_result_count in snapshot_region
                .get_scan_results()
                .get_visible_result_counts_by_data_type(region_global_scan_result_index_base, &self.deleted_scan_result_indices)
            {
                if let Some(existing_result_count) = result_counts_by_data_type
                    .iter_mut()
                    .find(|existing_result_count| existing_result_count.data_type_ref == region_result_count.data_type_ref)
                {
                    existing_result_count.result_count = existing_result_count
                        .result_count
                        .saturating_add(region_result_count.result_count);
                } else {
                    result_counts_by_data_type.push(region_result_count);
                }
            }

            region_global_scan_result_index_base = region_global_scan_result_index_base.saturating_add(snapshot_region_scan_results.get_number_of_results());
        }

        result_counts_by_data_type
    }

    /// Collects a filtered page of scan results in global address-ascending order.
    pub fn get_scan_results_page(
        &self,
        filtered_data_types: Option<&[DataTypeRef]>,
        page_index: u64,
        page_size: u64,
    ) -> (u64, Vec<ScanResultValued>) {
        let total_filtered_result_count = self.get_number_of_results_for_data_types(filtered_data_types);
        let last_page_index = if total_filtered_result_count == 0 {
            0
        } else {
            total_filtered_result_count.saturating_sub(1) / page_size.max(1)
        };
        let effective_page_index = page_index.clamp(0, last_page_index);
        let mut remaining_filtered_results_to_skip = effective_page_index.saturating_mul(page_size);
        let mut remaining_page_capacity = page_size;
        let mut region_global_scan_result_index_base: u64 = 0;
        let mut scan_results_page = Vec::with_capacity(page_size as usize);

        for snapshot_region in &self.snapshot_regions {
            if remaining_page_capacity == 0 {
                break;
            }

            let snapshot_region_scan_results = snapshot_region.get_scan_results();
            let region_filtered_result_count = snapshot_region_scan_results.get_visible_result_count_for_data_types(
                filtered_data_types,
                region_global_scan_result_index_base,
                &self.deleted_scan_result_indices,
            );
            let region_total_result_count = snapshot_region_scan_results.get_number_of_results();

            if remaining_filtered_results_to_skip >= region_filtered_result_count {
                remaining_filtered_results_to_skip = remaining_filtered_results_to_skip.saturating_sub(region_filtered_result_count);
                region_global_scan_result_index_base = region_global_scan_result_index_base.saturating_add(region_total_result_count);
                continue;
            }

            let mut region_scan_results_page = snapshot_region_scan_results.get_scan_results_page(
                snapshot_region,
                filtered_data_types,
                region_global_scan_result_index_base,
                remaining_filtered_results_to_skip,
                remaining_page_capacity,
                &self.deleted_scan_result_indices,
            );

            remaining_page_capacity = remaining_page_capacity.saturating_sub(region_scan_results_page.len() as u64);
            remaining_filtered_results_to_skip = 0;
            region_global_scan_result_index_base = region_global_scan_result_index_base.saturating_add(region_total_result_count);
            scan_results_page.append(&mut region_scan_results_page);
        }

        (effective_page_index, scan_results_page)
    }

    pub fn collect_scan_result_addresses_for_data_type(
        &self,
        data_type_ref: &DataTypeRef,
    ) -> Vec<u64> {
        let symbol_registry = SymbolRegistry::get_instance();
        let data_type_size_in_bytes = symbol_registry.get_unit_size_in_bytes(data_type_ref);
        let mut scan_result_addresses = Vec::new();

        for snapshot_region in &self.snapshot_regions {
            let Some(snapshot_region_filter_collection) = snapshot_region
                .get_scan_results()
                .get_scan_results_by_data_type(data_type_ref)
            else {
                continue;
            };
            let memory_alignment = (snapshot_region_filter_collection.get_memory_alignment() as u64).max(1);

            for snapshot_region_filter in snapshot_region_filter_collection.iter() {
                let filter_base_address = snapshot_region_filter.get_base_address();
                let filter_result_count =
                    snapshot_region_filter.get_element_count(data_type_size_in_bytes, snapshot_region_filter_collection.get_memory_alignment());

                for result_index in 0..filter_result_count {
                    scan_result_addresses.push(filter_base_address.saturating_add(result_index.saturating_mul(memory_alignment)));
                }
            }
        }

        scan_result_addresses
    }
}

#[cfg(test)]
mod tests {
    use super::Snapshot;
    use crate::structures::data_types::data_type_ref::DataTypeRef;
    use crate::structures::memory::memory_alignment::MemoryAlignment;
    use crate::structures::memory::normalized_region::NormalizedRegion;
    use crate::structures::results::snapshot_region_scan_results::SnapshotRegionScanResults;
    use crate::structures::scan_results::scan_result_data_type_count::ScanResultDataTypeCount;
    use crate::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
    use crate::structures::scanning::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
    use crate::structures::snapshots::snapshot_region::SnapshotRegion;

    #[test]
    fn get_scan_result_returns_address_sorted_result_order() {
        let mut snapshot = Snapshot::new();
        let mut snapshot_region = SnapshotRegion::new(NormalizedRegion::new(0x1000, 0x100), Vec::new());
        let u32_collection = SnapshotRegionFilterCollection::new(
            vec![vec![SnapshotRegionFilter::new(0x1010, 4)]],
            DataTypeRef::new("u32"),
            MemoryAlignment::Alignment1,
        );
        let u16_collection = SnapshotRegionFilterCollection::new(
            vec![vec![SnapshotRegionFilter::new(0x1004, 2)]],
            DataTypeRef::new("u16"),
            MemoryAlignment::Alignment1,
        );

        snapshot_region.set_scan_results(SnapshotRegionScanResults::new(vec![u32_collection, u16_collection]));
        snapshot.set_snapshot_regions(vec![snapshot_region]);

        let first_scan_result = snapshot
            .get_scan_result(0)
            .expect("Expected the first scan result.");
        let second_scan_result = snapshot
            .get_scan_result(1)
            .expect("Expected the second scan result.");

        assert_eq!(first_scan_result.get_address(), 0x1004);
        assert_eq!(first_scan_result.get_data_type_ref().get_data_type_id(), "u16");
        assert_eq!(second_scan_result.get_address(), 0x1010);
        assert_eq!(second_scan_result.get_data_type_ref().get_data_type_id(), "u32");
    }

    #[test]
    fn get_scan_results_page_filters_by_data_type_without_rewriting_global_indices() {
        let mut snapshot = Snapshot::new();
        let mut snapshot_region = SnapshotRegion::new(NormalizedRegion::new(0x2000, 0x100), Vec::new());
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

        snapshot_region.set_scan_results(SnapshotRegionScanResults::new(vec![u32_collection, u16_collection]));
        snapshot.set_snapshot_regions(vec![snapshot_region]);

        let filtered_data_types = vec![DataTypeRef::new("u32")];
        let (effective_page_index, scan_results_page) = snapshot.get_scan_results_page(Some(&filtered_data_types), 0, 10);

        assert_eq!(effective_page_index, 0);
        assert_eq!(scan_results_page.len(), 1);
        assert_eq!(scan_results_page[0].get_address(), 0x2010);
        assert_eq!(
            scan_results_page[0]
                .get_base_result()
                .get_scan_result_ref()
                .get_scan_result_global_index(),
            1
        );
    }

    #[test]
    fn get_result_counts_by_data_type_aggregates_across_regions() {
        let mut snapshot = Snapshot::new();
        let mut first_snapshot_region = SnapshotRegion::new(NormalizedRegion::new(0x1000, 0x100), Vec::new());
        let mut second_snapshot_region = SnapshotRegion::new(NormalizedRegion::new(0x2000, 0x100), Vec::new());

        first_snapshot_region.set_scan_results(SnapshotRegionScanResults::new(vec![
            SnapshotRegionFilterCollection::new(
                vec![vec![SnapshotRegionFilter::new(0x1010, 4)]],
                DataTypeRef::new("u32"),
                MemoryAlignment::Alignment1,
            ),
            SnapshotRegionFilterCollection::new(vec![], DataTypeRef::new("u16"), MemoryAlignment::Alignment1),
        ]));
        second_snapshot_region.set_scan_results(SnapshotRegionScanResults::new(vec![
            SnapshotRegionFilterCollection::new(
                vec![vec![SnapshotRegionFilter::new(0x2010, 8)]],
                DataTypeRef::new("u32"),
                MemoryAlignment::Alignment4,
            ),
            SnapshotRegionFilterCollection::new(
                vec![vec![SnapshotRegionFilter::new(0x2020, 4)]],
                DataTypeRef::new("u16"),
                MemoryAlignment::Alignment2,
            ),
        ]));
        snapshot.set_snapshot_regions(vec![first_snapshot_region, second_snapshot_region]);

        assert_eq!(
            snapshot.get_result_counts_by_data_type(),
            vec![
                ScanResultDataTypeCount::new(DataTypeRef::new("u32"), 3),
                ScanResultDataTypeCount::new(DataTypeRef::new("u16"), 2),
            ]
        );
    }

    #[test]
    fn delete_scan_results_hides_results_without_rewriting_global_indices() {
        let mut snapshot = Snapshot::new();
        let mut snapshot_region = SnapshotRegion::new(NormalizedRegion::new(0x3000, 0x100), Vec::new());
        let u8_collection = SnapshotRegionFilterCollection::new(
            vec![vec![SnapshotRegionFilter::new(0x3000, 3)]],
            DataTypeRef::new("u8"),
            MemoryAlignment::Alignment1,
        );

        snapshot_region.set_scan_results(SnapshotRegionScanResults::new(vec![u8_collection]));
        snapshot.set_snapshot_regions(vec![snapshot_region]);

        assert_eq!(snapshot.delete_scan_results([1]), 1);
        assert_eq!(snapshot.get_number_of_results(), 2);
        assert!(snapshot.get_scan_result(1).is_none());

        let (_effective_page_index, scan_results_page) = snapshot.get_scan_results_page(None, 0, 10);
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

        assert_eq!(page_addresses, vec![0x3000, 0x3002]);
        assert_eq!(scan_result_global_indices, vec![0, 2]);
    }

    #[test]
    fn delete_scan_results_updates_filtered_counts_by_data_type() {
        let mut snapshot = Snapshot::new();
        let mut snapshot_region = SnapshotRegion::new(NormalizedRegion::new(0x4000, 0x100), Vec::new());
        let u32_collection = SnapshotRegionFilterCollection::new(
            vec![vec![SnapshotRegionFilter::new(0x4010, 4)]],
            DataTypeRef::new("u32"),
            MemoryAlignment::Alignment1,
        );
        let u16_collection = SnapshotRegionFilterCollection::new(
            vec![vec![SnapshotRegionFilter::new(0x4004, 4)]],
            DataTypeRef::new("u16"),
            MemoryAlignment::Alignment2,
        );

        snapshot_region.set_scan_results(SnapshotRegionScanResults::new(vec![u32_collection, u16_collection]));
        snapshot.set_snapshot_regions(vec![snapshot_region]);
        snapshot.delete_scan_results([1, 2]);

        let filtered_u32_data_types = vec![DataTypeRef::new("u32")];

        assert_eq!(snapshot.get_number_of_results(), 1);
        assert_eq!(snapshot.get_number_of_results_for_data_types(Some(&filtered_u32_data_types)), 0);
        assert_eq!(
            snapshot.get_result_counts_by_data_type(),
            vec![ScanResultDataTypeCount::new(DataTypeRef::new("u16"), 1)]
        );
    }

    #[test]
    fn set_snapshot_regions_clears_deleted_scan_result_indices() {
        let mut snapshot = Snapshot::new();
        let mut snapshot_region = SnapshotRegion::new(NormalizedRegion::new(0x5000, 0x100), Vec::new());
        let u8_collection = SnapshotRegionFilterCollection::new(
            vec![vec![SnapshotRegionFilter::new(0x5000, 2)]],
            DataTypeRef::new("u8"),
            MemoryAlignment::Alignment1,
        );

        snapshot_region.set_scan_results(SnapshotRegionScanResults::new(vec![u8_collection]));
        snapshot.set_snapshot_regions(vec![snapshot_region]);
        snapshot.delete_scan_results([1]);

        let mut replacement_snapshot_region = SnapshotRegion::new(NormalizedRegion::new(0x6000, 0x100), Vec::new());
        let replacement_u8_collection = SnapshotRegionFilterCollection::new(
            vec![vec![SnapshotRegionFilter::new(0x6000, 2)]],
            DataTypeRef::new("u8"),
            MemoryAlignment::Alignment1,
        );
        replacement_snapshot_region.set_scan_results(SnapshotRegionScanResults::new(vec![replacement_u8_collection]));

        snapshot.set_snapshot_regions(vec![replacement_snapshot_region]);

        assert_eq!(snapshot.get_number_of_results(), 2);
        assert!(snapshot.get_scan_result(1).is_some());
    }
}
