use crate::scanners::comparers::snapshot_element_range_scanner::{SnapshotElementRangeScanner, SnapshotElementRangeScannerTrait};
use crate::scanners::comparers::snapshot_element_run_length_encoder::SnapshotElementRunLengthEncoder;
use crate::scanners::constraints::scan_constraint::{ConstraintType, ScanConstraint};
use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use squalr_engine_common::dynamic_struct::field_value::FieldValue;
use squalr_engine_memory::memory_alignment::MemoryAlignment;

pub struct SnapshotElementRangeScannerStandard<'a> {
    scanner: SnapshotElementRangeScanner<'a>,
}

impl<'a> SnapshotElementRangeScannerStandard<'a> {
    pub fn new() -> Self {
        return Self {
            scanner: SnapshotElementRangeScanner::new(),
        };
    }

    pub fn initialize(&mut self, element_range: &'a SnapshotElementRange<'a>, constraints: &ScanConstraints) {
        self.scanner.initialize(element_range, constraints);
    }

    pub fn dispose(&mut self) {
        self.scanner.dispose();
    }

    pub fn get_run_length_encoder(&mut self) -> &mut SnapshotElementRunLengthEncoder<'a> {
        return self.scanner.get_run_length_encoder();
    }

    pub fn set_run_length_encoder(&mut self, encoder: SnapshotElementRunLengthEncoder<'a>) {
        self.scanner.set_run_length_encoder(encoder);
    }

    pub fn get_element_range(&self) -> Option<&'a SnapshotElementRange<'a>> {
        return self.scanner.get_element_range();
    }

    pub fn set_element_range(&mut self, element_range: Option<&'a SnapshotElementRange<'a>>) {
        self.scanner.set_element_range(element_range);
    }

    pub fn get_data_type_size(&self) -> usize {
        return self.scanner.get_data_type_size();
    }

    pub fn set_data_type_size(&mut self, size: usize) {
        self.scanner.set_data_type_size(size);
    }

    pub fn get_alignment(&self) -> MemoryAlignment {
        return self.scanner.get_alignment();
    }

    pub fn set_alignment(&mut self, alignment: MemoryAlignment) {
        self.scanner.set_alignment(alignment);
    }

    pub fn get_data_type(&self) -> &FieldValue {
        return self.scanner.get_data_type();
    }

    pub fn set_data_type(&mut self, data_type: FieldValue) {
        self.scanner.set_data_type(data_type);
    }

    pub fn get_on_dispose(&self) -> Option<&Box<dyn Fn() + 'a>> {
        return self.scanner.get_on_dispose();
    }

    pub fn set_on_dispose(&mut self, on_dispose: Option<Box<dyn Fn() + 'a>>) {
        self.scanner.set_on_dispose(on_dispose);
    }

    pub fn do_compare_action(
        &self,
        current_value_ptr: *const u8,
        previous_value_ptr: *const u8,
        constraint: &ScanConstraint,
    ) -> bool {
        match constraint.get_constraint_type() {
            ConstraintType::Unchanged => self.compare_unchanged(current_value_ptr, previous_value_ptr),
            ConstraintType::Changed => self.compare_changed(current_value_ptr, previous_value_ptr),
            ConstraintType::Increased => self.compare_increased(current_value_ptr, previous_value_ptr),
            ConstraintType::Decreased => self.compare_decreased(current_value_ptr, previous_value_ptr),
            ConstraintType::IncreasedByX => self.compare_increased_by(current_value_ptr, previous_value_ptr, constraint.get_constraint_value().cloned().unwrap_or_default()),
            ConstraintType::DecreasedByX => self.compare_decreased_by(current_value_ptr, previous_value_ptr, constraint.get_constraint_value().cloned().unwrap_or_default()),
            ConstraintType::Equal => self.compare_equal(current_value_ptr, constraint.get_constraint_value().cloned().unwrap_or_default()),
            ConstraintType::NotEqual => self.compare_not_equal(current_value_ptr, constraint.get_constraint_value().cloned().unwrap_or_default()),
            ConstraintType::GreaterThan => self.compare_greater_than(current_value_ptr, constraint.get_constraint_value().cloned().unwrap_or_default()),
            ConstraintType::GreaterThanOrEqual => self.compare_greater_than_or_equal(current_value_ptr, constraint.get_constraint_value().cloned().unwrap_or_default()),
            ConstraintType::LessThan => self.compare_less_than(current_value_ptr, constraint.get_constraint_value().cloned().unwrap_or_default()),
            ConstraintType::LessThanOrEqual => self.compare_less_than_or_equal(current_value_ptr, constraint.get_constraint_value().cloned().unwrap_or_default()),
        }
    }

    fn get_current_values(&self, current_value_ptr: *const u8) -> FieldValue {
        let current_value = unsafe { self.read_value(current_value_ptr) };

        return current_value;
    }

    fn get_current_previous_values(&self, current_value_ptr: *const u8, previous_value_ptr: *const u8) -> (FieldValue, FieldValue) {
        let current_value = unsafe { self.read_value(current_value_ptr) };
        let previous_value = unsafe { self.read_value(previous_value_ptr) };

        return (current_value, previous_value);
    }

    unsafe fn read_value(&self, ptr: *const u8) -> FieldValue {
        let data_type = self.scanner.get_data_type();
        match data_type {
            FieldValue::U8(_) => FieldValue::U8(*ptr),
            FieldValue::I8(_) => FieldValue::I8(*ptr as i8),
            FieldValue::U16(_, endian) => {
                let bytes = [*ptr, *ptr.add(1)];
                FieldValue::U16(u16::from_ne_bytes(bytes), endian.clone())
            }
            FieldValue::I16(_, endian) => {
                let bytes = [*ptr, *ptr.add(1)];
                FieldValue::I16(i16::from_ne_bytes(bytes), endian.clone())
            }
            FieldValue::U32(_, endian) => {
                let bytes = [*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3)];
                FieldValue::U32(u32::from_ne_bytes(bytes), endian.clone())
            }
            FieldValue::I32(_, endian) => {
                let bytes = [*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3)];
                FieldValue::I32(i32::from_ne_bytes(bytes), endian.clone())
            }
            FieldValue::U64(_, endian) => {
                let bytes = [*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3), *ptr.add(4), *ptr.add(5), *ptr.add(6), *ptr.add(7)];
                FieldValue::U64(u64::from_ne_bytes(bytes), endian.clone())
            }
            FieldValue::I64(_, endian) => {
                let bytes = [*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3), *ptr.add(4), *ptr.add(5), *ptr.add(6), *ptr.add(7)];
                FieldValue::I64(i64::from_ne_bytes(bytes), endian.clone())
            }
            FieldValue::F32(_, endian) => {
                let bytes = [*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3)];
                let bits = u32::from_ne_bytes(bytes);
                FieldValue::F32(f32::from_bits(bits), endian.clone())
            }
            FieldValue::F64(_, endian) => {
                let bytes = [*ptr, *ptr.add(1), *ptr.add(2), *ptr.add(3), *ptr.add(4), *ptr.add(5), *ptr.add(6), *ptr.add(7)];
                let bits = u64::from_ne_bytes(bytes);
                FieldValue::F64(f64::from_bits(bits), endian.clone())
            }
            _ => unimplemented!("Type not supported"),
        }
    }

    fn compare_changed(&self, current_value_ptr: *const u8, previous_value_ptr: *const u8) -> bool {
        let (current_value, previous_value) = self.get_current_previous_values(current_value_ptr, previous_value_ptr);
        return current_value != previous_value;
    }

    fn compare_unchanged(&self, current_value_ptr: *const u8, previous_value_ptr: *const u8) -> bool {
        let (current_value, previous_value) = self.get_current_previous_values(current_value_ptr, previous_value_ptr);
        return current_value == previous_value;
    }

    fn compare_increased(&self, current_value_ptr: *const u8, previous_value_ptr: *const u8) -> bool {
        let (current_value, previous_value) = self.get_current_previous_values(current_value_ptr, previous_value_ptr);
        return current_value > previous_value;
    }

    fn compare_decreased(&self, current_value_ptr: *const u8, previous_value_ptr: *const u8) -> bool {
        let (current_value, previous_value) = self.get_current_previous_values(current_value_ptr, previous_value_ptr);
        return current_value < previous_value;
    }
    
    fn compare_equal(&self, current_value_ptr: *const u8, value: FieldValue) -> bool {
        let current_value = self.get_current_values(current_value_ptr);
        return current_value == value;
    }
    
    fn compare_not_equal(&self, current_value_ptr: *const u8, value: FieldValue) -> bool {
        let current_value = self.get_current_values(current_value_ptr);
        return current_value != value;
    }
    
    fn compare_greater_than(&self, current_value_ptr: *const u8, value: FieldValue) -> bool {
        let current_value = self.get_current_values(current_value_ptr);
        return current_value > value;
    }
    
    fn compare_greater_than_or_equal(&self, current_value_ptr: *const u8, value: FieldValue) -> bool {
        let current_value = self.get_current_values(current_value_ptr);
        return current_value >= value;
    }
    
    fn compare_less_than(&self, current_value_ptr: *const u8, value: FieldValue) -> bool {
        let current_value = self.get_current_values(current_value_ptr);
        return current_value < value;
    }
    
    fn compare_less_than_or_equal(&self, current_value_ptr: *const u8, value: FieldValue) -> bool {
        let current_value = self.get_current_values(current_value_ptr);
        return current_value <= value;
    }
    
    fn compare_increased_by(&self, current_value_ptr: *const u8, previous_value_ptr: *const u8, value: FieldValue) -> bool {
        let (current_value, previous_value) = self.get_current_previous_values(current_value_ptr, previous_value_ptr);
        return match (current_value, previous_value) {
            (FieldValue::U8(a), FieldValue::U8(b)) => a == b.wrapping_add(value.as_u8().unwrap()),
            (FieldValue::I8(a), FieldValue::I8(b)) => a == b.wrapping_add(value.as_i8().unwrap()),
            (FieldValue::U16(a, _), FieldValue::U16(b, _)) => a == b.wrapping_add(value.as_u16().unwrap()),
            (FieldValue::I16(a, _), FieldValue::I16(b, _)) => a == b.wrapping_add(value.as_i16().unwrap()),
            (FieldValue::U32(a, _), FieldValue::U32(b, _)) => a == b.wrapping_add(value.as_u32().unwrap()),
            (FieldValue::I32(a, _), FieldValue::I32(b, _)) => a == b.wrapping_add(value.as_i32().unwrap()),
            (FieldValue::U64(a, _), FieldValue::U64(b, _)) => a == b.wrapping_add(value.as_u64().unwrap()),
            (FieldValue::I64(a, _), FieldValue::I64(b, _)) => a == b.wrapping_add(value.as_i64().unwrap()),
            _ => false,
        };
    }
    
    fn compare_decreased_by(&self, current_value_ptr: *const u8, previous_value_ptr: *const u8, value: FieldValue) -> bool {
        let (current_value, previous_value) = self.get_current_previous_values(current_value_ptr, previous_value_ptr);
        return match (current_value, previous_value) {
            (FieldValue::U8(a), FieldValue::U8(b)) => a == b.wrapping_sub(value.as_u8().unwrap()),
            (FieldValue::I8(a), FieldValue::I8(b)) => a == b.wrapping_sub(value.as_i8().unwrap()),
            (FieldValue::U16(a, _), FieldValue::U16(b, _)) => a == b.wrapping_sub(value.as_u16().unwrap()),
            (FieldValue::I16(a, _), FieldValue::I16(b, _)) => a == b.wrapping_sub(value.as_i16().unwrap()),
            (FieldValue::U32(a, _), FieldValue::U32(b, _)) => a == b.wrapping_sub(value.as_u32().unwrap()),
            (FieldValue::I32(a, _), FieldValue::I32(b, _)) => a == b.wrapping_sub(value.as_i32().unwrap()),
            (FieldValue::U64(a, _), FieldValue::U64(b, _)) => a == b.wrapping_sub(value.as_u64().unwrap()),
            (FieldValue::I64(a, _), FieldValue::I64(b, _)) => a == b.wrapping_sub(value.as_i64().unwrap()),
            _ => false,
        };
    }
    
}
