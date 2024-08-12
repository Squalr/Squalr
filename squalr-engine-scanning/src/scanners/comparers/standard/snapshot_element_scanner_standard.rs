use crate::scanners::comparers::snapshot_element_range_scanner::{SnapshotElementRangeScanner, SnapshotElementRangeScannerTrait};
use crate::scanners::comparers::snapshot_element_run_length_encoder::SnapshotElementRunLengthEncoder;
use crate::scanners::constraints::scan_constraint::{ConstraintType, ScanConstraint};
use crate::scanners::constraints::scan_constraints::ScanConstraints;
use crate::snapshots::snapshot_element_range::SnapshotElementRange;
use squalr_engine_common::dynamic_struct::field_value::FieldValue;
use squalr_engine_memory::memory_alignment::MemoryAlignment;

pub struct SnapshotElementRangeScannerStandard<'a> {
    scanner: SnapshotElementRangeScanner<'a>,
    element_compare: Option<Box<dyn Fn() -> bool + 'a>>,
    current_value_pointer: Option<*const u8>,
    previous_value_pointer: Option<*const u8>,
}

impl<'a> SnapshotElementRangeScannerStandard<'a> {
    pub fn new() -> Self {
        return Self {
            scanner: SnapshotElementRangeScanner::new(),
            element_compare: None,
            current_value_pointer: None,
            previous_value_pointer: None,
        };
    }

    pub fn initialize(&mut self, element_range: &'a SnapshotElementRange<'a>, constraints: &ScanConstraints) {
        self.scanner.initialize(element_range, constraints);
        if let Some(root_constraint) = constraints.get_root_constraint() {
            let scan_constraint = root_constraint.borrow();
            self.element_compare = Some(self.build_compare_actions(&scan_constraint));
        }
        self.initialize_pointers();
    }

    pub fn initialize_no_pinning(&mut self, element_range: &'a SnapshotElementRange<'a>, constraints: &ScanConstraints) {
        self.scanner.initialize(element_range, constraints);
        if let Some(root_constraint) = constraints.get_root_constraint() {
            let scan_constraint = root_constraint.borrow();
            self.element_compare = Some(self.build_compare_actions(&scan_constraint));
        }
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

    pub fn initialize_pointers(&mut self) {
        if let Some(element_range) = self.scanner.get_element_range() {
            unsafe {
                self.current_value_pointer = Some(element_range.get_current_values().as_ptr().offset(element_range.get_region_offset() as isize));
            }

            unsafe {
                self.previous_value_pointer = Some(element_range.get_previous_values().as_ptr().offset(element_range.get_region_offset() as isize));
            }
        }
    }

    pub fn build_compare_actions(&self, constraint: &ScanConstraint) -> Box<dyn Fn() -> bool + 'a> {
        match constraint.constraint() {
            ConstraintType::Unchanged => Box::new(move || self.get_comparison_unchanged()),
            ConstraintType::Changed => Box::new(move || self.get_comparison_changed()),
            ConstraintType::Increased => Box::new(move || self.get_comparison_increased()),
            ConstraintType::Decreased => Box::new(move || self.get_comparison_decreased()),
            ConstraintType::IncreasedByX => {
                let value = constraint.constraint_value().cloned().unwrap_or_default();
                Box::new(move || self.get_comparison_increased_by(value))
            }
            ConstraintType::DecreasedByX => {
                let value = constraint.constraint_value().cloned().unwrap_or_default();
                Box::new(move || self.get_comparison_decreased_by(value))
            }
            ConstraintType::Equal => {
                let value = constraint.constraint_value().cloned().unwrap_or_default();
                Box::new(move || self.get_comparison_equal(value))
            }
            ConstraintType::NotEqual => {
                let value = constraint.constraint_value().cloned().unwrap_or_default();
                Box::new(move || self.get_comparison_not_equal(value))
            }
            ConstraintType::GreaterThan => {
                let value = constraint.constraint_value().cloned().unwrap_or_default();
                Box::new(move || self.get_comparison_greater_than(value))
            }
            ConstraintType::GreaterThanOrEqual => {
                let value = constraint.constraint_value().cloned().unwrap_or_default();
                Box::new(move || self.get_comparison_greater_than_or_equal(value))
            }
            ConstraintType::LessThan => {
                let value = constraint.constraint_value().cloned().unwrap_or_default();
                Box::new(move || self.get_comparison_less_than(value))
            }
            ConstraintType::LessThanOrEqual => {
                let value = constraint.constraint_value().cloned().unwrap_or_default();
                Box::new(move || self.get_comparison_less_than_or_equal(value))
            }
        }
    }

    fn get_current_previous_values(&self) -> Option<(FieldValue, FieldValue)> {
        if let (Some(current_ptr), Some(previous_ptr)) = (self.current_value_pointer, self.previous_value_pointer) {
            let current_value = unsafe { self.read_value(current_ptr) };
            let previous_value = unsafe { self.read_value(previous_ptr) };
            Some((current_value, previous_value))
        } else {
            None
        }
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

    fn get_comparison_changed(&self) -> bool {
        if let Some((current_value, previous_value)) = self.get_current_previous_values() {
            current_value != previous_value
        } else {
            false
        }
    }

    fn get_comparison_unchanged(&self) -> bool {
        if let Some((current_value, previous_value)) = self.get_current_previous_values() {
            current_value == previous_value
        } else {
            false
        }
    }

    fn get_comparison_increased(&self) -> bool {
        if let Some((current_value, previous_value)) = self.get_current_previous_values() {
            current_value > previous_value
        } else {
            false
        }
    }

    fn get_comparison_decreased(&self) -> bool {
        if let Some((current_value, previous_value)) = self.get_current_previous_values() {
            current_value < previous_value
        } else {
            false
        }
    }

    fn get_comparison_equal(&self, value: FieldValue) -> bool {
        if let Some((current_value, _)) = self.get_current_previous_values() {
            current_value == value
        } else {
            false
        }
    }

    fn get_comparison_not_equal(&self, value: FieldValue) -> bool {
        if let Some((current_value, _)) = self.get_current_previous_values() {
            current_value != value
        } else {
            false
        }
    }

    fn get_comparison_greater_than(&self, value: FieldValue) -> bool {
        if let Some((current_value, _)) = self.get_current_previous_values() {
            current_value > value
        } else {
            false
        }
    }

    fn get_comparison_greater_than_or_equal(&self, value: FieldValue) -> bool {
        if let Some((current_value, _)) = self.get_current_previous_values() {
            current_value >= value
        } else {
            false
        }
    }

    fn get_comparison_less_than(&self, value: FieldValue) -> bool {
        if let Some((current_value, _)) = self.get_current_previous_values() {
            current_value < value
        } else {
            false
        }
    }

    fn get_comparison_less_than_or_equal(&self, value: FieldValue) -> bool {
        if let Some((current_value, _)) = self.get_current_previous_values() {
            current_value <= value
        } else {
            false
        }
    }
    fn get_comparison_increased_by(&self, value: FieldValue) -> bool {
        if let Some((current_value, previous_value)) = self.get_current_previous_values() {
            match (current_value, previous_value) {
                (FieldValue::U8(a), FieldValue::U8(b)) => a == b.wrapping_add(value.as_u8().unwrap()),
                (FieldValue::I8(a), FieldValue::I8(b)) => a == b.wrapping_add(value.as_i8().unwrap()),
                (FieldValue::U16(a, _), FieldValue::U16(b, _)) => a == b.wrapping_add(value.as_u16().unwrap()),
                (FieldValue::I16(a, _), FieldValue::I16(b, _)) => a == b.wrapping_add(value.as_i16().unwrap()),
                (FieldValue::U32(a, _), FieldValue::U32(b, _)) => a == b.wrapping_add(value.as_u32().unwrap()),
                (FieldValue::I32(a, _), FieldValue::I32(b, _)) => a == b.wrapping_add(value.as_i32().unwrap()),
                (FieldValue::U64(a, _), FieldValue::U64(b, _)) => a == b.wrapping_add(value.as_u64().unwrap()),
                (FieldValue::I64(a, _), FieldValue::I64(b, _)) => a == b.wrapping_add(value.as_i64().unwrap()),
                _ => false,
            }
        } else {
            false
        }
    }    

    fn get_comparison_decreased_by(&self, value: FieldValue) -> bool {
        if let Some((current_value, previous_value)) = self.get_current_previous_values() {
            match (current_value, previous_value) {
                (FieldValue::U8(a), FieldValue::U8(b)) => a == b.wrapping_sub(value.as_u8().unwrap()),
                (FieldValue::I8(a), FieldValue::I8(b)) => a == b.wrapping_sub(value.as_i8().unwrap()),
                (FieldValue::U16(a, _), FieldValue::U16(b, _)) => a == b.wrapping_sub(value.as_u16().unwrap()),
                (FieldValue::I16(a, _), FieldValue::I16(b, _)) => a == b.wrapping_sub(value.as_i16().unwrap()),
                (FieldValue::U32(a, _), FieldValue::U32(b, _)) => a == b.wrapping_sub(value.as_u32().unwrap()),
                (FieldValue::I32(a, _), FieldValue::I32(b, _)) => a == b.wrapping_sub(value.as_i32().unwrap()),
                (FieldValue::U64(a, _), FieldValue::U64(b, _)) => a == b.wrapping_sub(value.as_u64().unwrap()),
                (FieldValue::I64(a, _), FieldValue::I64(b, _)) => a == b.wrapping_sub(value.as_i64().unwrap()),
                _ => false,
            }
        } else {
            false
        }
    }
}
