use crate::scanners::comparers::snapshot_element_run_length_encoder::SnapshotElementRunLengthEncoder;
use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use squalr_engine_common::dynamic_struct::field_value::FieldValue;
use squalr_engine_memory::memory_alignment::MemoryAlignment;
use std::sync::Arc;

pub trait SnapshotElementRangeScannerTrait {
    fn scan_element_range(
        &mut self,
        element_range: Arc<SnapshotElementRange>,
        constraints: Arc<ScanConstraints>,
    ) -> Vec<Arc<SnapshotElementRange>>;

    fn dispose(&mut self);

    // Getters
    fn get_run_length_encoder(&mut self) -> &SnapshotElementRunLengthEncoder;
    fn get_element_range(&self) -> Option<Arc<SnapshotElementRange>>;
    fn get_data_type_size(&self) -> usize;
    fn get_byte_alignment(&self) -> MemoryAlignment;
    fn get_data_type(&self) -> &FieldValue;
    fn get_on_dispose(&self) -> Option<&Box<dyn Fn()>>;

    // Setters
    fn set_run_length_encoder(&mut self, encoder: SnapshotElementRunLengthEncoder);
    fn set_element_range(&mut self, element_range: Option<Arc<SnapshotElementRange>>);
    fn set_data_type_size(&mut self, size: usize);
    fn set_alignment(&mut self, alignment: MemoryAlignment);
    fn set_data_type(&mut self, data_type: FieldValue);
    fn set_on_dispose(&mut self, on_dispose: Option<Box<dyn Fn()>>);
}

pub struct SnapshotElementRangeScanner {
    run_length_encoder: SnapshotElementRunLengthEncoder,
    element_range: Option<Arc<SnapshotElementRange>>,
    data_type_size: usize,
    alignment: MemoryAlignment,
    data_type: FieldValue,
    on_dispose: Option<Box<dyn Fn()>>,
}

impl SnapshotElementRangeScanner {
    pub fn new() -> Self {
        return Self {
            run_length_encoder: SnapshotElementRunLengthEncoder::new(),
            element_range: None,
            data_type_size: 0,
            alignment: MemoryAlignment::Auto,
            data_type: FieldValue::U8(0),
            on_dispose: None,
        };
    }

    pub fn initialize(&mut self, element_range: Arc<SnapshotElementRange>, constraints: Arc<ScanConstraints>) {
        self.run_length_encoder.initialize(element_range.clone());
        self.element_range = Some(element_range);
        self.data_type = constraints.get_element_type().clone();
        self.data_type_size = self.data_type.size_in_bytes();

        self.alignment = if let FieldValue::Bytes(_) = self.data_type {
            MemoryAlignment::Alignment1
        } else {
            if constraints.get_byte_alignment() == MemoryAlignment::Auto {
                MemoryAlignment::from(self.data_type_size as i32)
            } else {
                constraints.get_byte_alignment()
            }
        };
    }

    fn dispose_internal(&mut self) {
        self.element_range = None;

        if let Some(callback) = self.on_dispose.take() {
            callback();
        }
    }

    pub fn get_run_length_encoder(&mut self) -> &mut SnapshotElementRunLengthEncoder {
        &mut self.run_length_encoder
    }    

    pub fn set_run_length_encoder(&mut self, encoder: SnapshotElementRunLengthEncoder) {
        self.run_length_encoder = encoder;
    }

    pub fn get_element_range(&self) -> Option<Arc<SnapshotElementRange>> {
        return self.element_range.clone();
    }

    pub fn set_element_range(&mut self, element_range: Option<Arc<SnapshotElementRange>>) {
        self.element_range = element_range;
    }

    pub fn get_data_type_size(&self) -> usize {
        return self.data_type_size;
    }

    pub fn set_data_type_size(&mut self, size: usize) {
        self.data_type_size = size;
    }

    pub fn get_byte_alignment(&self) -> MemoryAlignment {
        return self.alignment;
    }

    pub fn set_alignment(&mut self, alignment: MemoryAlignment) {
        self.alignment = alignment;
    }

    pub fn get_data_type(&self) -> &FieldValue {
        return &self.data_type;
    }

    pub fn set_data_type(&mut self, data_type: FieldValue) {
        self.data_type = data_type;
    }

    pub fn get_on_dispose(&self) -> Option<&Box<dyn Fn()>> {
        return self.on_dispose.as_ref();
    }

    pub fn set_on_dispose(&mut self, on_dispose: Option<Box<dyn Fn()>>) {
        self.on_dispose = on_dispose;
    }
}

impl SnapshotElementRangeScannerTrait for SnapshotElementRangeScanner {
    fn scan_element_range(&mut self, element_range: Arc<SnapshotElementRange>, constraints: Arc<ScanConstraints>) -> Vec<Arc<SnapshotElementRange>> {
        self.run_length_encoder.initialize(element_range.clone());
        self.element_range = Some(element_range.clone());
        self.data_type = constraints.get_element_type().clone();
        self.data_type_size = self.data_type.size_in_bytes();

        self.alignment = if let FieldValue::Bytes(_) = self.data_type {
            MemoryAlignment::Alignment1
        } else {
            if constraints.get_byte_alignment() == MemoryAlignment::Auto {
                MemoryAlignment::from(self.data_type_size as i32)
            } else {
                constraints.get_byte_alignment()
            }
        };

        // Implement the scan logic here
        return vec![];
    }

    fn dispose(&mut self) {
        self.dispose_internal();
    }

    fn get_run_length_encoder(&mut self) -> &SnapshotElementRunLengthEncoder {
        &self.run_length_encoder
    }

    fn get_element_range(&self) -> Option<Arc<SnapshotElementRange>> {
        self.element_range.clone()
    }

    fn get_data_type_size(&self) -> usize {
        self.data_type_size
    }

    fn get_byte_alignment(&self) -> MemoryAlignment {
        self.alignment
    }

    fn get_data_type(&self) -> &FieldValue {
        &self.data_type
    }

    fn get_on_dispose(&self) -> Option<&Box<dyn Fn()>> {
        self.on_dispose.as_ref()
    }

    fn set_run_length_encoder(&mut self, encoder: SnapshotElementRunLengthEncoder) {
        self.run_length_encoder = encoder;
    }

    fn set_element_range(&mut self, element_range: Option<Arc<SnapshotElementRange>>) {
        self.element_range = element_range;
    }

    fn set_data_type_size(&mut self, size: usize) {
        self.data_type_size = size;
    }

    fn set_alignment(&mut self, alignment: MemoryAlignment) {
        self.alignment = alignment;
    }

    fn set_data_type(&mut self, data_type: FieldValue) {
        self.data_type = data_type;
    }

    fn set_on_dispose(&mut self, on_dispose: Option<Box<dyn Fn()>>) {
        self.on_dispose = on_dispose;
    }
}
