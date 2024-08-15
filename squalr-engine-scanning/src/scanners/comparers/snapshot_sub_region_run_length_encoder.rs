use crate::snapshots::snapshot_sub_region::SnapshotSubRegion;
use std::sync::{Arc, RwLock};

pub struct SnapshotSubRegionRunLengthEncoder {
    run_length_encode_offset: usize,
    is_encoding: bool,
    run_length: usize,
    snapshot_sub_region: Option<Arc<SnapshotSubRegion>>,
    result_regions: Vec<Arc<RwLock<SnapshotSubRegion>>>,
    parent_region_base_address: u64,
}

impl SnapshotSubRegionRunLengthEncoder {
    pub fn new() -> Self {
        Self {
            run_length_encode_offset: 0,
            is_encoding: false,
            run_length: 0,
            snapshot_sub_region: None,
            result_regions: Vec::new(),
            parent_region_base_address: 0,
        }
    }

    pub fn initialize(&mut self, snapshot_sub_region: Arc<SnapshotSubRegion>) {
        self.snapshot_sub_region = Some(snapshot_sub_region.clone());
        self.parent_region_base_address = snapshot_sub_region.parent_region.read().unwrap().get_base_address();
        self.run_length_encode_offset = snapshot_sub_region.get_region_offset();
        self.result_regions.clear();
    }

    pub fn adjust_for_misalignment(&mut self, misalignment_offset: usize) {
        self.run_length_encode_offset = self.run_length_encode_offset.saturating_sub(misalignment_offset);
    }

    pub fn encode_range(&mut self, advance_byte_count: usize) {
        self.run_length += advance_byte_count;
        self.is_encoding = true;
    }

    pub fn finalize_current_encode_checked(&mut self, advance_byte_count: usize) {
        if self.is_encoding {
            if let Some(snapshot_sub_region) = &self.snapshot_sub_region {
                let absolute_address_start = self.parent_region_base_address + self.run_length_encode_offset as u64;
                let absolute_address_end = absolute_address_start + self.run_length as u64;

                if absolute_address_start >= snapshot_sub_region.get_base_element_address()
                    && absolute_address_end <= snapshot_sub_region.get_end_element_address()
                {
                    self.result_regions.push(Arc::new(RwLock::new(SnapshotSubRegion::new_with_offset_and_range(
                        snapshot_sub_region.parent_region.clone(),
                        self.run_length_encode_offset,
                        self.run_length,
                    ))));
                }

                self.run_length_encode_offset += self.run_length;
                self.run_length = 0;
                self.is_encoding = false;
            }
        }

        self.run_length_encode_offset += advance_byte_count;
    }

    pub fn finalize_current_encode_unchecked(&mut self, advance_byte_count: usize) {
        if self.is_encoding && self.run_length > 0 {
            if let Some(snapshot_sub_region) = &self.snapshot_sub_region {
                self.result_regions.push(Arc::new(RwLock::new(SnapshotSubRegion::new_with_offset_and_range(
                    snapshot_sub_region.parent_region.clone(),
                    self.run_length_encode_offset,
                    self.run_length,
                ))));
            }
            self.run_length_encode_offset += self.run_length;
            self.run_length = 0;
            self.is_encoding = false;
        }

        self.run_length_encode_offset += advance_byte_count;
    }

    pub fn get_collected_regions(&self) -> &Vec<Arc<RwLock<SnapshotSubRegion>>> {
        return &self.result_regions;
    }
}
