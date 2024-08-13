use crate::scanners::comparers::snapshot_element_run_length_encoder::SnapshotElementRunLengthEncoder;
use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use squalr_engine_common::dynamic_struct::field_value::FieldValue;
use squalr_engine_memory::memory_alignment::MemoryAlignment;

pub trait SnapshotElementRangeScannerTrait<'a> {
    fn scan_element_range<'b>(
        &mut self,
        element_range: &'b SnapshotElementRange<'b>,
        constraints: &'b ScanConstraints,
    ) -> Vec<SnapshotElementRange<'b>>
    where
        'b: 'a;

    fn dispose(&mut self);

    // Getters
    fn get_run_length_encoder(&mut self) -> &SnapshotElementRunLengthEncoder<'a>;
    fn get_element_range(&self) -> Option<&'a SnapshotElementRange<'a>>;
    fn get_data_type_size(&self) -> usize;
    fn get_byte_alignment(&self) -> MemoryAlignment;
    fn get_data_type(&self) -> &FieldValue;
    fn get_on_dispose(&self) -> Option<&Box<dyn Fn() + 'a>>;

    // Setters
    fn set_run_length_encoder(&mut self, encoder: SnapshotElementRunLengthEncoder<'a>);
    fn set_element_range(&mut self, element_range: Option<&'a SnapshotElementRange<'a>>);
    fn set_data_type_size(&mut self, size: usize);
    fn set_alignment(&mut self, alignment: MemoryAlignment);
    fn set_data_type(&mut self, data_type: FieldValue);
    fn set_on_dispose(&mut self, on_dispose: Option<Box<dyn Fn() + 'a>>);
}

pub struct SnapshotElementRangeScanner<'a> {
    run_length_encoder: SnapshotElementRunLengthEncoder<'a>,
    element_range: Option<&'a SnapshotElementRange<'a>>,
    data_type_size: usize,
    alignment: MemoryAlignment,
    data_type: FieldValue,
    on_dispose: Option<Box<dyn Fn() + 'a>>,
}

impl<'a> SnapshotElementRangeScanner<'a> {
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

    pub fn initialize(
        &mut self,
        element_range: &'a SnapshotElementRange<'a>,
        constraints: &ScanConstraints,
    ) {
        self.run_length_encoder.initialize(element_range);
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

    pub fn get_run_length_encoder(&mut self) -> &mut SnapshotElementRunLengthEncoder<'a> {
        &mut self.run_length_encoder
    }    

    pub fn set_run_length_encoder(&mut self, encoder: SnapshotElementRunLengthEncoder<'a>) {
        self.run_length_encoder = encoder;
    }

    pub fn get_element_range(&self) -> Option<&'a SnapshotElementRange<'a>> {
        return self.element_range;
    }

    pub fn set_element_range(&mut self, element_range: Option<&'a SnapshotElementRange<'a>>) {
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

    pub fn get_on_dispose(&self) -> Option<&Box<dyn Fn() + 'a>> {
        return self.on_dispose.as_ref();
    }

    pub fn set_on_dispose(&mut self, on_dispose: Option<Box<dyn Fn() + 'a>>) {
        self.on_dispose = on_dispose;
    }
}

impl<'a> SnapshotElementRangeScannerTrait<'a> for SnapshotElementRangeScanner<'a> {
    fn scan_element_range<'b>(
        &mut self,
        element_range: &'b SnapshotElementRange<'b>,
        constraints: &'b ScanConstraints,
    ) -> Vec<SnapshotElementRange<'b>>
    where
        'b: 'a,
    {
        self.run_length_encoder.initialize(element_range);
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

        // Implement the scan logic here
        return vec![];
    }

    fn dispose(&mut self) {
        self.dispose_internal();
    }

    fn get_run_length_encoder(&mut self) -> &SnapshotElementRunLengthEncoder<'a> {
        return self.get_run_length_encoder();
    }

    fn get_element_range(&self) -> Option<&'a SnapshotElementRange<'a>> {
        return self.get_element_range();
    }

    fn get_data_type_size(&self) -> usize {
        return self.get_data_type_size();
    }

    fn get_byte_alignment(&self) -> MemoryAlignment {
        return self.get_byte_alignment();
    }

    fn get_data_type(&self) -> &FieldValue {
        return self.get_data_type();
    }

    fn get_on_dispose(&self) -> Option<&Box<dyn Fn() + 'a>> {
        return self.get_on_dispose();
    }

    fn set_run_length_encoder(&mut self, encoder: SnapshotElementRunLengthEncoder<'a>) {
        self.set_run_length_encoder(encoder);
    }

    fn set_element_range(&mut self, element_range: Option<&'a SnapshotElementRange<'a>>) {
        self.set_element_range(element_range);
    }

    fn set_data_type_size(&mut self, size: usize) {
        self.set_data_type_size(size);
    }

    fn set_alignment(&mut self, alignment: MemoryAlignment) {
        self.set_alignment(alignment);
    }

    fn set_data_type(&mut self, data_type: FieldValue) {
        self.set_data_type(data_type);
    }

    fn set_on_dispose(&mut self, on_dispose: Option<Box<dyn Fn() + 'a>>) {
        self.set_on_dispose(on_dispose);
    }
}
