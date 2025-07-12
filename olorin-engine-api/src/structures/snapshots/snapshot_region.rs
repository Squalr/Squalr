use crate::structures::data_types::data_type_ref::DataTypeRef;
use crate::structures::data_values::data_value::DataValue;
use crate::structures::memory::normalized_region::NormalizedRegion;
use crate::structures::results::snapshot_region_scan_results::SnapshotRegionScanResults;
use crate::structures::scanning::filters::snapshot_region_filter::SnapshotRegionFilter;
use crate::structures::scanning::filters::snapshot_region_filter_collection::SnapshotRegionFilterCollection;
use crate::structures::scanning::parameters::element_scan::element_scan_value::ElementScanValue;

/// Defines a contiguous region of memory within a snapshot.
/// JIRA: Please no public fields. These were made public to support pushing memory reading functionality into a trait.
pub struct SnapshotRegion {
    /// The underlying region that contains the start address and length of this snapshot.
    normalized_region: NormalizedRegion,

    /// The most recent values collected from memory within this snapshot region bounds.
    pub current_values: Vec<u8>,

    /// The prior values collected from memory within this snapshot region bounds.
    pub previous_values: Vec<u8>,

    /// Any OS level page boundaries that may sub-divide this snapshot region.
    pub page_boundaries: Vec<u64>,

    /// The current scan results on this snapshot region.
    scan_results: SnapshotRegionScanResults,
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
            scan_results: SnapshotRegionScanResults::new(vec![]),
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
        let data_type_size = data_type.get_unit_size_in_bytes();

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
        let data_type_size = data_type.get_unit_size_in_bytes();

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

    /// Gets a pointer to the first current value element in the specified filter contained within this snapshot region.
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

    /// Gets a pointer to the first previous value element in the specified filter contained within this snapshot region.
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

    // JIRA: Okay great, what about struct scans and whatnot.
    pub fn initialize_scan_results(
        &mut self,
        data_values_and_alignments: &Vec<ElementScanValue>,
    ) {
        if self.scan_results.get_filter_collections().len() > 0 {
            return;
        }

        let snapshot_region_filter_collections = data_values_and_alignments
            .iter()
            .map(|data_value_and_alignment| {
                SnapshotRegionFilterCollection::new(
                    vec![vec![SnapshotRegionFilter::new(
                        self.get_base_address(),
                        self.get_region_size(),
                    )]],
                    data_value_and_alignment.get_data_type().clone(),
                    data_value_and_alignment.get_memory_alignment(),
                )
            })
            .collect();

        self.scan_results = SnapshotRegionScanResults::new(snapshot_region_filter_collections)
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
