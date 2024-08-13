use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use std::sync::Arc;

pub struct SnapshotElementRunLengthEncoder {
    run_length_encode_offset: usize,
    is_encoding: bool,
    run_length: usize,
    element_range: Option<Arc<SnapshotElementRange>>,
    result_regions: Vec<Arc<SnapshotElementRange>>,
}

impl SnapshotElementRunLengthEncoder {
    pub fn new() -> Self {
        Self {
            run_length_encode_offset: 0,
            is_encoding: false,
            run_length: 0,
            element_range: None,
            result_regions: Vec::new(),
        }
    }

    pub fn initialize(&mut self, element_range: Arc<SnapshotElementRange>) {
        self.element_range = Some(element_range.clone());
        self.run_length_encode_offset = element_range.region_offset;
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
            if let Some(element_range) = &self.element_range {
                let absolute_address_start = element_range.parent_region.borrow().get_base_address()
                    + self.run_length_encode_offset as u64;
                let absolute_address_end = absolute_address_start + self.run_length as u64;

                if absolute_address_start >= element_range.get_base_element_address()
                    && absolute_address_end <= element_range.get_end_element_address()
                {
                    self.result_regions.push(Arc::new(SnapshotElementRange::with_offset_and_range(
                        element_range.parent_region.clone(),
                        self.run_length_encode_offset,
                        self.run_length,
                    )));
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
            if let Some(element_range) = &self.element_range {
                self.result_regions.push(Arc::new(SnapshotElementRange::with_offset_and_range(
                    element_range.parent_region.clone(),
                    self.run_length_encode_offset,
                    self.run_length,
                )));
            }
            self.run_length_encode_offset += self.run_length;
            self.run_length = 0;
            self.is_encoding = false;
        }

        self.run_length_encode_offset += advance_byte_count;
    }

    pub fn get_collected_regions(&self) -> &Vec<Arc<SnapshotElementRange>> {
        &self.result_regions
    }
}

impl Drop for SnapshotElementRunLengthEncoder {
    fn drop(&mut self) {
        // Perform cleanup here if needed, though Rust will handle most cleanup via ownership and RAII.
    }
}
